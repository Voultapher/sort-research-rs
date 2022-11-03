/*
	Copyright (C) 2014-2022 Igor van den Hoven ivdhoven@gmail.com
*/

/*
	Permission is hereby granted, free of charge, to any person obtaining
	a copy of this software and associated documentation files (the
	"Software"), to deal in the Software without restriction, including
	without limitation the rights to use, copy, modify, merge, publish,
	distribute, sublicense, and/or sell copies of the Software, and to
	permit persons to whom the Software is furnished to do so, subject to
	the following conditions:

	The above copyright notice and this permission notice shall be
	included in all copies or substantial portions of the Software.

	THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
	EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
	MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
	IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
	CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
	TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
	SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

/*
	crumsort 1.1.5.3
*/

#define CRUM_AUX 512
#define CRUM_OUT  24

void FUNC(fulcrum_partition)(VAR *array, VAR *swap, VAR *max, size_t swap_size, size_t nmemb, CMPFUNC *cmp);

void FUNC(crum_analyze)(VAR *array, VAR *swap, size_t swap_size, size_t nmemb, CMPFUNC *cmp)
{
	char loop, asum, zsum;
	size_t cnt, abalance = 0, zbalance = 0, astreaks = 0, zstreaks = 0;
	VAR *pta, *ptz, tmp;

	pta = array;
	ptz = array + nmemb - 2;

	for (cnt = nmemb ; cnt > 64 ; cnt -= 64)
	{
		for (asum = zsum = 0, loop = 32 ; loop ; loop--)
		{
			asum += cmp(pta, pta + 1) > 0; pta++;
			zsum += cmp(ptz, ptz + 1) > 0; ptz--;
		}
		astreaks += (asum == 0) | (asum == 32);
		zstreaks += (zsum == 0) | (zsum == 32);
		abalance += asum;
		zbalance += zsum;
	}

	while (--cnt)
	{
		zbalance += cmp(ptz, ptz + 1) > 0; ptz--;
	}

	if (abalance + zbalance == 0)
	{
		return;
	}

	if (abalance + zbalance == nmemb - 1)
	{
		ptz = array + nmemb;
		pta = array;

		cnt = nmemb / 2;

		do
		{
			tmp = *pta; *pta++ = *--ptz; *ptz = tmp;
		}
		while (--cnt);

		return;
	}

	if (astreaks + zstreaks > nmemb / 80)
	{
		if (nmemb >= 512)
		{
			size_t block = pta - array;

			if (astreaks < nmemb / 128)
			{
				FUNC(fulcrum_partition)(array, swap, NULL, swap_size, block, cmp);
			}
			else if (abalance)
			{
				FUNC(quadsort_swap)(array, swap, swap_size, block, cmp);
			}

			if (zstreaks < nmemb / 128)
			{
				FUNC(fulcrum_partition)(array + block, swap, NULL, swap_size, nmemb - block, cmp);
			}
			else if (zbalance)
			{
				FUNC(quadsort_swap)(array + block, swap, swap_size, nmemb - block, cmp);
			}
			FUNC(blit_merge_block)(array, swap, swap_size, block, nmemb - block, cmp);
		}
		else
		{
			FUNC(quadsort_swap)(array, swap, swap_size, nmemb, cmp);
		}
		return;
	}
	FUNC(fulcrum_partition)(array, swap, NULL, swap_size, nmemb, cmp);
}

// The next 3 functions are used for pivot selection

VAR *FUNC(crum_median_of_sqrt)(VAR *array, VAR *swap, size_t swap_size, size_t nmemb, CMPFUNC *cmp)
{
	VAR *pta, *piv;
	size_t cnt, sqrt, div;

	sqrt = nmemb < 65536 ? 16 : nmemb < 262144 ? 128 : 256;

	div = nmemb / sqrt;

	pta = array + nmemb - 1;
	piv = array + sqrt;

	for (cnt = sqrt ; cnt ; cnt--)
	{
		swap[0] = *--piv; *piv = *pta; *pta = swap[0];

		pta -= div;
	}
	FUNC(quadsort_swap)(piv, swap, swap_size, sqrt, cmp);

	return piv + sqrt / 2;
}

size_t FUNC(crum_median_of_three)(VAR *array, size_t v0, size_t v1, size_t v2, CMPFUNC *cmp)
{
	size_t v[3] = {v0, v1, v2};
	char x, y, z;

	x = cmp(array + v0, array + v1) > 0;
	y = cmp(array + v0, array + v2) > 0;
	z = cmp(array + v1, array + v2) > 0;

	return v[(x == y) + (y ^ z)];
}

VAR *FUNC(crum_median_of_nine)(VAR *array, size_t nmemb, CMPFUNC *cmp)
{
	size_t x, y, z, div = nmemb / 16;

	x = FUNC(crum_median_of_three)(array, div * 2, div * 1, div * 4, cmp);
	y = FUNC(crum_median_of_three)(array, div * 8, div * 6, div * 10, cmp);
	z = FUNC(crum_median_of_three)(array, div * 14, div * 12, div * 15, cmp);

	return array + FUNC(crum_median_of_three)(array, x, y, z, cmp);
}

size_t FUNC(fulcrum_default_partition)(VAR *array, VAR *swap, VAR *ptx, VAR *piv, size_t swap_size, size_t nmemb, CMPFUNC *cmp)
{
	size_t cnt, val, i, m = 0;
	VAR *ptl, *ptr, *pta, *tpa;

	if (nmemb <= swap_size)
	{
		cnt = nmemb / 8;

		do for (i = 8 ; i ; i--)
		{
			val = cmp(ptx, piv) <= 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
		}
		while (--cnt);

		for (cnt = nmemb % 8 ; cnt ; cnt--)
		{
			val = cmp(ptx, piv) <= 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
		}
		memcpy(array + m, swap - nmemb, sizeof(VAR) * (nmemb - m));

		return m;
	}

	memcpy(swap, array, 16 * sizeof(VAR));
	memcpy(swap + 16, array + nmemb - 16, 16 * sizeof(VAR));

	ptl = array;
	ptr = array + nmemb - 1;

	pta = array + 16;
	tpa = array + nmemb - 17;

	cnt = nmemb / 16 - 2;

	while (1)
	{
		if (pta - ptl - m <= 16)
		{
			if (cnt-- == 0) break;

			for (i = 16 ; i ; i--)
			{
				val = cmp(pta, piv) <= 0; ptl[m] = ptr[m] = *pta++; m += val; ptr--;
			}
		}
		if (pta - ptl - m > 16)
		{
			if (cnt-- == 0) break;

			for (i = 16 ; i ; i--)
			{
				val = cmp(tpa, piv) <= 0; ptl[m] = ptr[m] = *tpa--; m += val; ptr--;
			}
		}
	}

	if (pta - ptl - m <= 16)
	{
		for (cnt = nmemb % 16 ; cnt ; cnt--)
		{
			val = cmp(pta, piv) <= 0; ptl[m] = ptr[m] = *pta++; m += val; ptr--;
		}
	}
	else
	{
		for (cnt = nmemb % 16 ; cnt ; cnt--)
		{
			val = cmp(tpa, piv) <= 0; ptl[m] = ptr[m] = *tpa--; m += val; ptr--;
		}
	}
	pta = swap;

	for (cnt = 32 ; cnt ; cnt--)
	{
		val = cmp(pta, piv) <= 0; ptl[m] = ptr[m] = *pta++; m += val; ptr--;
	}
	return m;
}

// As per suggestion by Marshall Lochbaum to improve generic data handling, the original concept is from pdqsort

size_t FUNC(fulcrum_reverse_partition)(VAR *array, VAR *swap, VAR *ptx, VAR *piv, size_t swap_size, size_t nmemb, CMPFUNC *cmp)
{
	size_t cnt, val, i, m = 0;
	VAR *ptl, *ptr, *pta, *tpa;

	if (nmemb <= swap_size)
	{
		cnt = nmemb / 8;

		do for (i = 8 ; i ; i--)
		{
			val = cmp(piv, ptx) > 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
		}
		while (--cnt);

		for (cnt = nmemb % 8 ; cnt ; cnt--)
		{
			val = cmp(piv, ptx) > 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
		}
		memcpy(array + m, swap - nmemb, (nmemb - m) * sizeof(VAR));

		return m;
	}

	memcpy(swap, array, 16 * sizeof(VAR));
	memcpy(swap + 16, array + nmemb - 16, 16 * sizeof(VAR));

	ptl = array;
	ptr = array + nmemb - 1;

	pta = array + 16;
	tpa = array + nmemb - 17;

	cnt = nmemb / 16 - 2;

	while (1)
	{
		if (pta - ptl - m <= 16)
		{
			if (cnt-- == 0) break;

			for (i = 16 ; i ; i--)
			{
				val = cmp(piv, pta) > 0; ptl[m] = ptr[m] = *pta++; m += val; ptr--;
			}
		}
		if (pta - ptl - m > 16)
		{
			if (cnt-- == 0) break;

			for (i = 16 ; i ; i--)
			{
				val = cmp(piv, tpa) > 0; ptl[m] = ptr[m] = *tpa--; m += val; ptr--;
			}
		}
	}

	if (pta - ptl - m <= 16)
	{
		for (cnt = nmemb % 16 ; cnt ; cnt--)
		{
			val = cmp(piv, pta) > 0; ptl[m] = ptr[m] = *pta++; m += val; ptr--;
		}
	}
	else
	{
		for (cnt = nmemb % 16 ; cnt ; cnt--)
		{
			val = cmp(piv, tpa) > 0; ptl[m] = ptr[m] = *tpa--; m += val; ptr--;
		}
	}
	pta = swap;

	for (cnt = 32 ; cnt ; cnt--)
	{
		val = cmp(piv, pta) > 0; ptl[m] = ptr[m] = *pta++; m += val; ptr--;
	}
	return m;
}

void FUNC(fulcrum_partition)(VAR *array, VAR *swap, VAR *max, size_t swap_size, size_t nmemb, CMPFUNC *cmp)
{
	size_t a_size, s_size;
	VAR *ptp, piv;

	while (1)
	{
		if (nmemb <= 2048)
		{
			ptp = FUNC(crum_median_of_nine)(array, nmemb, cmp);
		}
		else
		{
			ptp = FUNC(crum_median_of_sqrt)(array, swap, swap_size, nmemb, cmp);
		}
		piv = *ptp;

		if (max && cmp(max, &piv) <= 0)
		{
			a_size = FUNC(fulcrum_reverse_partition)(array, swap, array, &piv, swap_size, nmemb, cmp);
			s_size = nmemb - a_size;

			if (s_size <= a_size / 16 || a_size <= CRUM_OUT)
			{
				return FUNC(quadsort_swap)(array, swap, swap_size, a_size, cmp);
			}
			nmemb = a_size; max = NULL;
			continue;
		}
		*ptp = array[--nmemb];

		a_size = FUNC(fulcrum_default_partition)(array, swap, array, &piv, swap_size, nmemb, cmp);
		s_size = nmemb - a_size;

		ptp = array + a_size; array[nmemb] = *ptp; *ptp = piv;

		if (a_size <= s_size / 16 || s_size <= CRUM_OUT)
		{
			if (s_size == 0)
			{
				a_size = FUNC(fulcrum_reverse_partition)(array, swap, array, &piv, swap_size, a_size, cmp);
				s_size = nmemb - a_size;

				if (s_size <= a_size / 16 || a_size <= CRUM_OUT)
				{
					return FUNC(quadsort_swap)(array, swap, swap_size, a_size, cmp);
				}
				max = NULL;
				nmemb = a_size;
				continue;
			}
			FUNC(quadsort_swap)(ptp + 1, swap, swap_size, s_size, cmp);
		}
		else
		{
			FUNC(fulcrum_partition)(ptp + 1, swap, max, swap_size, s_size, cmp);
		}

		if (s_size <= a_size / 32 || a_size <= CRUM_OUT)
		{
			return FUNC(quadsort_swap)(array, swap, swap_size, a_size, cmp);
		}
		max = ptp;
		nmemb = a_size;
	}
}

void FUNC(crumsort)(VAR *array, size_t nmemb, CMPFUNC *cmp)
{
	if (nmemb < 32)
	{
		return FUNC(tail_swap)(array, nmemb, cmp);
	}
#if CRUM_AUX
	size_t swap_size = CRUM_AUX;
#else
	size_t swap_size = 32;

	while (swap_size * swap_size <= nmemb)
	{
		swap_size *= 4;
	}
#endif
	VAR swap[swap_size];

	FUNC(crum_analyze)(array, swap, swap_size, nmemb, cmp);
}

void FUNC(crumsort_swap)(VAR *array, VAR *swap, size_t swap_size, size_t nmemb, CMPFUNC *cmp)
{
	if (nmemb < 32)
	{
		FUNC(tail_swap)(array, nmemb, cmp);
	}
	else
	{
		FUNC(crum_analyze)(array, swap, swap_size, nmemb, cmp);
	}
}
