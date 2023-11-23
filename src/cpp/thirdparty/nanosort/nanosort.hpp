/**
 * nanosort
 *
 * Copyright (C) 2021, by Arseny Kapoulkine (arseny.kapoulkine@gmail.com)
 * Report bugs and download new versions at https://github.com/zeux/nanosort
 *
 * This library is distributed under the MIT License. See notice at the end of
 * this file.
 *
 * Thank you to Andrei Alexandrescu for his branchless Lomuto partition code and
 * Gerben Stavenga for further research of branchless partitions; their work
 * inspired this algorithm.
 */
#pragma once

#include <assert.h>
#include <stddef.h>

#ifdef _MSC_VER
#define NANOSORT_NOINLINE __declspec(noinline)
#define NANOSORT_UNLIKELY(c) (c)
#else
#define NANOSORT_NOINLINE __attribute__((noinline))
#define NANOSORT_UNLIKELY(c) __builtin_expect(c, 0)
#endif

#if __cplusplus >= 201103L
#define NANOSORT_MOVE(v) static_cast<decltype(v)&&>(v)
#else
#define NANOSORT_MOVE(v) v
#endif

namespace nanosort_detail {

struct Less {
  template <typename T>
  bool operator()(const T& l, const T& r) const {
    return l < r;
  }
};

template <typename It>
struct IteratorTraits {
  typedef typename It::value_type value_type;
};

template <typename T>
struct IteratorTraits<T*> {
  typedef T value_type;
};

template <typename T>
void swap(T& l, T& r) {
  T t(NANOSORT_MOVE(l));
  l = NANOSORT_MOVE(r);
  r = NANOSORT_MOVE(t);
}

// Return median of 5 elements in the array
template <typename T, typename It, typename Compare>
T median5(It first, It last, Compare comp) {
  size_t n = last - first;
  assert(n >= 5);

  T e0 = first[(n >> 2) * 0];
  T e1 = first[(n >> 2) * 1];
  T e2 = first[(n >> 2) * 2];
  T e3 = first[(n >> 2) * 3];
  T e4 = first[n - 1];

  if (comp(e1, e0)) swap(e1, e0);
  if (comp(e4, e3)) swap(e4, e3);
  if (comp(e3, e0)) swap(e3, e0);

  if (comp(e1, e4)) swap(e1, e4);
  if (comp(e2, e1)) swap(e2, e1);
  if (comp(e3, e2)) swap(e2, e3);

  if (comp(e2, e1)) swap(e2, e1);

  return e2;
}

// Split array into x<pivot and x>=pivot
template <typename T, typename It, typename Compare>
It partition(T pivot, It first, It last, Compare comp) {
  It res = first;
  for (It it = first; it != last; ++it) {
    bool r = comp(*it, pivot);
    swap(*res, *it);
    res += r;
  }
  return res;
}

// Splits array into x<=pivot and x>pivot
template <typename T, typename It, typename Compare>
It partition_rev(T pivot, It first, It last, Compare comp) {
  It res = first;
  for (It it = first; it != last; ++it) {
    bool r = comp(pivot, *it);
    swap(*res, *it);
    res += !r;
  }
  return res;
}

// Push root down through the heap
template <typename It, typename Compare>
void heap_sift(It heap, size_t count, size_t root, Compare comp) {
  assert(count > 0);
  size_t last = (count - 1) >> 1;

  while (root < last) {
    assert(root * 2 + 2 < count);

    size_t next = root;
    next = comp(heap[next], heap[root * 2 + 1]) ? root * 2 + 1 : next;
    next = comp(heap[next], heap[root * 2 + 2]) ? root * 2 + 2 : next;

    if (next == root) break;
    swap(heap[root], heap[next]);
    root = next;
  }

  if (root == last && root * 2 + 1 < count &&
      comp(heap[root], heap[root * 2 + 1])) {
    swap(heap[root], heap[root * 2 + 1]);
  }
}

// Sort array using heap sort
template <typename It, typename Compare>
void heap_sort(It first, It last, Compare comp) {
  if (first == last) return;

  It heap = first;
  size_t count = last - first;

  for (size_t i = count / 2; i > 0; --i) {
    heap_sift(heap, count, i - 1, comp);
  }

  for (size_t i = count - 1; i > 0; --i) {
    swap(heap[0], heap[i]);
    heap_sift(heap, i, 0, comp);
  }
}

template <typename T, typename It, typename Compare>
void small_sort(It first, It last, Compare comp) {
  size_t n = last - first;

  for (size_t i = n; i > 1; i -= 2) {
    T x = NANOSORT_MOVE(first[0]);
    T y = NANOSORT_MOVE(first[1]);
    if (comp(y, x)) swap(y, x);

    for (size_t j = 2; j < i; j++) {
      T z = NANOSORT_MOVE(first[j]);

      if (comp(x, z)) swap(x, z);
      if (comp(y, z)) swap(y, z);
      if (comp(y, x)) swap(y, x);

      first[j - 2] = NANOSORT_MOVE(z);
    }

    first[i - 2] = NANOSORT_MOVE(x);
    first[i - 1] = NANOSORT_MOVE(y);
  }
}

template <typename T, typename It, typename Compare>
void sort(It first, It last, size_t limit, Compare comp) {
  for (;;) {
    if (last - first < 16) {
      small_sort<T>(first, last, comp);
      return;
    }

    if (NANOSORT_UNLIKELY(limit == 0)) {
      heap_sort(first, last, comp);
      return;
    }

    T pivot = median5<T>(first, last, comp);
    It mid = partition(pivot, first, last, comp);

    // For skewed partitions compute new midpoint by separating equal elements
    It midr = mid;
    if (NANOSORT_UNLIKELY(mid - first <= (last - first) >> 3)) {
      midr = partition_rev(pivot, mid, last, comp);
    }

    // Per MSVC STL, this allows 1.5 log2(N) recursive steps
    limit = (limit >> 1) + (limit >> 2);

    if (mid - first <= last - midr) {
      sort<T>(first, mid, limit, comp);
      first = midr;
    } else {
      sort<T>(midr, last, limit, comp);
      last = mid;
    }
  }
}

}  // namespace nanosort_detail

template <typename It, typename Compare>
void nanosort(It first, It last, Compare comp) {
  typedef typename nanosort_detail::IteratorTraits<It>::value_type T;
  nanosort_detail::sort<T>(first, last, last - first, comp);
}

template <typename It>
void nanosort(It first, It last) {
  typedef typename nanosort_detail::IteratorTraits<It>::value_type T;
  nanosort_detail::sort<T>(first, last, last - first, nanosort_detail::Less());
}

/**
 * Copyright (c) 2021 Arseny Kapoulkine
 *
 * Permission is hereby granted, free of charge, to any person
 * obtaining a copy of this software and associated documentation
 * files (the "Software"), to deal in the Software without
 * restriction, including without limitation the rights to use,
 * copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following
 * conditions:
 *
 * The above copyright notice and this permission notice shall be
 * included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
 * OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
 * NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
 * HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
 * WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
 * OTHER DEALINGS IN THE SOFTWARE.
 */
