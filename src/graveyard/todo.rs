/*
void FUNC(forward_merge)(VAR *dest, VAR *from, size_t block, CMPFUNC *cmp)
{
    VAR *ptl, *ptr, *m, *e; // left, right, middle, end
    size_t x, y;

    ptl = from;
    ptr = from + block;
    m = ptr - 1;
    e = ptr + block - 1;

    if (cmp(m, e - block / 4) <= 0)
    {
        while (ptl < m - 1)
        {
            if (cmp(ptl + 1, ptr) <= 0)
            {
                *dest++ = *ptl++; *dest++ = *ptl++;
            }
            else if (cmp(ptl, ptr + 1) > 0)
            {
                *dest++ = *ptr++; *dest++ = *ptr++;
            }
            else
            {
                x = cmp(ptl, ptr) <= 0; y = !x; dest[x] = *ptr; ptr += 1; dest[y] = *ptl; ptl += 1; dest += 2;
                x = cmp(ptl, ptr) <= 0; y = !x; dest[x] = *ptr; ptr += y; dest[y] = *ptl; ptl += x; dest++;
            }
        }

        while (ptl <= m)
        {
            *dest++ = cmp(ptl, ptr) <= 0 ? *ptl++ : *ptr++;
        }

        do *dest++ = *ptr++; while (ptr <= e);
    }
    else if (cmp(m - block / 4, e) > 0)
    {
        while (ptr < e - 1)
        {
            if (cmp(ptl, ptr + 1) > 0)
            {
                *dest++ = *ptr++; *dest++ = *ptr++;
            }
            else if (cmp(ptl + 1, ptr) <= 0)
            {
                *dest++ = *ptl++; *dest++ = *ptl++;
            }
            else
            {
                x = cmp(ptl, ptr) <= 0; y = !x; dest[x] = *ptr; ptr += 1; dest[y] = *ptl; ptl += 1; dest += 2;
                x = cmp(ptl, ptr) <= 0; y = !x; dest[x] = *ptr; ptr += y; dest[y] = *ptl; ptl += x; dest++;
            }
        }

        while (ptr <= e)
        {
            *dest++ = cmp(ptl, ptr) > 0 ? *ptr++ : *ptl++;
        }

        do *dest++ = *ptl++; while (ptl <= m);
    }
    else
    {
        FUNC(parity_merge)(dest, from, block, block * 2, cmp);
    }
}

// main memory: [A][B][C][D]
// swap memory: [A  B]       step 1
// swap memory: [A  B][C  D] step 2
// main memory: [A  B  C  D] step 3

void FUNC(quad_merge_block)(VAR *array, VAR *swap, size_t block, CMPFUNC *cmp)
{
    register VAR *pts, *c, *c_max;
    size_t block_x_2 = block * 2;

    c_max = array + block;

    if (cmp(c_max - 1, c_max) <= 0)
    {
        c_max += block_x_2;

        if (cmp(c_max - 1, c_max) <= 0)
        {
            c_max -= block;

            if (cmp(c_max - 1, c_max) <= 0)
            {
                return;
            }
            pts = swap;

            c = array;

            do *pts++ = *c++; while (c < c_max); // step 1

            c_max = c + block_x_2;

            do *pts++ = *c++; while (c < c_max); // step 2

            return FUNC(forward_merge)(array, swap, block_x_2, cmp); // step 3
        }
        pts = swap;

        c = array;
        c_max = array + block_x_2;

        do *pts++ = *c++; while (c < c_max); // step 1
    }
    else
    {
        FUNC(forward_merge)(swap, array, block, cmp); // step 1
    }
    FUNC(forward_merge)(swap + block_x_2, array + block_x_2, block, cmp); // step 2

    FUNC(forward_merge)(array, swap, block_x_2, cmp); // step 3
}

void FUNC(tail_merge)(VAR *array, VAR *swap, size_t swap_size, size_t nmemb, size_t block, CMPFUNC *cmp)
{
    register VAR *pta, *pte;

    pte = array + nmemb;

    while (block < nmemb && block <= swap_size)
    {
        for (pta = array ; pta + block < pte ; pta += block * 2)
        {
            if (pta + block * 2 < pte)
            {
                FUNC(partial_backward_merge)(pta, swap, block * 2, block, cmp);

                continue;
            }
            FUNC(partial_backward_merge)(pta, swap, pte - pta, block, cmp);

            break;
        }
        block *= 2;
    }
}

void FUNC(partial_backward_merge)(VAR *array, VAR *swap, size_t nmemb, size_t block, CMPFUNC *cmp)
{
    VAR *m, *e, *s; // middle, end, swap
    size_t x, y;

    m = array + block - 1;
    e = array + nmemb - 1;

    if (cmp(m, m + 1) <= 0)
    {
        return;
    }

    memcpy(swap, array + block, (nmemb - block) * sizeof(VAR));

    s = swap + nmemb - block - 1;

    while (s > swap + 1 && m > array + 1)
    {
        if (cmp(m - 1, s) > 0)
        {
            *e-- = *m--;
            *e-- = *m--;
        }
        else if (cmp(m, s - 1) <= 0)
        {
            *e-- = *s--;
            *e-- = *s--;
        }
        else
        {
            x = cmp(m, s) <= 0; y = !x; e--; e[x] = *s; s -= 1; e[y] = *m; m -= 1; e--;
            x = cmp(m, s) <= 0; y = !x; e--; e[x] = *s; s -= x; e[y] = *m; m -= y;
        }
    }

    while (s >= swap && m >= array)
    {
        *e-- = cmp(m, s) > 0 ? *m-- : *s--;
    }

    while (s >= swap)
    {
        *e-- = *s--;
    }
}

size_t FUNC(quad_merge)(VAR *array, VAR *swap, size_t swap_size, size_t nmemb, size_t block, CMPFUNC *cmp)
{
    register VAR *pta, *pte;

    pte = array + nmemb;

    block *= 4;

    while (block <= nmemb && block <= swap_size)
    {
        pta = array;

        do
        {
            FUNC(quad_merge_block)(pta, swap, block / 4, cmp);

            pta += block;
        }
        while (pta + block <= pte);

        FUNC(tail_merge)(pta, swap, swap_size, pte - pta, block / 4, cmp);

        block *= 4;
    }

    FUNC(tail_merge)(array, swap, swap_size, nmemb, block / 4, cmp);

    return block / 2;
}

*/
