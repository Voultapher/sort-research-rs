/** @author Sebastian Wild (wild@liverpool.ac.uk) */

#ifndef MERGESORTS_MERGING_MULTIWAY_H
#define MERGESORTS_MERGING_MULTIWAY_H

#include "merging.h"
#include <algorithm>
#include <vector>
#include <limits>
#include <cassert>
#include <tuple>




namespace algorithms {



    /**
     * 4way merge using tournament tree; assumes numeric type to be able to have a sentinel value.
     *
     * Merges runs [l..g1) and [g1..g2) and [g2..g3) and [g3..r) in-place into [l..r)
     * using a buffer at B of length at least r-l+4.
     *
     * This is a good bit slower than merge_4runs_willem which allows the pointer increment to happen further away
     * from the next read access; having a copy (of the pointer) in the tree is much faster.
     */
    template<typename Iter, typename Iter2>
    void merge_4runs_numeric(Iter l, Iter g1, Iter g2, Iter g3, Iter r, Iter2 B) {
        typedef typename std::iterator_traits<Iter>::value_type T;
        // static_assert(std::numeric_limits<T>::is_specialized, "Needs numeric type (for sentinels)");
        const int n = r - l;
        if (COUNT_MERGE_COSTS) totalMergeCosts += n;
        // Copy all runs to B and append a sentinel value after each.
        std::copy(l, g1, B);
        *(B + (g1 - l)) = plus_inf_sentinel<T>();
        std::copy(g1, g2, B + (g1 - l) + 1);
        *(B + (g2 - l) + 1) = plus_inf_sentinel<T>();
        std::copy(g2, g3, B + (g2 - l) + 2);
        *(B + (g3 - l) + 2) = plus_inf_sentinel<T>();
        std::copy(g3, r, B + (g3 - l) + 3);
        *(B + (r - l) + 3) = plus_inf_sentinel<T>();
        if (COUNT_MERGE_COSTS) totalBufferCosts += n+4;
        // initialize pointers to runs in B.
        Iter2 c[4] = {B, B + (g1 - l) + 1, B + (g2 - l) + 2, B + (g3 - l) + 3}; // current element
        // initialize tournament tree
        //       z
        //    /     \
        //   x       y
        //  / \     / \
        // a   b   c   d
        // Internal nodes x,y,z store the current value of the run and whether or not the minimum comes from a/b
        int x, y, z;
        x = *c[0] <= *c[1] ? 0 : 1;
        y = *c[2] <= *c[3] ? 2 : 3;
        z = *c[x] <= *c[y] ? x : y;
        // vacate root into output
        *l++ = *c[z]++;
        for (auto i = 1; i < n; ++i) {
            if (z <= 1) { // min came from 0 or 1, so recompute x.
                x = *c[0] <= *c[1] ? 0 : 1;
            } else { // otherwise min came from c or d, so recompute y.
                y = *c[2] <= *c[3] ? 2 : 3;
            }
            // always recompute z
            z = *c[x] <= *c[y] ? x : y;
            *l++ = *c[z]++;
        }
    }

    /**
     * 4way merge using tournament tree; assumes numeric type to be able to have a sentinel value.
     *
     * Merges runs [l..g1) and [g1..g2) and [g2..g3) and [g3..r) in-place into [l..r)
     * using a buffer at B of length at least r-l+4.
     *
     */
    template<typename Iter, typename Iter2>
    void merge_4runs_numeric_willem(Iter l, Iter g1, Iter g2, Iter g3, Iter r, Iter2 B) {
        typedef typename std::iterator_traits<Iter>::value_type T;
        // static_assert(std::numeric_limits<T>::is_specialized, "Needs numeric type (for sentinels)");
        const int n = r - l;
        if (COUNT_MERGE_COSTS) totalMergeCosts += n;
        // Copy all runs to B and append a sentinel value after each.
        std::copy(l, g1, B);
        *(B + (g1 - l)) = plus_inf_sentinel<T>();
        std::copy(g1, g2, B + (g1 - l) + 1);
        *(B + (g2 - l) + 1) = plus_inf_sentinel<T>();
        std::copy(g2, g3, B + (g2 - l) + 2);
        *(B + (g3 - l) + 2) = plus_inf_sentinel<T>();
        std::copy(g3, r, B + (g3 - l) + 3);
        *(B + (r - l) + 3) = plus_inf_sentinel<T>();
        if (COUNT_MERGE_COSTS) totalBufferCosts += n+4;
        // initialize pointers to runs in B.
        Iter2 a, b, c, d;
        a = B, b = B + (g1 - l) + 1, c = B + (g2 - l) + 2, d = B + (g3 - l) + 3;
        // initialize tournament tree
        //       z
        //    /     \
        //   x       y
        //  / \     / \
        // a   b   c   d
        // Internal nodes x,y,z store the current value of the run and whether or not the minimum comes from a/b
        std::pair<Iter2, bool> x, y, z;
        if (*a <= *b) x = {a++, true}; else x = {b++, true};
        if (*c <= *d) y = {c++, false}; else y = {d++, false};
        if (*x.first <= *y.first) z = x; else z = y;
        // vacate root into output
        *l++ = *(z.first);
        for (auto i = 1; i < n; ++i) {
            if (z.second) { // min came from a or b, so recompute x.
                if (*a <= *b) x = {a++, true}; else x = {b++, true};
            } else { // otherwise min came from c or d, so recompute y.
                if (*c <= *d) y = {c++, false}; else y = {d++, false};
            }
            // always recompute z
            z = *(x.first) <= *(y.first) ? x : y;
            *l++ = *(z.first);
        }
    }

    template <typename Iter, typename IterBuffer>
    void wb_merge4way3(Iter l, Iter g1, Iter g2, Iter g3, Iter r, IterBuffer B) {
        typedef typename std::iterator_traits<Iter>::value_type T;
        // static_assert(std::numeric_limits<T>::is_specialized, "Needs numeric type (for sentinels)");
        std::copy(l, g1, B);
        *(B + (g1 - l)) = plus_inf_sentinel<T>();
        std::copy(g1, g2, B + (g1 - l) + 1);
        *(B + (g2 - l) + 1) = plus_inf_sentinel<T>();
        std::copy(g2, g3, B + (g2 - l) + 2);
        *(B + (g3 - l) + 2) = plus_inf_sentinel<T>();
        std::copy(g3, r, B + (g3 - l) + 3);
        *(B + (r - l) + 3) = plus_inf_sentinel<T>();
        int size = r - l;
        IterBuffer a, b, c, d;
        a = B, b = B + (g1 - l) + 1, c = B + (g2 - l) + 2, d = B + (g3 - l) + 3;
        std::pair<T, int> x, y, z;
        if (*a <= *b) x = {*a++, 1}; else x = {*b++, 1};
        if (*c <= *d) y = {*c++, 2}; else y = {*d++, 2};
        if (x.first <= y.first) z = x; else z = y;
        *l++ = z.first;
        for (auto i = 1; i < size; i++) {
            switch (z.second) {
                case 1:
                    if (*a <= *b) x = {*a++, 1}; else x = {*b++, 1};
                    break;

                case 2:
                    if (*c <= *d) y = {*c++, 2}; else y = {*d++, 2};
                    break;
            }
            if (x.first <= y.first) z = x; else z = y;
            *l++ = z.first;
        }
    }


    /**
     * 4way merge using tournament tree; assumes numeric type to be able to have a sentinel value.
     *
     * Merges runs [l..g1) and [g1..g2) and [g2..g3) and [g3..r) in-place into [l..r)
     * using a buffer at B of length at least r-l+4.
     * (This is basically Willem's code with variables "renamed" to array entries.)
     *
     */
    template<typename Iter, typename Iter2>
    void merge_4runs_numeric_willem_a(Iter l, Iter g1, Iter g2, Iter g3, Iter r, Iter2 B) {
        typedef typename std::iterator_traits<Iter>::value_type T;
        // static_assert(std::numeric_limits<T>::is_specialized, "Needs numeric type (for sentinels)");
        const int n = r - l;
        if (COUNT_MERGE_COSTS) totalMergeCosts += n;
        // Copy all runs to B and append a sentinel value after each.
        std::copy(l, g1, B);
        *(B + (g1 - l)) = plus_inf_sentinel<T>();
        std::copy(g1, g2, B + (g1 - l) + 1);
        *(B + (g2 - l) + 1) = plus_inf_sentinel<T>();
        std::copy(g2, g3, B + (g2 - l) + 2);
        *(B + (g3 - l) + 2) = plus_inf_sentinel<T>();
        std::copy(g3, r, B + (g3 - l) + 3);
        *(B + (r - l) + 3) = plus_inf_sentinel<T>();
        if (COUNT_MERGE_COSTS) totalBufferCosts += n+4;
        // initialize pointers to runs in B.
        Iter2 c[4];
        c[0] = B, c[1] = B + (g1 - l) + 1, c[2] = B + (g2 - l) + 2, c[3] = B + (g3 - l) + 3;
        // initialize tournament tree
        //       z
        //    /     \
        //   x       y
        //  / \     / \
        // c[0]   c[1]   c[2]   c[3]
        // Internal nodes x,y,z store the current value of the run and whether or not the minimum comes from a/c[1]
        std::pair<Iter2, bool> x, y, z;
        if (*c[0] <= *c[1]) x = {c[0]++, true}; else x = {c[1]++, true};
        if (*c[2] <= *c[3]) y = {c[2]++, false}; else y = {c[3]++, false};
        if (*(x.first) <= *(y.first)) z = x; else z = y;
        // vacate root into output
        *l++ = *(z.first);
        for (auto i = 1; i < n; ++i) {
            if (z.second) { // min came from c[0] or c[1], so recompute x.
                if (*c[0] <= *c[1]) x = {c[0]++, true}; else x = {c[1]++, true};
            } else { // min came from c[2] or c[3], so recompute y.
                if (*c[2] <= *c[3]) y = {c[2]++, false}; else y = {c[3]++, false};
            }
            // always recompute z
            if (*(x.first) <= *(y.first)) z = x; else z = y;
            *l++ = *(z.first);
        }
    }


    /**
     * 4way merge using tournament tree; assumes numeric type to be able to have a sentinel value.
     *
     * Merges runs [l..g1) and [g1..g2) and [g2..g3) and [g3..r) in-place into [l..r)
     * using a buffer at B of length at least r-l+4.
     *
     * This is Willem's code with some experimentally determined modifications that improve readability
     * without affecting performance (on g++).
     */
    template<typename Iter, typename Iter2>
    void merge_4runs_numeric_willem_tuned(Iter l, Iter g1, Iter g2, Iter g3, Iter r, Iter2 B) {
        typedef typename std::iterator_traits<Iter>::value_type T;
        // static_assert(std::numeric_limits<T>::is_specialized, "Needs numeric type (for sentinels)");
        const int n = r - l;
        if (COUNT_MERGE_COSTS) totalMergeCosts += n;
        // Copy all runs to B and append a sentinel value after each.
        std::copy(l, g1, B);
        *(B + (g1 - l)) = plus_inf_sentinel<T>();
        std::copy(g1, g2, B + (g1 - l) + 1);
        *(B + (g2 - l) + 1) = plus_inf_sentinel<T>();
        std::copy(g2, g3, B + (g2 - l) + 2);
        *(B + (g3 - l) + 2) = plus_inf_sentinel<T>();
        std::copy(g3, r, B + (g3 - l) + 3);
        *(B + (r - l) + 3) = plus_inf_sentinel<T>();
        if (COUNT_MERGE_COSTS) totalBufferCosts += n+4;
        // initialize pointers to runs in B.
        Iter2 c[4];
        c[0] = B, c[1] = B + (g1 - l) + 1, c[2] = B + (g2 - l) + 2, c[3] = B + (g3 - l) + 3;
        // initialize tournament tree
        //       z
        //    /     \
        //   x       y
        //  / \     / \
        // c[0]   c[1]   c[2]   c[3]
        // Internal nodes x,y,z store the current value of the run and whether or not the minimum comes from a/c[1]
        Iter2 x, y;
        std::pair<Iter2, bool> z;
        if (*c[0] <= *c[1]) x = c[0]++; else x = c[1]++;
        if (*c[2] <= *c[3]) y = c[2]++; else y = c[3]++;
        if (*x <= *y) z = {x, true}; else z = {y, false};
        // vacate root into output
        *l++ = *(z.first);
        for (auto i = 1; i < n; ++i) {
            if (z.second) { // min came from c[0] or c[1], so recompute x.
                if (*c[0] <= *c[1]) x = c[0]++; else x = c[1]++;
            } else { // min came from c[2] or c[3], so recompute y.
                if (*c[2] <= *c[3]) y = c[2]++; else y = c[3]++;
            }
            // always recompute z
            if (*x <= *y) z = {x, true}; else z = {y, false};
            *l++ = *(z.first);
        }
    }

    /**
     * 4way merge with plain min computation; assumes numeric type to be able to have a sentinel value.
     *
     * Merges runs [l..g1) and [g1..g2) and [g2..g3) and [g3..r) in-place into [l..r)
     * using a buffer at B of length at least r-l+4.
     *
     * ~40% slower than tournament tree version!
     *
     */
    template<typename Iter, typename Iter2>
    void merge_4runs_numeric_plain_min(Iter l, Iter g1, Iter g2, Iter g3, Iter r, Iter2 B) {
        typedef typename std::iterator_traits<Iter>::value_type T;
        // static_assert(std::numeric_limits<T>::is_specialized, "Needs numeric type (for sentinels)");
        const int n = r - l;
        if (COUNT_MERGE_COSTS) totalMergeCosts += n;
        // Copy all runs to B and append a sentinel value after each.
        std::copy(l, g1, B);
        *(B + (g1 - l)) = plus_inf_sentinel<T>();
        std::copy(g1, g2, B + (g1 - l) + 1);
        *(B + (g2 - l) + 1) = plus_inf_sentinel<T>();
        std::copy(g2, g3, B + (g2 - l) + 2);
        *(B + (g3 - l) + 2) = plus_inf_sentinel<T>();
        std::copy(g3, r, B + (g3 - l) + 3);
        *(B + (r - l) + 3) = plus_inf_sentinel<T>();
        if (COUNT_MERGE_COSTS) totalBufferCosts += n+4;
        // initialize pointers to runs in B.
        Iter2 c[4];
        c[0] = B, c[1] = B + (g1 - l) + 1, c[2] = B + (g2 - l) + 2, c[3] = B + (g3 - l) + 3;
        for (auto i = 0; i < n; ++i) {
            auto argmin01 = (*c[0] <= *c[1]) ? 0 : 1;
            auto argmin23 = (*c[2] <= *c[3]) ? 2 : 3;
            auto argmin = (*c[argmin01] <= *c[argmin23]) ? argmin01 : argmin23;
            *l++ = *c[argmin]++;
        }
    }

    /**
     * 4way merge using tournament tree; does not require a sentinel value.
     *
     * Merges runs [l..g1) and [g1..g2) and [g2..g3) and [g3..r) in-place into [l..r)
     * using a buffer at B of length at least r-l+1.
     *
     */
    template<typename Iter, typename Iter2>
    void merge_4runs_indices(Iter l, Iter g1, Iter g2, Iter g3, Iter r, Iter2 B) {
        typedef typename std::iterator_traits<Iter>::value_type T;
        const int n = r - l;
        if (COUNT_MERGE_COSTS) totalMergeCosts += n;
        // Copy all runs to B
        std::copy(l, g1, B);
        std::copy(g1, g2, B + (g1 - l));
        std::copy(g2, g3, B + (g2 - l));
        std::copy(g3, r, B + (g3 - l));
        if (COUNT_MERGE_COSTS) totalBufferCosts += n;
        *(B+n) = *B; // sentinel value so that accesses to endpoints don't fail
        // initialize pointers to runs in B.
        Iter2 c[4] = {B, B + (g1 - l), B + (g2 - l), B + (g3 - l)}; // current element
        const Iter2 e[4] = {B + (g1 - l), B + (g2 - l), B + (g3 - l), B + n}; // endpoints (for convenience)
        // initialize tournament tree
        //       z
        //    /     \
        //   x       y
        //  / \     / \
        // 0   1   2   3
        // Internal nodes x,y,z store the run id
        int x, y, z;
        x = *c[0] <= *c[1] ? 0 : 1;
        if (c[x] == e[x]) x = 1-x; // if empty, use other run
        y = *c[2] <= *c[3] ? 2 : 3;
        if (c[y] == e[y]) y = 5-y; // if empty, use other run
        z = *c[x] <= *c[y] ? x : y;
        if (c[z] == e[z]) z = z <= 1 ? y : x; // if empty, use other child
        for (auto i = 0; i < n; ++i) {
            *l++ = *c[z]++; // vacate root to output
            if (z <= 1) { // min came from 0 or 1, so recompute x.
                x = *c[0] <= *c[1] ? 0 : 1;
                if (c[x] == e[x]) x = 1-x; // if empty, use other run
            } else { // otherwise min came from c or d, so recompute y.
                y = *c[2] <= *c[3] ? 2 : 3;
                if (c[y] == e[y]) y = 5-y; // if empty, use other run
            }
            // always recompute z
            z = *c[x] <= *c[y] ? x : y;
            if (c[z] == e[z]) z = z <= 1 ? y : x; // if empty, use other child
        }
    }




    /** Helper methods for merge_4runs_by_stages_split */
    namespace private_stages_split_ {
        template<typename Iter2>
        struct tournament_tree_node {
            Iter2 it;
            bool fromRun0Or1;
        };

        enum number_runs {
            TWO = 2,
            THREE = 3,
            FOUR = 4
        };

        template<typename Iter2>
        void detect_and_remove_empty_runs(std::vector<Iter2> & c, std::vector<Iter2> & e) {
            int nRuns = c.size();
            long safe;
            while (true) {
                std::vector<long> nn(nRuns);
                for (int i = 0; i < nRuns; ++i) nn[i] = e[i] - c[i];
                safe = *(std::min_element(nn.begin(), nn.end()));
                if (safe > 0) return;
                int i = std::find(nn.begin(), nn.end(), 0) - nn.begin();
                c.erase(c.begin() + i);
                e.erase(e.begin() + i);
                --nRuns;
            }
        }

        template<typename Iter2, number_runs nRuns>
        long compute_safe(std::vector<Iter2> & c, std::vector<Iter2> & e, std::vector<long> & nn) {
            assert(nRuns == c.size() && nRuns == e.size() && nRuns == nn.size());
            for (int i = 0; i < nRuns; ++i) nn[i] = e[i] - c[i];
            long safe = *(std::min_element(nn.begin(), nn.end()));
            assert (safe >= 0);
            return safe;
        }

        template<typename Iter2, number_runs nRuns>
        void initialize_tournament_tree(std::vector<Iter2> &c, std::vector<Iter2> &e,
                                       std::array<tournament_tree_node<Iter2>, 3> &N) {
            assert(nRuns == 3 || nRuns == 4);
            assert(nRuns == c.size() && nRuns == e.size());
            // tourament tree:
            //      N[0]
            //    /     \
            //  N[1]    N[2]
            //  / \     / \
            // 0   1   2   3
            if (*c[0] <= *c[1]) N[1] = {c[0]++, true}; else N[1] = {c[1]++, true};
            if (nRuns == 4)
                if (*c[2] <= *c[3]) N[2] = {c[2]++, false}; else N[2] = {c[3]++, false};
            else
                N[2] = {c[2]++, false};
            N[0] = *(N[1].it) <= *(N[2].it) ? N[1] : N[2];
        }
        template<typename Iter2, number_runs nRuns>
        void update_tournament_tree(std::vector<Iter2> &c, std::vector<Iter2> &e,
                                       std::array<tournament_tree_node<Iter2>, 3> &N) {
            assert(nRuns == 3 || nRuns == 4);
            assert(nRuns == c.size() && nRuns == e.size());
            // tourament tree:
            //      N[0]
            //    /     \
            //  N[1]    N[2]
            //  / \     / \
            // 0   1   2   3
            if (N[0].fromRun0Or1) {
                if (*c[0] <= *c[1]) N[1] = {c[0]++, true}; else N[1] = {c[1]++, true};
            } else { // otherwise min came from c[2] or c[3], so recompute y.
                if (nRuns == 4)
                    if (*c[2] <= *c[3]) N[2] = {c[2]++, false}; else N[2] = {c[3]++, false};
                else
                    N[2] = {c[2]++, false};
            }
            // always recompute z
            N[0] = *(N[1].it) <= *(N[2].it) ? N[1] : N[2];
        }

        template<typename Iter2, number_runs nRuns>
        bool rollback_tournament_tree(std::vector<Iter2> &c, std::vector<Iter2> &e,
                                     std::array<tournament_tree_node<Iter2>, 3> &N,
                                     std::vector<long> &nn) {
            assert(nRuns == c.size() && nRuns == e.size() && nRuns == nn.size());
            auto other = N[0].fromRun0Or1 ? N[2] : N[1];
            // roll back into 'its' run
            int rollbacks = 0;
#pragma GCC unroll 4
            for (auto i = 0; i < nRuns; ++i)
                if (c[i] - 1 == other.it) {
                    --c[i], ++nn[i], ++rollbacks;
                    break;
                }
            assert(rollbacks == 1);
            int i = std::find(nn.begin(), nn.end(), 0) - nn.begin();
            if (i == nRuns) {
                // rolled back into run that got empty; nasty special case.
                // But we made progress in the root, so just continue one more round with same nRuns.
                // need to rebuild the tree for that
                initialize_tournament_tree<Iter2, nRuns>(c, e, N);
                return false;
            } else {
                c.erase(c.begin() + i);
                e.erase(e.begin() + i);
                return true;
            }
        }

        template<typename Iter, typename Iter2, number_runs nRuns>
        bool do_merge_runs(Iter & l, Iter const r, std::vector<Iter2> &c, std::vector<Iter2> &e) {
            static_assert(nRuns == TWO || nRuns == THREE || nRuns == FOUR, "nRuns must be 2, 3 or 4");
            if (nRuns == TWO) {
                // simple two-way merge
                while (c[0] < e[0] && c[1] < e[1])
                    *l++ = *c[0] <= *c[1] ? *c[0]++ : *c[1]++;
                while (c[0] < e[0]) *l++ = *c[0]++;
                while (c[1] < e[1]) *l++ = *c[1]++;
                return true;
            } else {
                assert(nRuns == THREE || nRuns == FOUR && "nRuns must be 3 or 4");
                // use tournament tree
                std::array<tournament_tree_node<Iter2>, 3> N;
                initialize_tournament_tree<Iter2, nRuns>(c, e, N);
                std::vector<long> nn(nRuns); // run sizes
                while (l < r) {
                    long safe = compute_safe<Iter2, nRuns>(c, e, nn);
                    if (safe > 0) {
                        for (; safe > 0; --safe) {
                            *l++ = *(N[0].it); // output root
                            update_tournament_tree<Iter2, nRuns>(c, e, N);
                        }
                    } else {
                        // one run is exhausted; need to handle elements in the tree
                        *l++ = *(N[0].it); // easy for the root (guaranteed min)
                        // rollback other element into its run
                        if (rollback_tournament_tree<Iter2, nRuns>(c, e, N, nn))
                            // occasionally, we rollback into an empty run and have to keep going;
                            // otherwise, terminate loop.
                            break;
                    }
                }
                return false;
            }
        }

    }

    /**
     * 4way merge using tournament tree; does not require a sentinel value.
     *
     * Merges runs [l..g1) and [g1..g2) and [g2..g3) and [g3..r) in-place into [l..r)
     * using a buffer at B of length at least r-l+1.
     *
     */
    template<typename Iter, typename Iter2>
    void merge_4runs_by_stages_split(Iter l0, Iter g1, Iter g2, Iter g3, Iter r, Iter2 B) {
        using namespace private_stages_split_;
        // Step 0: copy runs to buffer and prepare iterators
        Iter l = l0;
        const auto n = r - l;
        if (COUNT_MERGE_COSTS) totalMergeCosts += n;
        // Copy all runs to B
        std::copy(l, g1, B);
        std::copy(g1, g2, B + (g1 - l));
        std::copy(g2, g3, B + (g2 - l));
        std::copy(g3, r, B + (g3 - l));
        if (COUNT_MERGE_COSTS) totalBufferCosts += n;
        *(B+n) = *(B+n-1); // sentinel value so that accesses to endpoints don't fail
        std::vector<Iter2> c {B, B + (g1 - l), B + (g2 - l), B + (g3 - l)}; // current element
        std::vector<Iter2> e {B + (g1 - l), B + (g2 - l), B + (g3 - l), B + n}; // endpoints (for convenience)

        detect_and_remove_empty_runs(c, e);
        while (l < r) {
            switch (c.size()) {
                case 4:
                    if (do_merge_runs<Iter, Iter2, FOUR>(l, r, c, e)) break;
                case 3:
                    if (do_merge_runs<Iter, Iter2, THREE>(l, r, c, e)) break;
                case 2:
                    if (do_merge_runs<Iter, Iter2, TWO>(l, r, c, e)) break;
                case 1:
                    return;
                default:
                    assert(false);
                    __builtin_unreachable();
            };
        }
    }

    /**
     * 4way merge using tournament tree; does not require a sentinel value.
     *
     * Merges runs [l..g1) and [g1..g2) and [g2..g3) and [g3..r) in-place into [l..r)
     * using a buffer at B of length at least r-l+1.
     *
     * OBSOLETE VERSION OF THE ABOVE FUNCTION.
     * Kept for comparison purposes, but should eventually not be needed.
     *
     */
    template<typename Iter, typename Iter2>
    void merge_4runs_by_stages(Iter l0, Iter g1, Iter g2, Iter g3, Iter r, Iter2 B) {
        Iter l = l0;
        const auto n = r - l;
        if (COUNT_MERGE_COSTS) totalMergeCosts += n;
        // Copy all runs to B
        std::copy(l, g1, B);
        std::copy(g1, g2, B + (g1 - l));
        std::copy(g2, g3, B + (g2 - l));
        std::copy(g3, r, B + (g3 - l));
        if (COUNT_MERGE_COSTS) totalBufferCosts += n;
        *(B+n) = *(B+n-1); // sentinel value so that accesses to endpoints don't fail

        long todo = n; // number of elements to output

        // initialize pointers to runs in B.
        std::vector<Iter2> c {B, B + (g1 - l), B + (g2 - l), B + (g3 - l)}; // current element
        std::vector<Iter2> e {B + (g1 - l), B + (g2 - l), B + (g3 - l), B + n}; // endpoints (for convenience)
        {// Check for initially emtpy runs
            int nRuns = 4;
            long safe;
            while (true) {
                std::vector<long> nn(nRuns);
                for (int i = 0; i < nRuns; ++i) nn[i] = e[i] - c[i];
                safe = *(std::min_element(nn.begin(), nn.end()));
                if (safe > 0) break;
                int i = std::find(nn.begin(), nn.end(), 0) - nn.begin();
                c.erase(c.begin() + i);
                e.erase(e.begin() + i);
                --nRuns;
            }
            if (nRuns == 3) goto STAGE_3RUNS;
            if (nRuns == 2) goto STAGE_2RUNS;
            if (nRuns == 1) return;
        }
        { // STAGE 1: 4way
            const int nRuns = 4;
            // initialize tournament tree
            //       z
            //    /     \
            //   x       y
            //  / \     / \
            // 0   1   2   3
            // Internal nodes x,y,z store the current value of the run and whether or not the minimum comes from a/c[1]
            std::pair<Iter2, bool> x, y, z;
            x = {*c[0] <= *c[1] ? c[0]++ : c[1]++, true};
            y = {*c[2] <= *c[3] ? c[2]++ : c[3]++, false};
            z = *(x.first) <= *(y.first) ? x : y;
            while (todo > 0) {
                std::vector<long> nn(nRuns);
#pragma GCC unroll 4
                for (int i = 0; i < nRuns; ++i) nn[i] = e[i] - c[i];
                auto safe = *(std::min_element(nn.begin(), nn.end()));
                assert (safe >= 0);
                if (safe == 0) {
                    // one run is exhausted, so need to eliminate a run from the tree
                    // but: need to output the elements currently in the tree first
                    *l++ = *(z.first); // easy for the root (guaranteed min)
                    --todo;
                    auto other = z.second ? y : x;
                    // outputting other now gets absurdly messy; instead roll back into its run
                    int rollbacks = 0;
#pragma GCC unroll 4
                    for (auto i = 0; i < nRuns; ++i)
                        if (c[i] - 1 == other.first) {
                            --c[i], ++nn[i], ++rollbacks;
                            break;
                        }
                    assert(rollbacks == 1);
                    int i = std::find(nn.begin(), nn.end(), 0) - nn.begin();
                    if (i == nRuns) {
                        // rolled back into run that got empty; nasty special case.
                        // But we made progress in the root, so just go one more round.
                        // need to rebuild the tree
                        x = {*c[0] <= *c[1] ? c[0]++ : c[1]++, true};
                        y = {*c[2] <= *c[3] ? c[2]++ : c[3]++, false};
                        z = *(x.first) <= *(y.first) ? x : y;
                        continue;
                    }
                    c.erase(c.begin() + i);
                    e.erase(e.begin() + i);
                    break; // done with this stage
                } else {
                    todo -= safe;
                    for (; safe > 0; --safe) {
                        *l++ = *(z.first);
                        if (z.second) { // min came from c[0] or c[1], so recompute x.
                            x = {*c[0] <= *c[1] ? c[0]++ : c[1]++, true};
                        } else { // otherwise min came from c[2] or c[3], so recompute y.
                            y = {*c[2] <= *c[3] ? c[2]++ : c[3]++, false};
                        }
                        // always recompute z
                        z = *(x.first) <= *(y.first) ? x : y;
                    }
                }
            }
        }
        STAGE_3RUNS:
        { // STAGE 2: 3way
            const int nRuns = 3;
            // initialize tournament tree
            //       z
            //    /     \
            //   x       y
            //  / \     /
            // 0   1   2
            std::pair<Iter2, bool> x, y, z;
            x = {*c[0] <= *c[1] ? c[0]++ : c[1]++, true};
            y = {c[2]++, false};
            z = *(x.first) <= *(y.first) ? x : y;
            while (todo > 0) {
                std::vector<long> nn(nRuns);
#pragma GCC unroll 4
                for (int i = 0; i < nRuns; ++i) nn[i] = e[i] - c[i];
                auto safe = *(std::min_element(nn.begin(), nn.end()));
                assert (safe >= 0);
                if (safe == 0) {
                    // as above, eliminate one run after tree cleanup
                    *l++ = *(z.first);
                    --todo;
                    auto other = z.second ? y : x;
                    // outputting other now gets absurdly messy; instead roll back into its run
                    int rollbacks = 0;
#pragma GCC unroll 4
                    for (auto i = 0; i < nRuns; ++i)
                        if (c[i] - 1 == other.first) {
                            --c[i], ++nn[i], ++rollbacks;
                            break;
                        }
                    assert(rollbacks == 1);
                    int i = std::find(nn.begin(), nn.end(), 0) - nn.begin();
                    if (i == nRuns) {
                        // rolled back into run that got empty; nasty special case.
                        // But we made progress in the root, so just go one more round.
                        // need to rebuild the tree
                        x = {*c[0] <= *c[1] ? c[0]++ : c[1]++, true};
                        y = {c[2]++, false};
                        z = *(x.first) <= *(y.first) ? x : y;
                        continue;
                    }
                    c.erase(c.begin() + i);
                    e.erase(e.begin() + i);
                    break; // done with this stage
                } else {
                    todo -= safe;
                    for (; safe > 0; --safe) {
                        *l++ = *(z.first);
                        if (z.second) { // min came from c[0] or c[1], so recompute x.
                            x = {*c[0] <= *c[1] ? c[0]++ : c[1]++, true};
                        } else { // otherwise min came from c[2] or c[3], so recompute y.
                            y = {c[2]++, false};
                        }
                        // always recompute z
                        z = *(x.first) <= *(y.first) ? x : y;
                    }
                }
            }
        }
        STAGE_2RUNS:
        { // STAGE 3: 2way
//            assert(nRuns == 2);
            // simple two-way merge
            while (c[0] < e[0] && c[1] < e[1])
                *l++ = *c[0] <= *c[1] ? *c[0]++ : *c[1]++;
            while (c[0] < e[0]) *l++ = *c[0]++;
            while (c[1] < e[1]) *l++ = *c[1]++;
        }
    }


    /** Helper methods for merge_4runs_explicit_nodes */
    namespace private_explicit_nodes_ {
        template<typename Iter2>
        struct tournament_tree_node {
            bool valid;
            Iter2 it;
            int runId;
        };

        template<typename Iter2, int child1, int child2>
        inline tournament_tree_node<Iter2> updateTournamentNode(tournament_tree_node<Iter2> *N) {
            static_assert(child1 < child2);
            bool useChild1 = N[child1].valid != N[child2].valid
                             ? N[child1].valid
                             : *(N[child1].it) <= *(N[child2].it);
            return useChild1? N[child1] : N[child2];
        }
    }


    /**
     * 4way merge using tournament tree; does not require a sentinel value.
     *
     * Merges runs [l..g1) and [g1..g2) and [g2..g3) and [g3..r) in-place into [l..r)
     * using a buffer at B of length at least r-l+1.
     *
     * VERY SLOW
     */
    template<typename Iter, typename Iter2>
    void merge_4runs_explicit_nodes(Iter l, Iter g1, Iter g2, Iter g3, Iter r, Iter2 B) {
        using namespace private_explicit_nodes_;
        typedef typename std::iterator_traits<Iter>::value_type T;
        const int n = r - l;
        if (COUNT_MERGE_COSTS) totalMergeCosts += n;
        // Copy all runs to B
        std::copy(l, g1, B);
        std::copy(g1, g2, B + (g1 - l));
        std::copy(g2, g3, B + (g2 - l));
        std::copy(g3, r, B + (g3 - l));
        if (COUNT_MERGE_COSTS) totalBufferCosts += n;
        *(B+n) = *(B+n-1); // sentinel value so that accesses to endpoints don't fail
        // initialize pointers to runs in B.
        Iter2 c[4] = {B, B + (g1 - l), B + (g2 - l), B + (g3 - l)}; // current element
        const Iter2 e[4] = {B + (g1 - l), B + (g2 - l), B + (g3 - l), B + n}; // endpoints (for convenience)
        // initialize tournament tree
        //       6
        //    /     \
        //   4       5
        //  / \     / \
        // 0   1   2   3
        // Nodes store <valid?, iter, from 0/1?>
        // TODO This seems very slow; try to get rid of the objects for leaves.
        tournament_tree_node<Iter2> N[7]; // todo: try to use length instead of valid?
        for (auto i = 0; i < 4; ++i)
            N[i] = {c[i] < e[i], c[i]++, i};
        N[4] = updateTournamentNode<Iter2,0,1>(N);
        N[5] = updateTournamentNode<Iter2,2,3>(N);
        N[6] = updateTournamentNode<Iter2,4,5>(N);
        for (auto i = 0; i < n; ++i) {
            *l++ = *(N[6].it); // copy root to output
            int id = N[6].runId;
            N[id] = {c[id] < e[id], c[id]++, id};
            if (id < 2) { // min came from 0 or 1, so recompute 4.
                N[4] = updateTournamentNode<Iter2,0,1>(N);
            } else { // otherwise min came from 2 or 3, so recompute 5.
                N[5] = updateTournamentNode<Iter2,2,3>(N);
            }
            // always recompute 6
            N[6] = updateTournamentNode<Iter2,4,5>(N);
        }
    }



    /**
     * Different choices for 4way merging.
     */
    enum merging4way_methods {
        FOR_NUMERIC_DATA /** @deprecated */,
        FOR_NUMERIC_DATA_PLAIN_MIN  /** @deprecated */,
        WILLEM  /** @deprecated */,
        WILLEM_TUNED,
        WILLEM_VALUES,
        WILLEM_WITH_INDICES,
        GENERAL_NO_SENTINELS  /** @deprecated */,
        GENERAL_INDICES  /** @deprecated */,
        GENERAL_BY_STAGES,
        GENERAL_BY_STAGES_SPLIT
    };

    std::string to_string(merging4way_methods implementation) {
        switch (implementation) {
            case FOR_NUMERIC_DATA:
                return "FOR_NUMERIC_DATA";
            case GENERAL_NO_SENTINELS:
                return "GENERAL_NO_SENTINELS";
            case WILLEM:
                return "WILLEM";
            case WILLEM_VALUES:
                return "WILLEM_VALUES";
            case WILLEM_TUNED:
                return "WILLEM_TUNED";
            case WILLEM_WITH_INDICES:
                return "WILLEM_WITH_INDICES";
            case GENERAL_INDICES:
                return "GENERAL_INDICES";
            case GENERAL_BY_STAGES:
                return "GENERAL_BY_STAGES";
            case FOR_NUMERIC_DATA_PLAIN_MIN:
                return "FOR_NUMERIC_DATA_PLAIN_MIN";
            case GENERAL_BY_STAGES_SPLIT:
                return "GENERAL_BY_STAGES_SPLIT";
        }
        assert(false);
        __builtin_unreachable();
    };

    /**
     * Merges runs [l..g1) and [g1..g2) and [g2..g3) and [g3..r) in-place into [l..r)
     * using a buffer B.
     */
    template<merging4way_methods mergingMethod, typename Iter, typename Iter2>
    void merge_4runs(Iter l, Iter g1, Iter g2, Iter g3, Iter r, Iter2 B) {
        switch (mergingMethod) {
            case merging4way_methods::FOR_NUMERIC_DATA:
                return merge_4runs_numeric(l, g1, g2, g3, r, B);
            case merging4way_methods::GENERAL_NO_SENTINELS:
                return merge_4runs_explicit_nodes(l, g1, g2, g3, r, B);
            case merging4way_methods::WILLEM:
                return merge_4runs_numeric_willem(l, g1, g2, g3, r, B);
            case merging4way_methods::WILLEM_TUNED:
                return merge_4runs_numeric_willem_tuned(l, g1, g2, g3, r, B);
            case merging4way_methods::WILLEM_VALUES:
                return wb_merge4way3(l, g1, g2, g3, r, B);
            case merging4way_methods::WILLEM_WITH_INDICES:
                return merge_4runs_numeric_willem_a(l, g1, g2, g3, r, B);
            case merging4way_methods::GENERAL_INDICES:
                return merge_4runs_indices(l, g1, g2, g3, r, B);
            case merging4way_methods::GENERAL_BY_STAGES:
                return merge_4runs_by_stages(l, g1, g2, g3, r, B);
            case merging4way_methods::FOR_NUMERIC_DATA_PLAIN_MIN:
                return merge_4runs_numeric_plain_min(l, g1, g2, g3, r, B);
            case merging4way_methods::GENERAL_BY_STAGES_SPLIT:
                return merge_4runs_by_stages_split(l, g1, g2, g3, r, B);
            default:
                assert(false);
                __builtin_unreachable();
        }

    }

}


#endif //MERGESORTS_MERGING_MULTIWAY_H

