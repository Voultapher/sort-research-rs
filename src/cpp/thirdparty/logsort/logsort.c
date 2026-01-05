/*
 *
MIT License

Copyright (c) 2022-2024 aphitorite

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
 *
 */

#include <stdlib.h>
#include <string.h>

#define MIN_SMALLSORT 7
#define MIN_PIPOSORT 512

char FUNC(log_ceil_log)(size_t n) {
    char r = 0;
    while ((1 << r) < n)
        r++;
    return r;
}

////////////////
//            //
//  PIPOSORT  //
//            //
////////////////

// courtesy of @scandum's piposort

void FUNC(log_smallsort)(VAR* array, size_t nmemb) {
    VAR swap, *pta, *pte;
    unsigned char w = 1, x, y, z = 1;

    switch (nmemb) {
    default:
        pte = array + nmemb - 3;

        do {
            pta = pte + (z = !z);

            do {
                x = CMP(pta, pta + 1) > 0;
                y = !x;
                swap = pta[y];
                pta[0] = pta[x];
                pta[1] = swap;
                pta -= 2;
                w |= x;
            } while (pta >= array);
        } while (w-- && --nmemb);
        return;

    case 3:
        pta = array;

        x = CMP(pta, pta + 1) > 0;
        y = !x;
        swap = pta[y];
        pta[0] = pta[x];
        pta[1] = swap;
        pta++;

        x = CMP(pta, pta + 1) > 0;
        y = !x;
        swap = pta[y];
        pta[0] = pta[x];
        pta[1] = swap;

        if (x == 0)
            return;

    case 2:
        pta = array;

        x = CMP(pta, pta + 1) > 0;
        y = !x;
        swap = pta[y];
        pta[0] = pta[x];
        pta[1] = swap;

    case 1:
    case 0:
        return;
    }
}

void FUNC(log_parity_merge)(VAR* from, VAR* dest, size_t left, size_t right) {
    VAR *ptl, *ptr, *tpl, *tpr, *tpd, *ptd;
    unsigned char x;

    ptl = from;
    ptr = from + left;
    ptd = dest;
    tpl = from + left - 1;
    tpr = from + left + right - 1;
    tpd = dest + left + right - 1;

    if (left < right)
        *ptd++ = CMP(ptl, ptr) <= 0 ? *ptl++ : *ptr++;

    while (--left) {
        x = CMP(ptl, ptr) <= 0;
        *ptd = *ptl;
        ptl += x;
        ptd[x] = *ptr;
        ptr += !x;
        ptd++;
        x = CMP(tpl, tpr) <= 0;
        *tpd = *tpl;
        tpl -= !x;
        tpd--;
        tpd[x] = *tpr;
        tpr -= x;
    }
    *tpd = CMP(tpl, tpr) > 0 ? *tpl : *tpr;
    *ptd = CMP(ptl, ptr) <= 0 ? *ptl : *ptr;
}
void FUNC(log_piposort)(VAR* array, VAR* swap, size_t n) {
    size_t q1, q2, q3, q4, h1, h2;

    if (n <= MIN_SMALLSORT) {
        FUNC(log_smallsort)(array, n);
        return;
    }
    h1 = n / 2;
    q1 = h1 / 2;
    q2 = h1 - q1;
    h2 = n - h1;
    q3 = h2 / 2;
    q4 = h2 - q3;

    FUNC(log_piposort)(array, swap, q1);
    FUNC(log_piposort)(array + q1, swap, q2);
    FUNC(log_piposort)(array + h1, swap, q3);
    FUNC(log_piposort)(array + h1 + q3, swap, q4);

    if (CMP(array + q1 - 1, array + q1) <= 0 && CMP(array + h1 - 1, array + h1) <= 0 &&
        CMP(array + h1 + q3 - 1, array + h1 + q3) <= 0)
        return;

    FUNC(log_parity_merge)(array, swap, q1, q2);
    FUNC(log_parity_merge)(array + h1, swap + h1, q3, q4);
    FUNC(log_parity_merge)(swap, array, h1, h2);
}

///////////////////////
//                   //
//  PIVOT SELECTION  //
//                   //
///////////////////////

// courtesy of @scandum's blitsort

void FUNC(log_trim_four)(VAR* pta) {
    VAR swap;
    size_t x;

    x = CMP(pta, pta + 1) > 0;
    swap = pta[!x];
    pta[0] = pta[x];
    pta[1] = swap;
    pta += 2;
    x = CMP(pta, pta + 1) > 0;
    swap = pta[!x];
    pta[0] = pta[x];
    pta[1] = swap;
    pta -= 2;

    x = (CMP(pta, pta + 2) <= 0) * 2;
    pta[2] = pta[x];
    pta++;
    x = (CMP(pta, pta + 2) > 0) * 2;
    pta[0] = pta[x];
}

VAR FUNC(log_median_of_nine)(VAR* a, VAR* s, size_t n) {
    size_t step = (n - 1) / 8, i;
    VAR* pa = a;

    for (i = 0; i < 9; i++) {
        s[i] = *pa;
        pa += step;
    }

    FUNC(log_smallsort)(s, 9);
    return s[4];
}

VAR FUNC(log_smart_median)(VAR* array, VAR* swap, size_t n, size_t bLen) {
    if (bLen < 64)
        return FUNC(log_median_of_nine)(array, swap, n);

    size_t cbrt;
    for (cbrt = 32; cbrt * cbrt * cbrt < n && cbrt < 1024; cbrt *= 2) {
    }

    size_t div = bLen < cbrt ? bLen : cbrt;
    size_t step = n / div, c;
    VAR *i = array, *j;

    // copy sample to swap space

    for (c = 0; c < div; c++) {
        swap[c] = *i;
        i += step;
    }

    // halve the sample using trim fours

    div /= 2;
    i = swap;
    j = swap + div;

    for (c = (div /= 4); c; c--) {
        FUNC(log_trim_four)(i);
        FUNC(log_trim_four)(j);

        i[0] = j[1];
        i[3] = j[2];
        i += 4;
        j += 4;
    }

    // sort sample for median

    div *= 4;
    FUNC(log_piposort)(swap, swap + div, div);

    return swap[div / 2 + 1];
}

///////////////
//           //
//  LOGSORT  //
//           //
///////////////

void FUNC(log_block_xor)(VAR* a, VAR* b, size_t v) {
    VAR t;

    while (v) {
        if (v & 1) {
            t = *a;
            *a = *b;
            *b = t;
        }
        v >>= 1;
        a++;
        b++;
    }
}

#define PIVFUNC(NAME) FUNC(NAME##_less)
#define PIVCMP(a, b) (CMP((b), (a)) > 0)

#include "logPartition.c"

#undef PIVFUNC
#undef PIVCMP

#define PIVFUNC(NAME) FUNC(NAME##_less_eq)
#define PIVCMP(a, b) (CMP((a), (b)) <= 0)

#include "logPartition.c"

#undef PIVFUNC
#undef PIVCMP

// logsort sorting functions

void FUNC(logsort_rec)(VAR* a, VAR* s, size_t n, size_t bLen) {
    size_t minSort = bLen < MIN_PIPOSORT ? bLen : MIN_PIPOSORT;

    while (n > minSort) {
        VAR piv =
            n < 2048 ? FUNC(log_median_of_nine)(a, s, n) : FUNC(log_smart_median)(a, s, n, bLen);

        VAR* p = FUNC(log_partition_less_eq)(a, s, n, bLen, &piv);
        size_t m = p - a;

        if (m == n) {  // in the case of many equal elements
            p = FUNC(log_partition_less)(a, s, n, bLen, &piv);
            n = p - a;

            continue;
        }
        FUNC(logsort_rec)(p, s, n - m, bLen);
        n = m;
    }
    FUNC(log_piposort)(a, s, n);
}
void FUNC(logsort)(VAR* a, size_t n, size_t bLen) {
    if (n < bLen)
        bLen = n;
    if (bLen < 9)
        bLen = 9;  // for median of nine

    VAR* s = (VAR*)(malloc(bLen * sizeof(VAR)));
    FUNC(logsort_rec)(a, s, n, bLen);
    free(s);
}

#undef MIN_SMALLSORT
#undef MIN_PIPOSORT
