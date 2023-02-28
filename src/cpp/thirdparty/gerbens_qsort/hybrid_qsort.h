// Copyright 2020 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#ifndef EXPERIMENTAL_USERS_GERBENS_HYBRID_QSORT_H_
#define EXPERIMENTAL_USERS_GERBENS_HYBRID_QSORT_H_

#include <algorithm>
#include <cassert>
#include <cstddef>
#include <cstring>
#include <functional>

namespace exp_gerbens {

constexpr ptrdiff_t kSmallSortThreshold = 16;

// Moves median of first, middle, last
template <typename RandomIt, typename Compare>
auto MedianOfThree(RandomIt first,
                   RandomIt last,
                   Compare comp = std::less<>{}) {
  auto n = last - first;
  auto f = *first;
  auto m = first[n >> 1];
  auto l = last[-1];
  using std::swap;
  if (comp(m, f))
    swap(f, m);
  if (comp(l, f))
    swap(f, l);
  if (comp(l, m))
    swap(l, m);
  return m;
}

template <typename RandomIt, typename Compare>
void BranchlessSwap(RandomIt a, RandomIt b, Compare comp) {
  auto x = *a;
  auto y = *b;
  if (comp(y, x))
    std::swap(a, b);
  *a = x;
  *b = y;
}

// Moves median of first, middle, last
template <typename RandomIt, typename Compare>
void MoveMedianOfThreeToEnd(RandomIt first, RandomIt last, Compare comp) {
  auto mid = first + ((last - first) >> 1);
  auto back = last - 1;
  BranchlessSwap(first, mid, comp);
  BranchlessSwap(first, back, comp);
  BranchlessSwap(back, mid, comp);
}

// BubbleSort works better it has N(N-1)/2 stores, but x is updated in the inner
// loop. This is cmp/cmov sequence making the inner loop 2 cycles.
template <typename RandomIt, typename Compare>
void BubbleSort(RandomIt first, RandomIt last, Compare comp = std::less<>{}) {
  auto n = last - first;
  for (auto i = n; i > 1; i--) {
    auto x = first[0];
    for (decltype(n) j = 1; j < i; j++) {
      auto y = first[j];
      bool is_smaller = comp(y, x);
      first[j - 1] = is_smaller ? y : x;
      x = is_smaller ? x : y;
    }
    first[i - 1] = x;
  }
}

// BubbleSort2 bubbles two elements at a time. This means it's doing N(N+1)/4
// iterations and therefore much less stores. Correctly ordering the cmov's it
// is still possible to execute the inner loop in 2 cycles with respect to
// data dependencies. So in effect this cuts running time by 2x, even though
// it's not cutting number of comparisons.
template <typename RandomIt, typename Compare>
void BubbleSort2(RandomIt first, RandomIt last, Compare comp = std::less<>{}) {
  auto n = last - first;
  for (auto i = n; i > 1; i -= 2) {
    auto x = first[0];
    auto y = first[1];
    if (comp(y, x))
      std::swap(x, y);
    for (decltype(n) j = 2; j < i; j++) {
      auto z = first[j];
      bool is_smaller = comp(z, y);
      auto w = is_smaller ? z : y;
      y = is_smaller ? y : z;
      is_smaller = comp(z, x);
      first[j - 2] = is_smaller ? z : x;
      x = is_smaller ? x : w;
    }
    first[i - 2] = x;
    first[i - 1] = y;
  }
}

template <typename RandomIt, typename Compare>
void SmallSort(RandomIt first, RandomIt last, Compare comp) {
  BubbleSort2(first, last, comp);
}

template <typename It, typename ScratchIt, typename Compare>
ScratchIt PartitionInto(It first, It last, ScratchIt out, Compare comp) {
  auto n = last - first;
  auto pivot = first[n - 1];
  auto l = out + n - 1;
#pragma clang loop unroll_count(2)
  for (ptrdiff_t i = -(n - 1); i < 0; i++) {
    auto x = first[i + n - 1];
    bool is_larger = !comp(x, pivot);
    auto dest = is_larger ? 0 : i;
    l[dest] = x;
    l -= is_larger;
  }
  *l = pivot;
  return l;
}

template <typename RandomIt, typename ScratchIt, typename Compare>
void QuickSortScratch(RandomIt first,
                      RandomIt last,
                      ScratchIt scratch,
                      Compare comp);

template <typename RandomIt, typename OutIt, typename Compare>
void QuickSortInto(RandomIt first, RandomIt last, OutIt out, Compare comp) {
  auto n = last - first;
  if (n > kSmallSortThreshold) {
    MoveMedianOfThreeToEnd(first, last, comp);
    auto p = PartitionInto(first, last, out, comp);
    QuickSortScratch(out, p, first, comp);
    QuickSortScratch(p + 1, out + n, first, comp);
  } else {
    SmallSort(first, last, comp);
    std::move(first, last, out);
  }
}

template <typename RandomIt, typename ScratchIt, typename Compare>
void QuickSortScratch(RandomIt first,
                      RandomIt last,
                      ScratchIt scratch,
                      Compare comp) {
  auto n = last - first;
  if (n > kSmallSortThreshold) {
    MoveMedianOfThreeToEnd(first, last, comp);
    auto p = PartitionInto(first, last, scratch, comp);
    QuickSortInto(scratch, p, first, comp);
    first[p - scratch] = *p;
    QuickSortInto(p + 1, scratch + n, first + (p - scratch) + 1, comp);
  } else {
    SmallSort(first, last, comp);
  }
}

// Lomuto inspired partitioning, except it's not in-place and therefore is
// much like bucket sort. It distributes as many elements in the interval
// the interval [first, last) into two buckets. The elements smaller then
// the pivot are distributed in-place at [first, ret). The elements larger
// or equal to the pivot are distributed to the scratch buffer filling it
// backwards. Execution stops when either scratch if full or all elements
// are processed.
template <typename T, typename RandomIt, typename ScratchIt, typename Compare>
RandomIt DistributeForward(T pivot,
                           RandomIt first,
                           RandomIt last,
                           ScratchIt scratch,
                           ptrdiff_t scratch_size,
                           Compare comp) {
  ptrdiff_t larger = 0;
  auto scratch_end = scratch + scratch_size - 1;
  while (first < last) {
    auto x = *first;
    bool is_larger = !comp(x, pivot);
    auto dest = is_larger ? &scratch_end[larger] : &first[larger];
    *dest = x;
    first++;
    larger -= is_larger;
    if (larger == -scratch_size)
      break;
  }
  return first + larger;
}

// Same as above only reversed. This fills the scratch buffer starting at the
// beginning.
template <typename T, typename RandomIt, typename ScratchIt, typename Compare>
RandomIt DistributeBackward(T pivot,
                            RandomIt first,
                            RandomIt last,
                            ScratchIt scratch,
                            ptrdiff_t scratch_size,
                            Compare comp) {
  ptrdiff_t smaller = 0;
  while (first < last) {
    --last;
    auto x = *last;
    bool is_smaller = comp(x, pivot);
    auto dest = is_smaller ? &scratch[smaller] : &last[smaller];
    *dest = x;
    smaller += is_smaller;
    if (smaller == scratch_size)
      break;
  }
  return last + smaller;
}

// New partition algorithm. It's a branch "reduced" hybrid between Hoare and
// a simplified Lomuto partitioning schemes. Lomuto partitioning works by
// ensuring that the first part of the array is properly partitioned with
// respect to the pivot and grow it by the next element, swapping if needed. Now
// obviously you also have a reverse Lomuto partitioning scheme that works
// backwards, mutatis mutandis. Hoare's algorithm is more symmetrical as it
// starts from both ends, working inwards while swapping elements.  Lomuto's
// scheme can be implemented branch free but has the overhead of doing two
// stores per iteration necessary for branchless implementation of swap.
// Furthermore it runs into the problem that the load at the partition index
// potentially depends on previous stores, which quickly disable CPU load store
// reordering.
//
// We can weaken Lomuto partioning scheme by unconditionally storing elements in
// one of two buckets. This is not so much partitioning as it is distributing.
// The algorithm distributes the elements over the two buckets based on the
// pivot. This is much simpler and cheaper. The bucket containing the elements
// smaller than the pivot can overlap with the array, however we need a
// temporary buffer to hold the other elements. At the end we can copy the
// elements of the temporary buffer to the end of the array to achieve a
// partition. Note this would lead to a stable quicksort. Unfortunately such an
// algorithm would not be in-place as it needs O(n) additional memory.

// Let's call this distribution algorithm L', just like Lomuto there is a
// reverse version of it as well. If we make our temporary buffer a small fixed
// size buffer, we have to terminate the distributing when the fixed buffer is
// full, at which point only a part of the array will have been processed.
// Luckily we can leverage a modified version of Hoare's algorithm. Applying L'
// backward with another tempory buffer with the same fixed size, will terminate
// with that buffer full. Now there is enough space in the array to swap the
// temporary buffers with their proper place in the array. What we are getting
// is a tunable Hoare algorithm that works bulkwise, in the limiting case the
// temporary buffers are of size 1, we recover the original Hoare algorithm.
//
// This scheme greatly improves on branchless Lomuto partioning by reducing the
// amount of work that needs to be done in the inner loop and it greatly
// improves on Hoare algorithm by only hitting branch misses every N elements
// and swapping elements wholesale.
template <ptrdiff_t kScratchSize,
          typename RandomIt,
          typename T,
          typename Compare>
RandomIt HoareLomutoHybridPartition(T pivot,
                                    RandomIt first,
                                    RandomIt last,
                                    T* scratch,
                                    Compare comp) {
  auto pfirst =
      DistributeForward(pivot, first, last, scratch, kScratchSize, comp);
  if (auto size = last - pfirst; size <= kScratchSize) {
    std::move(scratch + kScratchSize - size, scratch + kScratchSize, pfirst);
    return pfirst;
  }
  first = pfirst + kScratchSize;
  RandomIt res;
  while (true) {
    last = DistributeBackward(pivot, first, last, first - kScratchSize,
                              kScratchSize, comp) -
           kScratchSize;
    if (last <= first) {
      res = last;
      break;
    }
    first = DistributeForward(pivot, first, last, last, kScratchSize, comp) +
            kScratchSize;
    if (last <= first) {
      res = first - kScratchSize;
      break;
    }
  }
  std::move(scratch, scratch + kScratchSize, res);
  return res;
}

template <ptrdiff_t kScratchSize,
          typename RandomIt,
          typename T,
          typename Compare>
std::pair<RandomIt, RandomIt> ChoosePivotAndPartition(RandomIt first,
                                                      RandomIt last,
                                                      T* scratch,
                                                      Compare comp) {
  auto pivot = MedianOfThree(first, last, comp);
  auto res = HoareLomutoHybridPartition<kScratchSize>(pivot, first, last,
                                                      scratch, comp);
  auto n = last - first;
  auto m = res - first;
  if (m < (n >> 3)) {
    // Fallback path, a surprisingly skewed partition has happened. Likely pivot
    // has many identical elements
    return {res, std::partition(res, last,
                                [&](const T& p) { return !comp(pivot, p); })};
  }
  return {res, res};
}

template <ptrdiff_t kScratchSize,
          typename RandomIt,
          typename T,
          typename Compare>
void QuickSortImpl(RandomIt first, RandomIt last, T* scratch, Compare comp) {
  while (last - first > kScratchSize) {
    auto p = ChoosePivotAndPartition<kScratchSize>(first, last, scratch, comp);
    auto nleft = p.first - first;
    auto nright = last - p.second;
    // Recurse only on the smallest partition guaranteeing O(log n) stack.
    if (nleft <= nright) {
      QuickSortImpl<kScratchSize>(first, p.first, scratch, comp);
      first = p.second;
    } else {
      QuickSortImpl<kScratchSize>(p.second, last, scratch, comp);
      last = p.first;
    }
  }
  //  SmallSort(first, last, comp);
  QuickSortScratch(first, last, scratch, comp);
}

constexpr ptrdiff_t SCRATCH_SIZE_DEFAULT = 128;

template <ptrdiff_t kScratchSize = SCRATCH_SIZE_DEFAULT,
          typename RandomIt,
          typename Compare>
void QuickSort(RandomIt first, RandomIt last, Compare comp) {
  static_assert(kScratchSize > 0, "Must have a positive scratch space size");
  using T = typename std::decay<decltype(*first)>::type;
  T scratch[kScratchSize];
  QuickSortImpl<kScratchSize>(first, last, scratch, comp);
}

template <ptrdiff_t kScratchSize = SCRATCH_SIZE_DEFAULT, typename RandomIt>
void QuickSort(RandomIt first, RandomIt last) {
  QuickSort<kScratchSize>(first, last, std::less<>{});
}

}  // namespace exp_gerbens

#endif  // EXPERIMENTAL_USERS_GERBENS_HYBRID_QSORT_H_
