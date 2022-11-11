/** @author Sebastian Wild (wild@liverpool.ac.uk) */

#ifndef MERGESORTS_POWERSORT_4WAY_H
#define MERGESORTS_POWERSORT_4WAY_H

#include <cassert>
#include <cmath>
#include "algorithms.h"
#include "insertionsort.h"
#include "merging.h"
#include "merging_3way.h"
#include "merging_multiway.h"
#include "powersort.h"

namespace algorithms {

/** Print stats about calls to non-4way merges */
//#define PRINT_MERGES_AND_MERGECOST_PER_K true

/**
 * Different choices for how to compute node powers.
 */
enum node_power4_implementations {
  TRIVIAL4,
  DIVISION_LOOP4,
  BITWISE_LOOP4,
  MOST_SIGNIFICANT_SET_BIT4,
};

std::string to_string(node_power4_implementations implementation) {
  switch (implementation) {
    case TRIVIAL4:
      return "TRIVIAL";
    case DIVISION_LOOP4:
      return "DIVISION_LOOP";
    case BITWISE_LOOP4:
      return "BITWISE_LOOP";
    case MOST_SIGNIFICANT_SET_BIT4:
      return "MOST_SIGNIFICANT_SET_BIT";
  }
  assert(false);
  __builtin_unreachable();
};

power_t node_power4_trivial(size_t begin,
                            size_t end,
                            size_t beginA,
                            size_t beginB,
                            size_t endB) {
  size_t n = end - begin;
  size_t n1 = beginB - beginA, n2 = endB - beginB;
  double a = ((beginA - begin) + 0.5 * n1) / n;
  double b = ((beginB - begin) + 0.5 * n2) / n;
  power_t k = 0;
  while (true) {
    ++k;
    unsigned long fourToK = 1u << (2 * k);
    if (floor(a * fourToK) < floor(b * fourToK))
      break;
  }
  return k;
}

power_t node_power4_div(size_t begin,
                        size_t end,
                        size_t beginA,
                        size_t beginB,
                        size_t endB) {
  size_t twoN = 2 * (end - begin);                  // 2*n
  size_t n1 = beginB - beginA, n2 = endB - beginB;  // lengths of runs
  unsigned long a = 2 * beginA + n1 - 2 * begin;
  unsigned long b = 2 * beginB + n2 - 2 * begin;
  power_t k = 0;
  while (b - a <= twoN && a / twoN == b / twoN) {
    ++k;
    a *= 4;
    b *= 4;
  }
  return k;
}

power_t node_power4_bitwise(size_t begin,
                            size_t end,
                            size_t beginA,
                            size_t beginB,
                            size_t endB) {
  size_t n = end - begin;
  assert(n < (1L << 63));
  size_t l = beginA - begin + beginB - begin;
  size_t r = beginB - begin + endB - begin;
  // a and b are given by l/(2*n) and r/(2*n), both are in [0,1).
  // we have to find the number of common digits in the
  // base-4 representation in the fractional part.
  // That is the same as the number of common bits in the binary
  // representation divided by 2 (rounded down).
  power_t nCommonBits = 0;
  bool digitA = l >= n, digitB = r >= n;
  while (digitA == digitB) {
    ++nCommonBits;
    if (digitA) {
      l -= n;
      r -= n;
    }
    l *= 2;
    r *= 2;
    digitA = l >= n;
    digitB = r >= n;
  }
  return nCommonBits / 2 + 1;
}

power_t node_power4_clz(size_t begin,
                        size_t end,
                        size_t beginA,
                        size_t beginB,
                        size_t endB) {
  size_t n = end - begin;
  assert(n <= (1L << 31));
  unsigned long l2 = beginA + beginB - 2 * begin;  // 2*l
  unsigned long r2 = beginB + endB - 2 * begin;    // 2*r
  auto a = static_cast<unsigned int>((l2 << 30) / n);
  auto b = static_cast<unsigned int>((r2 << 30) / n);
  return (__builtin_clz(a ^ b) - 1) / 2 + 1;
}

#ifdef PRINT_MERGES_AND_MERGECOST_PER_K
static long nMerges3, nMerges4, nMerges2;
static long long mergeCost2, mergeCost3, mergeCost4;
#endif

/**
 * 4-way Powersort implementation based on William Cawley Gelling's code.
 *
 * Natural runs are extended to minRunLen if needed before we continue
 * merging.
 * Unless useMsbMergeType is false, node powers are computed using
 * a most-significant-bit trick;
 * otherwise a loop is used.
 * If onlyIncreasingRuns is true, only weakly increasing runs are picked up.
 *
 * @author Sebastian Wild (wild@liverpool.ac.uk)
 */
template <typename Iterator,
          unsigned int minRunLen = 24,
          merging4way_methods mergingMethod = WILLEM_TUNED,
          bool onlyIncreasingRuns = false,
          node_power4_implementations nodePowerImplementation =
              MOST_SIGNIFICANT_SET_BIT4 /** very little difference */,
          bool useParallelArraysForStack = false, /** very little difference*/
          bool useCheckFirstMergeLoop = true /** very little difference */,
          bool useSpecialized3wayMerge =
              true /** no huge difference, but no detriment */
          >
class powersort_4way final : public sorter<Iterator> {
 private:
  using typename sorter<Iterator>::elem_t;
  using typename sorter<Iterator>::diff_t;
  std::vector<elem_t> _buffer;
  Iterator globalBegin, globalEnd;

  struct run_begin_n_power {
    Iterator begin;
    power_t power;
  };

  struct run {
    Iterator begin;
    Iterator end;
  };

  struct run_n_power {
    Iterator begin;
    Iterator end;
    power_t power = 0;
  };

  run_begin_n_power NULL_RUN_N_POWER{};

 public:
  void sort(Iterator begin, Iterator end) override {
    _buffer.resize(end - begin + 4);
    globalBegin = begin;
    globalEnd = end;
    if (useParallelArraysForStack)
      power_sort_paper_parallel_arrays(begin, end);
    else
      power_sort_paper(begin, end);
  }

  power_t node_power(size_t begin,
                     size_t end,
                     size_t beginA,
                     size_t beginB,
                     size_t endB) {
    switch (nodePowerImplementation) {
      case MOST_SIGNIFICANT_SET_BIT4:
        return node_power4_clz(begin, end, beginA, beginB, endB);
      case BITWISE_LOOP4:
        return node_power4_bitwise(begin, end, beginA, beginB, endB);
      case DIVISION_LOOP4:
        return node_power4_div(begin, end, beginA, beginB, endB);
      case TRIVIAL4:
        return node_power4_trivial(begin, end, beginA, beginB, endB);
    }
    assert(false);
    __builtin_unreachable();
  }

  /**
   * sorts [begin,end), assuming that [begin,leftRunEnd) and
   * [rightRunBegin,end) are sorted
   * uses the explicit stack from the pair
   */
  void power_sort_paper(Iterator begin, Iterator end) {
#ifdef PRINT_MERGES_AND_MERGECOST_PER_K
    nMerges2 = nMerges3 = nMerges4 = 0;
    mergeCost2 = mergeCost3 = mergeCost4 = 0;
#endif
    const size_t n = end - begin;
    const unsigned maxStackHeight = 3 * (floor_log2(n) / 2) + 2;
#ifdef ALLOCATE_RUN_STACK_ON_HEAP
    auto stack = new run_n_power[maxStackHeight];
#else
    run_begin_n_power stack[maxStackHeight];
#endif
    run_begin_n_power* top_of_stack = stack;  // topmost valid stack element
    *top_of_stack =
        NULL_RUN_N_POWER;  // keep on NULL_RUN_N_POWER in stack[0] as sentinel
    run_begin_n_power* const end_of_stack = stack + maxStackHeight;

    run_n_power runA = {begin, extend_and_reverse_run_right(begin, end), 0};
    // extend to minRunLen
    if (diff_t lenA = runA.end - runA.begin < minRunLen) {
      runA.end = std::min(end, runA.begin + minRunLen);
      insertionsort(runA.begin, runA.end, lenA);
    }
    while (runA.end < end) {
      run runB = {runA.end, extend_and_reverse_run_right(runA.end, end)};
      // extend to minRunLen
      if (size_t lenB = runB.end - runB.begin < minRunLen) {
        runB.end = std::min(end, runB.begin + minRunLen);
        insertionsort(runB.begin, runB.end, lenB);
      }
      runA.power =
          node_power(0, n, (size_t)(runA.begin - begin),
                     (size_t)(runB.begin - begin), (size_t)(runB.end - begin));
      // Invariant: powers on stack must be *weakly* increasing from bottom to
      // top
      while (top_of_stack->power > runA.power) {
        if (useCheckFirstMergeLoop)
          merge_loop_check_first(top_of_stack, runA);
        else
          merge_loop(top_of_stack, runA);
      }
      // store updated runA to be merged with runB at power k
      assert(top_of_stack < end_of_stack);
      *(++top_of_stack) = {runA.begin, runA.power};  // push
      runA = {runB.begin, runB.end, 0};
    }
    assert(runA.end == end);
    merge_down(stack, top_of_stack, runA);
    assert(top_of_stack == stack);
#ifdef ALLOCATE_RUN_STACK_ON_HEAP
    delete[] stack;
#endif
#ifdef PRINT_MERGES_AND_MERGECOST_PER_K
    std::cout << "nMerges2: " << nMerges2 << std::endl;
    std::cout << "nMerges3: " << nMerges3 << std::endl;
    std::cout << "nMerges4: " << nMerges4 << std::endl;
    std::cout << "mergeCost2: " << mergeCost2 << std::endl;
    std::cout << "mergeCost3: " << mergeCost3 << std::endl;
    std::cout << "mergeCost4: " << mergeCost4 << std::endl;
    std::cout << "average run length in 2way merges: "
              << ((float)mergeCost2) / (2 * nMerges2) << std::endl;
    std::cout << "average run length in 3way merges: "
              << ((float)mergeCost3) / (3 * nMerges3) << std::endl;
    std::cout << "average run length in 4way merges: "
              << ((float)mergeCost4) / (4 * nMerges4) << std::endl;
#endif
  }

  void merge_loop_check_first(run_begin_n_power*& top_of_stack,
                              run_n_power& runA) {
    int nRunsSamePower = 1;
    while ((top_of_stack - nRunsSamePower)->power == top_of_stack->power)
      ++nRunsSamePower;
    if (nRunsSamePower == 1) {  // 2way
      Iterator g[] = {top_of_stack->begin};
      merge_runs<COPY_BOTH>(g[0], runA.begin, runA.end, _buffer.begin());
      runA.begin = g[0];
#ifdef PRINT_MERGES_AND_MERGECOST_PER_K
      ++nMerges2;
      mergeCost2 += runA.end - runA.begin;
#endif
    } else if (nRunsSamePower == 2) {  // 3way
      Iterator g[] = {(top_of_stack - 1)->begin, top_of_stack->begin};
      if (useSpecialized3wayMerge)
        merge_3runs<mergingMethod>(g[0], g[1], runA.begin, runA.end,
                                   _buffer.begin());
      else
        merge_4runs<mergingMethod>(g[0], g[1], runA.begin, runA.end, runA.end,
                                   _buffer.begin());
      runA.begin = g[0];
#ifdef PRINT_MERGES_AND_MERGECOST_PER_K
      ++nMerges3;
      mergeCost3 += runA.end - runA.begin;
#endif
    } else {  // 4way
      assert(nRunsSamePower == 3);
      Iterator g[] = {(top_of_stack - 2)->begin, (top_of_stack - 1)->begin,
                      top_of_stack->begin};
      merge_4runs<mergingMethod>(g[0], g[1], g[2], runA.begin, runA.end,
                                 _buffer.begin());
      runA.begin = g[0];
#ifdef PRINT_MERGES_AND_MERGECOST_PER_K
      ++nMerges4;
      mergeCost4 += runA.end - runA.begin;
#endif
    }
    top_of_stack -= nRunsSamePower;  // pop runs with same power
  }

  void merge_loop(run_begin_n_power*& top_of_stack, run_n_power& runA) {
    Iterator g[3];  // boundaries between 4 runs: [g[0],g[1]), [g[1],g[2]),
                    // [g[2],runA.begin) and [runA.begin,runA.end)
    run_begin_n_power topRun = *top_of_stack--;  // pop
    g[2] = topRun.begin;
    if (top_of_stack->power != topRun.power) {  // 2way
      // use specialized method (had no measurable effect for rp ...)
      merge_runs<COPY_BOTH>(g[2], runA.begin, runA.end, _buffer.begin());
      runA.begin = g[2];
#ifdef PRINT_MERGES_AND_MERGECOST_PER_K
      ++nMerges2;
      mergeCost2 += runA.end - runA.begin;
#endif
    } else if ((top_of_stack - 1)->power != topRun.power) {  // 3way
      g[1] = (top_of_stack--)->begin;                        // pop
      if (useSpecialized3wayMerge)
        merge_3runs<mergingMethod>(g[1], g[2], runA.begin, runA.end,
                                   _buffer.begin());
      else
        merge_4runs<mergingMethod>(g[1], g[2], runA.begin, runA.end, runA.end,
                                   _buffer.begin());
      runA.begin = g[1];
#ifdef PRINT_MERGES_AND_MERGECOST_PER_K
      ++nMerges3;
      mergeCost3 += runA.end - runA.begin;
#endif
    } else {                           // 4way
      g[1] = (top_of_stack--)->begin;  // pop
      g[0] = (top_of_stack--)->begin;  // pop
      merge_4runs<mergingMethod>(g[0], g[1], g[2], runA.begin, runA.end,
                                 _buffer.begin());
      runA.begin = g[0];
#ifdef PRINT_MERGES_AND_MERGECOST_PER_K
      ++nMerges4;
      mergeCost4 += runA.end - runA.begin;
#endif
    }
  }

  void merge_down(run_begin_n_power* begin_of_stack,
                  run_begin_n_power*& top_of_stack,
                  run_n_power& runA) {
    // we have the entire stack of runs, so instead of following exactly the
    // powersort rule, we can be slightly more clever and make sure we have 4way
    // merges all the way through except the first merge
    auto nRuns = top_of_stack - begin_of_stack + 1;  // stack and runA
    // We want 3k+1 runs, so that repeatedly merging 4 and putting the result
    // back gives 4way merges all the way through.
    switch (nRuns % 3) {
      case 0:  // merge topmost 3 runs
        assert(nRuns >= 3);
        if (useSpecialized3wayMerge)
          merge_3runs<mergingMethod>((top_of_stack - 1)->begin,
                                     top_of_stack->begin, runA.begin, runA.end,
                                     _buffer.begin());
        else
          merge_4runs<mergingMethod>((top_of_stack - 1)->begin,
                                     top_of_stack->begin, runA.begin, runA.end,
                                     runA.end, _buffer.begin());
        runA.begin = (top_of_stack - 1)->begin;
#ifdef PRINT_MERGES_AND_MERGECOST_PER_K
        ++nMerges3;
        mergeCost3 += runA.end - runA.begin;
#endif
        top_of_stack -= 2;
        break;
      case 2:  // merge topmost 2 runs
        merge_runs<COPY_BOTH>(top_of_stack->begin, runA.begin, runA.end,
                              _buffer.begin());
        runA.begin = top_of_stack->begin;
#ifdef PRINT_MERGES_AND_MERGECOST_PER_K
        ++nMerges2;
        mergeCost2 += runA.end - runA.begin;
#endif
        --top_of_stack;
        break;
      default:
        break;
    }
    assert(((top_of_stack - begin_of_stack) % 3) == 0);
    // merge remaining stack 4way each
    while (top_of_stack > begin_of_stack) {
      merge_4runs<mergingMethod>((top_of_stack - 2)->begin,
                                 (top_of_stack - 1)->begin, top_of_stack->begin,
                                 runA.begin, runA.end, _buffer.begin());
      runA.begin = (top_of_stack - 2)->begin;
#ifdef PRINT_MERGES_AND_MERGECOST_PER_K
      ++nMerges4;
      mergeCost4 += runA.end - runA.begin;
#endif
      top_of_stack -= 3;
    }
  }

  /**
   * sorts [begin,end), assuming that [begin,leftRunEnd) and
   * [rightRunBegin,end) are sorted
   * uses the explicit stack from the pair
   *
   * @deprecated not faster and less readable
   */
  void power_sort_paper_parallel_arrays(Iterator begin, Iterator end) {
    assert(useCheckFirstMergeLoop && "not implemented");
#ifdef PRINT_MERGES_AND_MERGECOST_PER_K
    nMerges2 = nMerges3 = nMerges4 = 0;
    mergeCost2 = mergeCost3 = mergeCost4 = 0;
#endif
    const size_t n = end - begin;
    const unsigned maxStackHeight = 3 * (floor_log2(n) / 2) + 2;
#ifdef ALLOCATE_RUN_STACK_ON_HEAP
    assert(false && "not implemented");
    auto stack = new run_n_power[maxStackHeight];
#else
    power_t stack_power[maxStackHeight];  // powers
    Iterator stack_run[maxStackHeight];   // left endpoints of runs
#endif
    power_t* top_of_stack_power = stack_power;  // topmost valid stack element
    Iterator* top_of_stack_run = stack_run;     // topmost valid stack element
    *top_of_stack_power =
        0;  // keep on NULL_RUN_N_POWER in stack[0] as sentinel
    power_t* const end_of_stack_power = stack_power + maxStackHeight;

    run_n_power runA = {begin, extend_and_reverse_run_right(begin, end), 0};
    // extend to minRunLen
    if (diff_t lenA = runA.end - runA.begin < minRunLen) {
      runA.end = std::min(end, runA.begin + minRunLen);
      insertionsort(runA.begin, runA.end, lenA);
    }
    while (runA.end < end) {
      run runB = {runA.end, extend_and_reverse_run_right(runA.end, end)};
      // extend to minRunLen
      if (size_t lenB = runB.end - runB.begin < minRunLen) {
        runB.end = std::min(end, runB.begin + minRunLen);
        insertionsort(runB.begin, runB.end, lenB);
      }
      runA.power =
          node_power(0, n, (size_t)(runA.begin - begin),
                     (size_t)(runB.begin - begin), (size_t)(runB.end - begin));
      // Invariant: powers on stack must be *weakly* increasing from bottom to
      // top
      while (*top_of_stack_power > runA.power) {
        if (useCheckFirstMergeLoop)
          merge_loop_check_first_parallel_arrays(top_of_stack_power,
                                                 top_of_stack_run, runA);
        else
          assert(false && "not implemented");
      }
      // store updated runA on stack
      assert(top_of_stack_power < end_of_stack_power);
      *(++top_of_stack_power) = runA.power;  // push
      *(++top_of_stack_run) = runA.begin;    // push
      runA = {runB.begin, runB.end, 0};
    }
    assert(runA.end == end);
    merge_down_parallel_arrays(stack_run, top_of_stack_run, runA);
    assert(top_of_stack_run == stack_run);
#ifdef ALLOCATE_RUN_STACK_ON_HEAP
    // delete[] stack;
#endif
#ifdef PRINT_MERGES_AND_MERGECOST_PER_K
    std::cout << "nMerges2: " << nMerges2 << std::endl;
    std::cout << "nMerges3: " << nMerges3 << std::endl;
    std::cout << "nMerges4: " << nMerges4 << std::endl;
    std::cout << "mergeCost2: " << mergeCost2 << std::endl;
    std::cout << "mergeCost3: " << mergeCost3 << std::endl;
    std::cout << "mergeCost4: " << mergeCost4 << std::endl;
    std::cout << "average run length in 2way merges: "
              << ((float)mergeCost2) / (2 * nMerges2) << std::endl;
    std::cout << "average run length in 3way merges: "
              << ((float)mergeCost3) / (3 * nMerges3) << std::endl;
    std::cout << "average run length in 4way merges: "
              << ((float)mergeCost4) / (4 * nMerges4) << std::endl;
#endif
  }

  void merge_loop_check_first_parallel_arrays(power_t*& top_of_stack_power,
                                              Iterator*& top_of_stack_run,
                                              run_n_power& runA) {
    int nRunsSamePower = 1;
    while (*(top_of_stack_power - nRunsSamePower) == *top_of_stack_power)
      ++nRunsSamePower;
    if (nRunsSamePower == 1) {  // 2way
      Iterator g[] = {*top_of_stack_run};
      merge_runs<COPY_BOTH>(g[0], runA.begin, runA.end, _buffer.begin());
      runA.begin = g[0];
#ifdef PRINT_MERGES_AND_MERGECOST_PER_K
      ++nMerges2;
      mergeCost2 += runA.end - runA.begin;
#endif
    } else if (nRunsSamePower == 2) {  // 3way
      Iterator g[] = {*(top_of_stack_run - 1), *top_of_stack_run};
      if (useSpecialized3wayMerge)
        merge_3runs<mergingMethod>(g[0], g[1], runA.begin, runA.end,
                                   _buffer.begin());
      else
        merge_4runs<mergingMethod>(g[0], g[1], runA.begin, runA.end, runA.end,
                                   _buffer.begin());
      runA.begin = g[0];
#ifdef PRINT_MERGES_AND_MERGECOST_PER_K
      ++nMerges3;
      mergeCost3 += runA.end - runA.begin;
#endif
    } else {  // 4way
      assert(nRunsSamePower == 3);
      Iterator g[] = {*(top_of_stack_run - 2), *(top_of_stack_run - 1),
                      *top_of_stack_run};
      merge_4runs<mergingMethod>(g[0], g[1], g[2], runA.begin, runA.end,
                                 _buffer.begin());
      runA.begin = g[0];
#ifdef PRINT_MERGES_AND_MERGECOST_PER_K
      ++nMerges4;
      mergeCost4 += runA.end - runA.begin;
#endif
    }
    top_of_stack_power -= nRunsSamePower;  // pop runs with same power
    top_of_stack_run -= nRunsSamePower;    // pop runs with same power
  }

  void merge_down_parallel_arrays(Iterator* begin_of_stack_run,
                                  Iterator*& top_of_stack_run,
                                  run_n_power& runA) {
    // we have the entire stack of runs, so instead of following exactly the
    // powersort rule, we can be slightly more clever and make sure we have 4way
    // merges all the way through except the first merge
    auto nRuns = top_of_stack_run - begin_of_stack_run + 1;  // stack and runA
    // We want 3k+1 runs, so that repeatedly merging 4 and putting the result
    // back gives 4way merges all the way through.
    switch (nRuns % 3) {
      case 0:  // merge topmost 3 runs
        assert(nRuns >= 3);
        if (useSpecialized3wayMerge)
          merge_3runs<mergingMethod>(*(top_of_stack_run - 1), *top_of_stack_run,
                                     runA.begin, runA.end, _buffer.begin());
        else
          merge_4runs<mergingMethod>(*(top_of_stack_run - 1), *top_of_stack_run,
                                     runA.begin, runA.end, runA.end,
                                     _buffer.begin());
        runA.begin = *(top_of_stack_run - 1);
#ifdef PRINT_MERGES_AND_MERGECOST_PER_K
        ++nMerges3;
        mergeCost3 += runA.end - runA.begin;
#endif
        top_of_stack_run -= 2;
        break;
      case 2:  // merge topmost 2 runs
        merge_runs<COPY_BOTH>(*top_of_stack_run, runA.begin, runA.end,
                              _buffer.begin());
        runA.begin = *top_of_stack_run;
#ifdef PRINT_MERGES_AND_MERGECOST_PER_K
        ++nMerges2;
        mergeCost2 += runA.end - runA.begin;
#endif
        --top_of_stack_run;
        break;
      default:
        break;
    }
    assert(((top_of_stack_run - begin_of_stack_run) % 3) == 0);
    // merge remaining stack 4way each
    while (top_of_stack_run > begin_of_stack_run) {
      merge_4runs<mergingMethod>(*(top_of_stack_run - 2),
                                 *(top_of_stack_run - 1), *top_of_stack_run,
                                 runA.begin, runA.end, _buffer.begin());
      runA.begin = *(top_of_stack_run - 2);
#ifdef PRINT_MERGES_AND_MERGECOST_PER_K
      ++nMerges4;
      mergeCost4 += runA.end - runA.begin;
#endif
      top_of_stack_run -= 3;
    }
  }

  std::string name() const override {
    return "PowerSort4Way+minRunLen=" + std::to_string(minRunLen) +
           "+mergeMethod=" + to_string(mergingMethod) +
           "+onlyIncRuns=" + std::to_string(onlyIncreasingRuns);
  }

  std::string full_name() const {
    return "PowerSort4Way+minRunLen=" + std::to_string(minRunLen) +
           "+nodePowerImplementation=" + to_string(nodePowerImplementation) +
           "+mergeMethod=" + to_string(mergingMethod) +
           "+onlyIncRuns=" + std::to_string(onlyIncreasingRuns) +
           +"+useParallelArraysForStack=" +
           std::to_string(useParallelArraysForStack) +
           "+useSpecialized3wayMerge=" +
           std::to_string(useSpecialized3wayMerge) +
           "+useCheckFirstMergeLoop=" + std::to_string(useCheckFirstMergeLoop);
  }
};

}  // namespace algorithms

#endif  // MERGESORTS_POWERSORT_4WAY_H
