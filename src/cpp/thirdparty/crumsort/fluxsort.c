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
	fluxsort 1.1.5.3
*/

#define FLUX_OUT 24

void FUNC(flux_partition)(VAR *array, VAR *swap, VAR *ptx, VAR *ptp, size_t nmemb, CMPFUNC *cmp);

// Determine whether to use mergesort or quicksort

void FUNC(flux_analyze)(VAR *array, VAR *swap, size_t swap_size, size_t nmemb, CMPFUNC *cmp)
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
		if (nmemb >= 1024)
		{
			size_t block = pta - array;

			if (astreaks < nmemb / 128)
			{
				FUNC(flux_partition)(array, swap, array, swap + block, block, cmp);
			}
			else if (abalance)
			{
				FUNC(quadsort_swap)(array, swap, swap_size, block, cmp);
			}

			if (zstreaks < nmemb / 128)
			{
				FUNC(flux_partition)(array + block, swap, array + block, swap + nmemb - block, nmemb - block, cmp);
			}
			else if (zbalance)
			{
				FUNC(quadsort_swap)(array + block, swap, swap_size, nmemb - block, cmp);
			}
			FUNC(partial_forward_merge)(array, swap, nmemb, block, cmp);
		}
		else
		{
			FUNC(quadsort_swap)(array, swap, swap_size, nmemb, cmp);
		}
		return;
	}
	FUNC(flux_partition)(array, swap, array, swap + nmemb, nmemb, cmp);
}

// The next 5 functions are used for pivot selection

VAR FUNC(median_of_sqrt)(VAR *array, VAR *swap, VAR *ptx, size_t nmemb, CMPFUNC *cmp)
{
	VAR *pta, *pts;
	size_t cnt, sqrt, div;

	sqrt = nmemb > 262144 ? 256 : 128;

	div = nmemb / sqrt;

	pta = ptx + rand() % sqrt;

	pts = ptx == array ? swap : array;

	for (cnt = 0 ; cnt < sqrt ; cnt++)
	{
		pts[cnt] = pta[0];

		pta += div;
	}
	FUNC(quadsort_swap)(pts, pts + sqrt, sqrt, sqrt, cmp);

	return pts[sqrt / 2];
}

VAR FUNC(median_of_five)(VAR *array, size_t v0, size_t v1, size_t v2, size_t v3, size_t v4, CMPFUNC *cmp)
{
	VAR swap[6], *pta;
	size_t x, y, z;

	swap[2] = array[v0];
	swap[3] = array[v1];
	swap[4] = array[v2];
	swap[5] = array[v3];

	pta = swap + 2;

	x = cmp(pta, pta + 1) > 0; y = !x; swap[0] = pta[y]; pta[0] = pta[x]; pta[1] = swap[0]; pta += 2;
	x = cmp(pta, pta + 1) > 0; y = !x; swap[0] = pta[y]; pta[0] = pta[x]; pta[1] = swap[0]; pta -= 2;
	x = cmp(pta, pta + 2) > 0; y = !x; swap[0] = pta[0]; swap[1] = pta[2]; pta[0] = swap[x]; pta[2] = swap[y]; pta++;
	x = cmp(pta, pta + 2) > 0; y = !x; swap[0] = pta[0]; swap[1] = pta[2]; pta[0] = swap[x]; pta[2] = swap[y];

	pta[2] = array[v4];

	x = cmp(pta, pta + 1) > 0;
	y = cmp(pta, pta + 2) > 0;
	z = cmp(pta + 1, pta + 2) > 0;

	return pta[(x == y) + (y ^ z)];
}

VAR FUNC(median_of_twentyfive)(VAR *array, size_t nmemb, CMPFUNC *cmp)
{
	VAR swap[5];
	size_t div = nmemb / 64;

	swap[0] = FUNC(median_of_five)(array, div *  4, div *  1, div *  2, div *  8, div * 10, cmp);
	swap[1] = FUNC(median_of_five)(array, div * 16, div * 12, div * 14, div * 18, div * 20, cmp);
	swap[2] = FUNC(median_of_five)(array, div * 32, div * 24, div * 30, div * 34, div * 38, cmp);
	swap[3] = FUNC(median_of_five)(array, div * 48, div * 42, div * 44, div * 50, div * 52, cmp);
	swap[4] = FUNC(median_of_five)(array, div * 60, div * 54, div * 56, div * 62, div * 63, cmp);

	return FUNC(median_of_five)(swap, 0, 1, 2, 3, 4, cmp);
}

size_t FUNC(median_of_three)(VAR *array, size_t v0, size_t v1, size_t v2, CMPFUNC *cmp)
{
	size_t v[3] = {v0, v1, v2};
	char x, y, z;

	x = cmp(array + v0, array + v1) > 0;
	y = cmp(array + v0, array + v2) > 0;
	z = cmp(array + v1, array + v2) > 0;

	return v[(x == y) + (y ^ z)];
}

VAR FUNC(median_of_nine)(VAR *array, size_t nmemb, CMPFUNC *cmp)
{
	size_t x, y, z, div = nmemb / 16;

	x = FUNC(median_of_three)(array, div * 2, div * 1, div * 4, cmp);
	y = FUNC(median_of_three)(array, div * 8, div * 6, div * 10, cmp);
	z = FUNC(median_of_three)(array, div * 14, div * 12, div * 15, cmp);

	return array[FUNC(median_of_three)(array, x, y, z, cmp)];
}

// As per suggestion by Marshall Lochbaum to improve generic data handling

void FUNC(flux_reverse_partition)(VAR *array, VAR *swap, VAR *ptx, VAR *piv, size_t nmemb, CMPFUNC *cmp)
{
	size_t a_size, s_size;

	{
		size_t cnt, val, m;
		VAR *pts = swap;

		for (m = 0, cnt = nmemb / 8 ; cnt ; cnt--)
		{
			val = cmp(piv, ptx) > 0; pts[-m] = array[m] = *ptx++; m += val; pts++;
			val = cmp(piv, ptx) > 0; pts[-m] = array[m] = *ptx++; m += val; pts++;
			val = cmp(piv, ptx) > 0; pts[-m] = array[m] = *ptx++; m += val; pts++;
			val = cmp(piv, ptx) > 0; pts[-m] = array[m] = *ptx++; m += val; pts++;
			val = cmp(piv, ptx) > 0; pts[-m] = array[m] = *ptx++; m += val; pts++;
			val = cmp(piv, ptx) > 0; pts[-m] = array[m] = *ptx++; m += val; pts++;
			val = cmp(piv, ptx) > 0; pts[-m] = array[m] = *ptx++; m += val; pts++;
			val = cmp(piv, ptx) > 0; pts[-m] = array[m] = *ptx++; m += val; pts++;
		}

		for (cnt = nmemb % 8 ; cnt ; cnt--)
		{
			val = cmp(piv, ptx) > 0; pts[-m] = array[m] = *ptx++; m += val; pts++;
		}
		a_size = m;
		s_size = nmemb - a_size;
	}
	memcpy(array + a_size, swap, s_size * sizeof(VAR));

	if (s_size <= a_size / 16 || a_size <= FLUX_OUT)
	{
		return FUNC(quadsort_swap)(array, swap, a_size, a_size, cmp);
	}
	FUNC(flux_partition)(array, swap, array, piv, a_size, cmp);
}

size_t FUNC(flux_default_partition)(VAR *array, VAR *swap, VAR *ptx, VAR *piv, size_t nmemb, CMPFUNC *cmp)
{
	size_t run = 0, val, a = 0, m = 0;

	for (a = 8 ; a <= nmemb ; a += 8)
	{
		val = cmp(ptx, piv) <= 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
		val = cmp(ptx, piv) <= 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
		val = cmp(ptx, piv) <= 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
		val = cmp(ptx, piv) <= 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
		val = cmp(ptx, piv) <= 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
		val = cmp(ptx, piv) <= 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
		val = cmp(ptx, piv) <= 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
		val = cmp(ptx, piv) <= 0; swap[-m] = array[m] = *ptx++; m += val; swap++;

		if (m == a) run = a;
	}

	for (a = nmemb % 8 ; a ; a--)
	{
		val = cmp(ptx, piv) <= 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
	}

	if (run < nmemb / 4)
	{
		return m;
	}

	if (m == nmemb)
	{
		return m;
	}
	swap -= nmemb;
	a = nmemb - m;

	memcpy(array + m, swap, a * sizeof(VAR));

	FUNC(quadsort_swap)(array + m, swap, a, a, cmp);
	FUNC(quadsort_swap)(array, swap, m, m, cmp);

	return 0;
}

void FUNC(flux_partition)(VAR *array, VAR *swap, VAR *ptx, VAR *piv, size_t nmemb, CMPFUNC *cmp)
{
	size_t a_size = 0, s_size;

	while (1)
	{
		--piv;

		if (nmemb <= 2048)
		{
			*piv = FUNC(median_of_nine)(ptx, nmemb, cmp);
		}
		else if (nmemb <= 65536)
		{
			*piv = FUNC(median_of_twentyfive)(ptx, nmemb, cmp);
		}
		else
		{
			*piv = FUNC(median_of_sqrt)(array, swap, ptx, nmemb, cmp);
		}

		if (a_size && cmp(piv + 1, piv) <= 0)
		{
			return FUNC(flux_reverse_partition)(array, swap, array, piv, nmemb, cmp);
		}
		a_size = FUNC(flux_default_partition)(array, swap, ptx, piv, nmemb, cmp);
		s_size = nmemb - a_size;

		if (a_size <= s_size / 16 || s_size <= FLUX_OUT)
		{
			if (a_size == 0)
			{
				return;
			}
			if (s_size == 0)
			{
				return FUNC(flux_reverse_partition)(array, swap, array, piv, a_size, cmp);
			}
			memcpy(array + a_size, swap, s_size * sizeof(VAR));
			FUNC(quadsort_swap)(array + a_size, swap, s_size, s_size, cmp);
		}
		else
		{
			FUNC(flux_partition)(array + a_size, swap, swap, piv, s_size, cmp);
		}

		if (s_size <= a_size / 16 || a_size <= FLUX_OUT)
		{
			return FUNC(quadsort_swap)(array, swap, a_size, a_size, cmp);
		}
		nmemb = a_size;
		ptx = array;
	}
}

void FUNC(fluxsort)(VAR *array, size_t nmemb, CMPFUNC *cmp)
{
	if (nmemb < 32)
	{
		FUNC(tail_swap)(array, nmemb, cmp);
	}
	else
	{
		VAR *swap = static_cast<VAR*>(malloc(nmemb * sizeof(VAR)));

		if (swap == NULL)
		{
			return FUNC(quadsort)(array, nmemb, cmp);
		}
		FUNC(flux_analyze)(array, swap, nmemb, nmemb, cmp);

		free(swap);
	}
}

void FUNC(fluxsort_swap)(VAR *array, VAR *swap, size_t swap_size, size_t nmemb, CMPFUNC *cmp)
{
	if (nmemb < 32)
	{
		FUNC(tail_swap)(array, nmemb, cmp);
	}
	else if (swap_size < nmemb)
	{
		FUNC(quadsort_swap)(array, swap, swap_size, nmemb, cmp);
	}
	else
	{
		FUNC(flux_analyze)(array, swap, swap_size, nmemb, cmp);
	}
}
