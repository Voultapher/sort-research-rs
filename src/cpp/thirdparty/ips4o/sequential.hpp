/******************************************************************************
 * include/ips4o/sequential.hpp
 *
 * In-place Parallel Super Scalar Samplesort (IPS⁴o)
 *
 ******************************************************************************
 * BSD 2-Clause License
 *
 * Copyright © 2017, Michael Axtmann <michael.axtmann@gmail.com>
 * Copyright © 2017, Daniel Ferizovic <daniel.ferizovic@student.kit.edu>
 * Copyright © 2017, Sascha Witt <sascha.witt@kit.edu>
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * * Redistributions of source code must retain the above copyright notice, this
 *   list of conditions and the following disclaimer.
 *
 * * Redistributions in binary form must reproduce the above copyright notice,
 *   this list of conditions and the following disclaimer in the documentation
 *   and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
 * SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
 * CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
 * OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 *****************************************************************************/

#pragma once

#include <utility>

#include "ips4o_fwd.hpp"
#include "base_case.hpp"
#include "memory.hpp"
#include "partitioning.hpp"
#include "scheduler.hpp"

namespace ips4o {
namespace detail {

#if defined(_REENTRANT)

/**
 * Recursive entry point for sequential algorithm.
 */
template <class Cfg>
void Sorter<Cfg>::sequential(const iterator begin, const Task& task,
                             PrivateQueue<Task>& queue) {
    // Check for base case
    const auto n = task.end - task.begin;
    IPS4OML_IS_NOT(n <= 2 * Cfg::kBaseCaseSize);

    diff_t bucket_start[Cfg::kMaxBuckets + 1];

    // Do the partitioning
    const auto res =
            partition<false>(begin + task.begin, begin + task.end, bucket_start, 0, 1);
    const int num_buckets = std::get<0>(res);
    const bool equal_buckets = std::get<1>(res);

    // Final base case is executed in cleanup step, so we're done here
    if (n <= Cfg::kSingleLevelThreshold) {
        return;
    }

    // Recurse
    if (equal_buckets) {
        const auto start = bucket_start[num_buckets - 1];
        const auto stop = bucket_start[num_buckets];
        if (stop - start > 2 * Cfg::kBaseCaseSize) {
            queue.emplace(task.begin + start, task.begin + stop);
        }
    }
    for (int i = num_buckets - 1 - equal_buckets; i >= 0; i -= 1 + equal_buckets) {
        const auto start = bucket_start[i];
        const auto stop = bucket_start[i + 1];
        if (stop - start > 2 * Cfg::kBaseCaseSize) {
            queue.emplace(task.begin + start, task.begin + stop);
        }
    }
}

#endif  // _REENTRANT

/**
 * Recursive entry point for sequential algorithm.
 */
template <class Cfg>
void Sorter<Cfg>::sequential(const iterator begin, const iterator end) {
    // Check for base case
    const auto n = end - begin;
    if (n <= 2 * Cfg::kBaseCaseSize) {
#ifdef IPS4O_TIMER
        g_overhead.stop();
        g_base_case.start();
#endif

        detail::baseCaseSort(begin, end, local_.classifier.getComparator());

#ifdef IPS4O_TIMER
        g_base_case.stop();
        g_overhead.start();
#endif

        return;
    }

    sequential_rec(begin, end);
}

/**
 * Recursive entry point for sequential algorithm.
 */
template <class Cfg>
void Sorter<Cfg>::sequential_rec(const iterator begin, const iterator end) {
    // Check for base case
    const auto n = end - begin;
    IPS4OML_IS_NOT(n <= 2 * Cfg::kBaseCaseSize);

    diff_t bucket_start[Cfg::kMaxBuckets + 1];

    // Do the partitioning
    const auto res = partition<false>(begin, end, bucket_start, 0, 1);
    const int num_buckets = std::get<0>(res);
    const bool equal_buckets = std::get<1>(res);

    // Final base case is executed in cleanup step, so we're done here
    if (n <= Cfg::kSingleLevelThreshold) {
        return;
    }

#ifdef IPS4O_TIMER
    g_ips4o_level++;
#endif

    // Recurse
    for (int i = 0; i < num_buckets; i += 1 + equal_buckets) {
        const auto start = bucket_start[i];
        const auto stop = bucket_start[i + 1];
        if (stop - start > 2 * Cfg::kBaseCaseSize)
            sequential(begin + start, begin + stop);
    }
    if (equal_buckets) {
        const auto start = bucket_start[num_buckets - 1];
        const auto stop = bucket_start[num_buckets];
        if (stop - start > 2 * Cfg::kBaseCaseSize)
            sequential(begin + start, begin + stop);
    }

#ifdef IPS4O_TIMER
    g_ips4o_level--;
#endif

}

}  // namespace detail

/**
 * Reusable sequential sorter.
 */
template <class Cfg>
class SequentialSorter {
    using Sorter = detail::Sorter<Cfg>;
    using iterator = typename Cfg::iterator;

 public:
    explicit SequentialSorter(bool check_sorted, typename Cfg::less comp)
        : check_sorted_(check_sorted)
        , buffer_storage_(1)
        , local_ptr_(Cfg::kDataAlignment, std::move(comp), buffer_storage_.get()) {}

    explicit SequentialSorter(bool check_sorted, typename Cfg::less comp,
                              char* buffer_storage)
        : check_sorted_(check_sorted)
        , local_ptr_(Cfg::kDataAlignment, std::move(comp), buffer_storage) {}

    void operator()(iterator begin, iterator end) {
        if (check_sorted_) {
            const bool sorted = detail::sortSimpleCases(
                    begin, end, local_ptr_.get().classifier.getComparator());
            if (sorted) return;
        }

        Sorter(local_ptr_.get()).sequential(std::move(begin), std::move(end));
    }

 private:
    const bool check_sorted_;
    typename Sorter::BufferStorage buffer_storage_;
    detail::AlignedPtr<typename Sorter::LocalData> local_ptr_;
};

}  // namespace ips4o
