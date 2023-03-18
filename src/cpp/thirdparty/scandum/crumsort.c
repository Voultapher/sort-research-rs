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
	crumsort 1.1.5.4
*/

#define CRUM_AUX 512
#define CRUM_OUT  24

void FUNC(fulcrum_partition)(VAR *array, VAR *swap, VAR *max, size_t swap_size, size_t nmemb, CMPFUNC *cmp);

void FUNC(crum_analyze)(VAR *array, VAR *swap, size_t swap_size, size_t nmemb, CMPFUNC *cmp)
{
	unsigned char loop, asum, bsum, csum, dsum;
	unsigned int astreaks, bstreaks, cstreaks, dstreaks;
	size_t quad1, quad2, quad3, quad4, half1, half2;
	size_t cnt, abalance, bbalance, cbalance, dbalance;
	VAR *pta, *ptb, *ptc, *ptd;

	half1 = nmemb / 2;
	quad1 = half1 / 2;
	quad2 = half1 - quad1;
	half2 = nmemb - half1;
	quad3 = half2 / 2;
	quad4 = half2 - quad3;

	pta = array;
	ptb = array + quad1;
	ptc = array + half1;
	ptd = array + half1 + quad3;

	astreaks = bstreaks = cstreaks = dstreaks = 0;
	abalance = bbalance = cbalance = dbalance = 0;

	for (cnt = nmemb ; cnt > 132 ; cnt -= 128)
	{
		for (asum = bsum = csum = dsum = 0, loop = 32 ; loop ; loop--)
		{
			asum += cmp(pta, pta + 1) > 0; pta++;
			bsum += cmp(ptb, ptb + 1) > 0; ptb++;
			csum += cmp(ptc, ptc + 1) > 0; ptc++;
			dsum += cmp(ptd, ptd + 1) > 0; ptd++;
		}
		abalance += asum; astreaks += asum = (asum == 0) | (asum == 32);
		bbalance += bsum; bstreaks += bsum = (bsum == 0) | (bsum == 32);
		cbalance += csum; cstreaks += csum = (csum == 0) | (csum == 32);
		dbalance += dsum; dstreaks += dsum = (dsum == 0) | (dsum == 32);

		if (cnt > 516 && asum + bsum + csum + dsum == 0)
		{
			abalance += 48; pta += 96;
			bbalance += 48; ptb += 96;
			cbalance += 48; ptc += 96;
			dbalance += 48; ptd += 96;
			cnt -= 384;
		}
	}

	for ( ; cnt > 7 ; cnt -= 4)
	{
		abalance += cmp(pta, pta + 1) > 0; pta++;
		bbalance += cmp(ptb, ptb + 1) > 0; ptb++;
		cbalance += cmp(ptc, ptc + 1) > 0; ptc++;
		dbalance += cmp(ptd, ptd + 1) > 0; ptd++;
	}

	if (quad1 < quad2) {bbalance += cmp(ptb, ptb + 1) > 0; ptb++;}
	if (quad1 < quad3) {cbalance += cmp(ptc, ptc + 1) > 0; ptc++;}
	if (quad1 < quad4) {dbalance += cmp(ptd, ptd + 1) > 0; ptd++;}

	cnt = abalance + bbalance + cbalance + dbalance;

	if (cnt == 0)
	{
		if (cmp(pta, pta + 1) <= 0 && cmp(ptb, ptb + 1) <= 0 && cmp(ptc, ptc + 1) <= 0)
		{
			return;
		}
	}

	asum = quad1 - abalance == 1;
	bsum = quad2 - bbalance == 1;
	csum = quad3 - cbalance == 1;
	dsum = quad4 - dbalance == 1;

	if (asum | bsum | csum | dsum)
	{
		unsigned char span1 = (asum && bsum) * (cmp(pta, pta + 1) > 0);
		unsigned char span2 = (bsum && csum) * (cmp(ptb, ptb + 1) > 0);
		unsigned char span3 = (csum && dsum) * (cmp(ptc, ptc + 1) > 0);

		switch (span1 | span2 * 2 | span3 * 4)
		{
			case 0: break;
			case 1: FUNC(quad_reversal)(array, ptb);   abalance = bbalance = 0; break;
			case 2: FUNC(quad_reversal)(pta + 1, ptc); bbalance = cbalance = 0; break;
			case 3: FUNC(quad_reversal)(array, ptc);   abalance = bbalance = cbalance = 0; break;
			case 4: FUNC(quad_reversal)(ptb + 1, ptd); cbalance = dbalance = 0; break;
			case 5: FUNC(quad_reversal)(array, ptb);
				FUNC(quad_reversal)(ptb + 1, ptd); abalance = bbalance = cbalance = dbalance = 0; break;
			case 6: FUNC(quad_reversal)(pta + 1, ptd); bbalance = cbalance = dbalance = 0; break;
			case 7: FUNC(quad_reversal)(array, ptd); return;
		}

		if (asum && abalance) {FUNC(quad_reversal)(array,   pta); abalance = 0;}
		if (bsum && bbalance) {FUNC(quad_reversal)(pta + 1, ptb); bbalance = 0;}
		if (csum && cbalance) {FUNC(quad_reversal)(ptb + 1, ptc); cbalance = 0;}
		if (dsum && dbalance) {FUNC(quad_reversal)(ptc + 1, ptd); dbalance = 0;}
	}

#ifdef cmp
	cnt = nmemb / 256; // switch to quadsort if at least 50% ordered
#else
	cnt = nmemb / 512; // switch to quadsort if at least 25% ordered
#endif
	asum = astreaks > cnt;
	bsum = bstreaks > cnt;
	csum = cstreaks > cnt;
	dsum = dstreaks > cnt;

#ifndef cmp
	if (quad1 > QUAD_CACHE)
	{
		asum = bsum = csum = dsum = 1;
	}
#endif
	switch (asum + bsum * 2 + csum * 4 + dsum * 8)
	{
		case 0:
			FUNC(fulcrum_partition)(array, swap, NULL, swap_size, nmemb, cmp);
			return;
		case 1:
			if (abalance) FUNC(quadsort_swap)(array, swap, swap_size, quad1, cmp);
			FUNC(fulcrum_partition)(pta + 1, swap, NULL, swap_size, quad2 + half2, cmp);
			break;
		case 2:
			FUNC(fulcrum_partition)(array, swap, NULL, swap_size, quad1, cmp);
			if (bbalance) FUNC(quadsort_swap)(pta + 1, swap, swap_size, quad2, cmp);
			FUNC(fulcrum_partition)(ptb + 1, swap, NULL, swap_size, half2, cmp);
			break;
		case 3:
			if (abalance) FUNC(quadsort_swap)(array, swap, swap_size, quad1, cmp);
			if (bbalance) FUNC(quadsort_swap)(pta + 1, swap, swap_size, quad2, cmp);
			FUNC(fulcrum_partition)(ptb + 1, swap, NULL, swap_size, half2, cmp);
			break;
		case 4:
			FUNC(fulcrum_partition)(array, swap, NULL, swap_size, half1, cmp);
			if (cbalance) FUNC(quadsort_swap)(ptb + 1, swap, swap_size, quad3, cmp);
			FUNC(fulcrum_partition)(ptc + 1, swap, NULL, swap_size, quad4, cmp);
			break;
		case 8:
			FUNC(fulcrum_partition)(array, swap, NULL, swap_size, half1 + quad3, cmp);
			if (dbalance) FUNC(quadsort_swap)(ptc + 1, swap, swap_size, quad4, cmp);
			break;
		case 9:
			if (abalance) FUNC(quadsort_swap)(array, swap, swap_size, quad1, cmp);
			FUNC(fulcrum_partition)(pta + 1, swap, NULL, swap_size, quad2 + quad3, cmp);
			if (dbalance) FUNC(quadsort_swap)(ptc + 1, swap, swap_size, quad4, cmp);
			break;
		case 12:
			FUNC(fulcrum_partition)(array, swap, NULL, swap_size, half1, cmp);
			if (cbalance) FUNC(quadsort_swap)(ptb + 1, swap, swap_size, quad3, cmp);
			if (dbalance) FUNC(quadsort_swap)(ptc + 1, swap, swap_size, quad4, cmp);
			break;
		case 5:
		case 6:
		case 7:
		case 10:
		case 11:
		case 13:
		case 14:
		case 15:
			if (asum)
			{
				if (abalance) FUNC(quadsort_swap)(array, swap, swap_size, quad1, cmp);
			}
			else FUNC(fulcrum_partition)(array, swap, NULL, swap_size, quad1, cmp);
			if (bsum)
			{
				if (bbalance) FUNC(quadsort_swap)(pta + 1, swap, swap_size, quad2, cmp);
			}
			else FUNC(fulcrum_partition)(pta + 1, swap, NULL, swap_size, quad2, cmp);
			if (csum)
			{
				if (cbalance) FUNC(quadsort_swap)(ptb + 1, swap, swap_size, quad3, cmp);
			}
			else FUNC(fulcrum_partition)(ptb + 1, swap, NULL, swap_size, quad3, cmp);
			if (dsum)
			{
				if (dbalance) FUNC(quadsort_swap)(ptc + 1, swap, swap_size, quad4, cmp);
			}
			else FUNC(fulcrum_partition)(ptc + 1, swap, NULL, swap_size, quad4, cmp);
			break;
	}

	if (cmp(pta, pta + 1) <= 0)
	{
		if (cmp(ptc, ptc + 1) <= 0)
		{
			if (cmp(ptb, ptb + 1) <= 0)
			{
				return;
			}
		}
		else
		{
			FUNC(blit_merge_block)(array + half1, swap, swap_size, quad3, quad4, cmp);
		}
	}
	else
	{
		FUNC(blit_merge_block)(array, swap, swap_size, quad1, quad2, cmp);

		if (cmp(ptc, ptc + 1) > 0)
		{
			FUNC(blit_merge_block)(array + half1, swap, swap_size, quad3, quad4, cmp);
		}
	}
	FUNC(blit_merge_block)(array, swap, swap_size, half1, half2, cmp);
}

// The next 3 functions are used for pivot selection

VAR *FUNC(crum_median_of_sqrt)(VAR *array, VAR *swap, size_t swap_size, size_t nmemb, CMPFUNC *cmp)
{
	VAR *pta, *piv;
	size_t cnt, sqrt, div;

	sqrt = nmemb < 65536 ? 32 : nmemb < 262144 ? 128 : nmemb < 16777216 ? 256 : 512;

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

size_t FUNC(crum_median_of_five)(VAR *array, size_t v0, size_t v1, size_t v2, size_t v3, size_t v4, CMPFUNC *cmp)
{
	VAR *swap[6], **pta;
	size_t x, y, z;

	swap[2] = &array[v0];
	swap[3] = &array[v1];
	swap[4] = &array[v2];
	swap[5] = &array[v3];

	pta = swap + 2;

	x = cmp(pta[0], pta[1]) > 0; y = !x; swap[0] = pta[y]; pta[0] = pta[x]; pta[1] = swap[0]; pta += 2;
	x = cmp(pta[0], pta[1]) > 0; y = !x; swap[0] = pta[y]; pta[0] = pta[x]; pta[1] = swap[0]; pta -= 2;
	x = cmp(pta[0], pta[2]) > 0; y = !x; swap[0] = pta[0]; swap[1] = pta[2]; pta[0] = swap[x]; pta[2] = swap[y]; pta++;
	x = cmp(pta[0], pta[2]) > 0; y = !x; swap[0] = pta[0]; swap[1] = pta[2]; pta[0] = swap[x]; pta[2] = swap[y];

	pta[2] = &array[v4];

	x = cmp(pta[0], pta[1]) > 0;
	y = cmp(pta[0], pta[2]) > 0;
	z = cmp(pta[1], pta[2]) > 0;

	return pta[(x == y) + (y ^ z)] - array;
}

VAR *FUNC(crum_median_of_twentyfive)(VAR *array, size_t nmemb, CMPFUNC *cmp)
{
	size_t swap[5];
	size_t div = nmemb / 64;

	swap[0] = FUNC(crum_median_of_five)(array, div *  4, div *  1, div *  2, div *  8, div * 10, cmp);
	swap[1] = FUNC(crum_median_of_five)(array, div * 16, div * 12, div * 14, div * 18, div * 20, cmp);
	swap[2] = FUNC(crum_median_of_five)(array, div * 32, div * 24, div * 30, div * 34, div * 38, cmp);
	swap[3] = FUNC(crum_median_of_five)(array, div * 48, div * 42, div * 44, div * 50, div * 52, cmp);
	swap[4] = FUNC(crum_median_of_five)(array, div * 60, div * 54, div * 56, div * 62, div * 63, cmp);

	return array + FUNC(crum_median_of_five)(array, swap[0], swap[1], swap[2], swap[3], swap[4], cmp);
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

// As per suggestion by Marshall Lochbaum to improve generic data handling by mimicking dual-pivot quicksort

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
		else if (nmemb <= 32768)
		{
			ptp = FUNC(crum_median_of_twentyfive)(array, nmemb, cmp);
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
	if (nmemb <= 132)
	{
		return FUNC(quadsort)(array, nmemb, cmp);
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
	if (nmemb <= 132)
	{
		FUNC(quadsort_swap)(array, swap, swap_size, nmemb, cmp);
	}
	else
	{
		FUNC(crum_analyze)(array, swap, swap_size, nmemb, cmp);
	}
}