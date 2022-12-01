/******************************************************************************
 * include/ips4o/parallel.hpp
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
#if defined(_REENTRANT)

#include <algorithm>
#include <atomic>
#include <condition_variable>
#include <cstdint>
#include <functional>
#include <memory>
#include <mutex>
#include <thread>
#include <utility>
#include <vector>

#include <tbb/concurrent_queue.h>

#include "ips4o_fwd.hpp"
#include "config.hpp"
#include "memory.hpp"
#include "partitioning.hpp"
#include "scheduler.hpp"
#include "sequential.hpp"
#include "task.hpp"

namespace ips4o {
namespace detail {

/**
 * Processes sequential subtasks in the parallel algorithm.
 */
template <class Cfg>
void Sorter<Cfg>::processSmallTasks(const iterator begin) {
    auto& scheduler = shared_->scheduler;
    auto& my_queue = local_.seq_task_queue;
    Task task;
    auto comp = local_.classifier.getComparator();

    while (scheduler.getJob(my_queue, task)) {
        scheduler.offerJob(my_queue);
        if (task.end - task.begin <= 2 * Cfg::kBaseCaseSize) {
#ifdef IPS4O_TIMER
            g_overhead.stop();
            g_base_case.start();
#endif

            detail::baseCaseSort(begin + task.begin, begin + task.end, comp);

#ifdef IPS4O_TIMER
            g_base_case.stop();
            g_overhead.start();
#endif
        } else {
            sequential(begin, task, my_queue);
        }
    }
}

template <class Cfg>
void Sorter<Cfg>::queueTasks(const diff_t stripe, const int id,
                             const int task_num_threads, const diff_t parent_task_size,
                             const diff_t offset, const diff_t* bucket_start,
                             int num_buckets, bool equal_buckets) {
    // create a new task sorter on subsequent levels

    const diff_t parent_task_stripe =
            (parent_task_size + task_num_threads - 1) / task_num_threads;

    const auto queueTask = [&](const diff_t task_begin, const diff_t task_end) {
        const int thread_begin = (offset + task_begin + stripe / 2) / stripe;
        const int thread_end = (offset + task_end + stripe / 2) / stripe;

        const auto task_size = task_end - task_begin;

        if (thread_end - thread_begin <= 1
            || task_end - task_begin <= Cfg::kBaseCaseSize) {
            const auto thread = (task_begin + task_size / 2) / parent_task_stripe;

            shared_->local[id + thread]->seq_task_queue.emplace(offset + task_begin,
                                                                offset + task_end);

        } else {
            shared_->thread_pools[thread_begin] =
                    std::make_shared<SubThreadPool>(thread_end - thread_begin);

            for (auto t = thread_begin; t != thread_end; ++t) {
                auto& bt = shared_->big_tasks[t];

                bt.begin = offset + task_begin;
                bt.end = offset + task_end;
                bt.task_thread_id = t - thread_begin;
                bt.root_thread = thread_begin;
                bt.has_task = true;
            }
        }
    };

    for (auto t = id; t != id + task_num_threads; ++t) {
        shared_->big_tasks[t].has_task = false;
    }

    // Queue subtasks if we didn't reach the last level yet
    const bool is_last_level = parent_task_size <= Cfg::kSingleLevelThreshold;
    if (!is_last_level) {
        if (equal_buckets) {
            const auto start = bucket_start[num_buckets - 1];
            const auto stop = bucket_start[num_buckets];
            if (start < stop) queueTask(start, stop);
        }

        // Skip equality buckets
        for (int i = num_buckets - 1 - equal_buckets; i >= 0; i -= 1 + equal_buckets) {
            const auto start = bucket_start[i];
            const auto stop = bucket_start[i + 1];
            if (start < stop) queueTask(start, stop);
        }
    }
}

/**
 * Process a big task with multiple threads in the parallel algorithm.
 */
template <class Cfg>
void Sorter<Cfg>::processBigTasks(const iterator begin, const diff_t stripe, const int id,
                                  BufferStorage& buffer_storage,
                                  std::vector<std::shared_ptr<SubThreadPool>>& tp_trash) {
    BigTask& task = shared_->big_tasks[id];

    while (task.has_task) {
        if (task.root_thread == id) {
            // Only thread 0 passes a task sorter (the one stored in this
            // object). The other threads have to create a task sorter if
            // required.
            processBigTaskPrimary(begin, stripe, id, buffer_storage, tp_trash);
        } else {
            processBigTasksSecondary(id);
        }
    }
}

/**
 * Set shared data.
 */
template <class Cfg>
void Sorter<Cfg>::setShared(SharedData* shared) {
    shared_ = shared;
}

/**
 * Process a big task with multiple threads in the parallel algorithm.
 */
template <class Cfg>
void Sorter<Cfg>::processBigTasksSecondary(const int id) {
    BigTask& task = shared_->big_tasks[id];
    auto partial_thread_pool = shared_->thread_pools[task.root_thread];

    partial_thread_pool->join(task.task_thread_id);
}

/**
 * Process a big task with multiple threads in the parallel algorithm.
 */
template <class Cfg>
void Sorter<Cfg>::processBigTaskPrimary(
        const iterator begin, const diff_t stripe, const int id,
        BufferStorage& buffer_storage,
        std::vector<std::shared_ptr<SubThreadPool>>& tp_trash) {
    BigTask& task = shared_->big_tasks[id];

    // Thread pool of this task.
    auto partial_thread_pool = shared_->thread_pools[id];

    using Sorter =
            Sorter<ExtendedConfig<iterator, decltype(shared_->classifier.getComparator()),
                                  Config<>, SubThreadPool>>;

    // Create shared data.
    detail::AlignedPtr<typename Sorter::SharedData> partial_shared_ptr(
            Cfg::kDataAlignment, shared_->classifier.getComparator(),
            partial_thread_pool->sync(), partial_thread_pool->numThreads());
    auto& partial_shared = partial_shared_ptr.get();

    // Create local data.
    typename Sorter::BufferStorage partial_buffer_storage(
            partial_thread_pool->numThreads());
    std::unique_ptr<detail::AlignedPtr<typename Sorter::LocalData>[]> partial_local_ptrs(
            new detail::AlignedPtr<
                    typename Sorter::LocalData>[partial_thread_pool->numThreads()]);

    for (int i = 0; i != partial_thread_pool->numThreads(); ++i) {
        partial_local_ptrs[i] = detail::AlignedPtr<typename Sorter::LocalData>(
                Cfg::kDataAlignment, shared_->classifier.getComparator(),
                buffer_storage.forThread(task.root_thread + i));
        partial_shared.local[i] = &partial_local_ptrs[i].get();
    }

    std::pair<std::vector<diff_t>, bool> ret;

    // Execute in parallel
    partial_thread_pool->operator()(
            [&partial_shared, begin, &task, &ret](int partial_id,
                                                  int partial_num_threads) {
                Sorter sorter(*partial_shared.local[partial_id]);
                sorter.setShared(&partial_shared);
                if (partial_id == 0) {
                    ret = sorter.parallelPartitionPrimary(
                            begin + task.begin, begin + task.end, partial_num_threads);
                } else {
                    sorter.parallelPartitionSecondary(begin + task.begin,
                                                      begin + task.end, partial_id,
                                                      partial_num_threads);
                }
            },
            partial_thread_pool->numThreads());

    const auto& offsets = ret.first;
    const auto equal_buckets = ret.second;
    const int num_buckets = offsets.size() - 1;

    // Move my thread pool to the trash as I might create a new one.
    tp_trash.emplace_back(std::move(shared_->thread_pools[id]));

    queueTasks(stripe, id, partial_thread_pool->numThreads(), task.end - task.begin,
               task.begin, offsets.data(), num_buckets, equal_buckets);

    partial_thread_pool->release_threads();
}

/**
 * Entry point to execute a single partitioning recursion step with
 * secondary threads.
 */
template <class Cfg>
void Sorter<Cfg>::parallelPartitionSecondary(const iterator begin, const iterator end,
                                             int id, int num_threads) {
    shared_->local[id] = &local_;
    partition<true>(begin, end, shared_->bucket_start, id, num_threads);
    shared_->sync.barrier();
}

/**
 * Entry point to execute a single partitioning recursion step with
 * the first thread.
 */
template <class Cfg>
std::pair<std::vector<typename Cfg::difference_type>, bool>
Sorter<Cfg>::parallelPartitionPrimary(const iterator begin, const iterator end,
                                      const int num_threads) {

    const auto res = partition<true>(begin, end, shared_->bucket_start, 0, num_threads);
    const int num_buckets = std::get<0>(res);
    const bool equal_buckets = std::get<1>(res);

    std::vector<diff_t> bucket_start(shared_->bucket_start,
                                     shared_->bucket_start + num_buckets + 1);

    shared_->reset();
    shared_->sync.barrier();

    return {bucket_start, equal_buckets};
}

/**
 * Main loop for secondary threads in the parallel algorithm.
 */
template <class Cfg>
void Sorter<Cfg>::parallelSortSecondary(
        const iterator begin, const iterator end, int id, int num_threads,
        BufferStorage& buffer_storage,
        std::vector<std::shared_ptr<SubThreadPool>>& tp_trash) {
    shared_->local[id] = &local_;

    partition<true>(begin, end, shared_->bucket_start, id, num_threads);
    shared_->sync.barrier();

    const auto stripe = ((end - begin) + num_threads - 1) / num_threads;
    processBigTasks(begin, stripe, id, buffer_storage, tp_trash);
    processSmallTasks(begin);
}

/**
 * Main loop for the primary thread in the parallel algorithm.
 */
template <class Cfg>
void Sorter<Cfg>::parallelSortPrimary(
        const iterator begin, const iterator end, const int num_threads,
        BufferStorage& buffer_storage,
        std::vector<std::shared_ptr<SubThreadPool>>& tp_trash) {
    const auto res = partition<true>(begin, end, shared_->bucket_start, 0, num_threads);

    const bool is_last_level = end - begin <= Cfg::kSingleLevelThreshold;
    const auto stripe = ((end - begin) + num_threads - 1) / num_threads;

    if (!is_last_level) {
        const int num_buckets = std::get<0>(res);
        const bool equal_buckets = std::get<1>(res);

        queueTasks(stripe, 0, num_threads, end - begin, begin - begin,
                   shared_->bucket_start, num_buckets, equal_buckets);
    }

    shared_->reset();
    shared_->sync.barrier();

    processBigTasks(begin, stripe, 0, buffer_storage, tp_trash);
    processSmallTasks(begin);
}

}  // namespace detail

/**
 * Reusable parallel sorter.
 */
template <class Cfg>
class ParallelSorter {
    using Sorter = detail::Sorter<Cfg>;
    using iterator = typename Cfg::iterator;

 public:
    /**
     * Construct the sorter. Thread pool may be passed by reference.
     */
    ParallelSorter(typename Cfg::less comp, typename Cfg::ThreadPool thread_pool,
                   bool check_sorted)
        : check_sorted_(check_sorted)
        , thread_pool_(std::forward<typename Cfg::ThreadPool>(thread_pool))
        , shared_ptr_(Cfg::kDataAlignment, std::move(comp), thread_pool_.sync(),
                      thread_pool_.numThreads())
        , buffer_storage_(thread_pool_.numThreads())
        , local_ptrs_(new detail::AlignedPtr<
                      typename Sorter::LocalData>[thread_pool_.numThreads()])
    {
        // Allocate local data and reuse memory of the previous recursion level
        thread_pool_([this](int my_id, int) {
            auto& shared = this->shared_ptr_.get();
            this->local_ptrs_[my_id] = detail::AlignedPtr<typename Sorter::LocalData>(
                    Cfg::kDataAlignment, shared.classifier.getComparator(),
                    buffer_storage_.forThread(my_id));
            shared.local[my_id] = &this->local_ptrs_[my_id].get();
        });
    }

    /**
     * Sort in parallel.
     */
    void operator()(iterator begin, iterator end) {
        // Sort small input sequentially
        const int num_threads = Cfg::numThreadsFor(begin, end, thread_pool_.numThreads());
        if (num_threads < 2 || end - begin <= 2 * Cfg::kBaseCaseSize) {
            Sorter(local_ptrs_[0].get()).sequential(std::move(begin), std::move(end));
            return;
        }

        if (check_sorted_
            && detail::isSorted(begin, end,
                                local_ptrs_[0].get().classifier.getComparator(),
                                thread_pool_)) {
            return;
        }

        // Set up base data before switching to parallel mode
        // auto& shared = shared_ptr_.get();

        // Execute in parallel
        thread_pool_(
                [this, begin, end](int my_id, int num_threads) {
                    std::vector<std::shared_ptr<typename Sorter::SubThreadPool>> tp_trash;
                    auto& shared = this->shared_ptr_.get();
                    Sorter sorter(*shared.local[my_id]);
                    sorter.setShared(&shared);
                    if (my_id == 0)
                        sorter.parallelSortPrimary(begin, end, num_threads,
                                                   buffer_storage_, tp_trash);
                    else
                        sorter.parallelSortSecondary(begin, end, my_id, num_threads,
                                                     buffer_storage_, tp_trash);
                },
                num_threads);
    }

 private:
    const bool check_sorted_;
    typename Cfg::ThreadPool thread_pool_;
    detail::AlignedPtr<typename Sorter::SharedData> shared_ptr_;
    typename Sorter::BufferStorage buffer_storage_;
    std::unique_ptr<detail::AlignedPtr<typename Sorter::LocalData>[]> local_ptrs_;
};

}  // namespace ips4o
#endif  // _REENTRANT
