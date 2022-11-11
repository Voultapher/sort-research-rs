/** @author William Cawley Gelling */

#ifndef MERGESORTS_MERGING_3WAY_H
#define MERGESORTS_MERGING_3WAY_H

#include "merging.h"
#include <algorithm>
#include <vector>
#include <limits>
#include <cassert>
#include <tuple>
#include "merging_multiway.h"


namespace algorithms {

        
    /**
     * 3way merge using tournament tree; assumes numeric type to be able to have a sentinel value.
     *
     * Merges runs [l..g1) and [g1..g2) and [g2..r) in-place into [l..r)
     * using a buffer at B of length at least r-l+3.
     */
    template<typename Iter, typename Iter2>
    void merge_3runs_numeric_willem_a(Iter l, Iter g1, Iter g2, Iter r, Iter2 B) {
        assert(l <= g1 && g1 <= g2 && g2 <= r);
        typedef typename std::iterator_traits<Iter>::value_type T;
        // static_assert(std::numeric_limits<T>::is_specialized, "Needs numeric type (for sentinels)");
        const int n = r - l;
        if (COUNT_MERGE_COSTS) totalMergeCosts += n;
        // Copy all runs to B and append a sentinel value after each.
        std::copy(l, g1, B);
        *(B + (g1 - l)) = plus_inf_sentinel<T>();
        std::copy(g1, g2, B + (g1 - l) + 1);
        *(B + (g2 - l) + 1) = plus_inf_sentinel<T>();
        std::copy(g2, r, B + (g2 - l) + 2);
        *(B + (r - l) + 2) = plus_inf_sentinel<T>();
        if (COUNT_MERGE_COSTS) totalBufferCosts += n+3;
        // initialize pointers to runs in B.
        Iter2 c[3];
        c[0] = B, c[1] = B + (g1 - l) + 1, c[2] = B + (g2 - l) + 2;
        // initialize tournament tree
        //          z
        //       /     \
        //      x       y = c[2]
        //     / \
        // c[0]   c[1]   
        // Internal nodes x, y, z store the current value of the run and whether or not the minimum comes from a/c[1]
        std::pair<Iter2, bool> x, y, z;
        if (*c[0] <= *c[1]) x = {c[0]++, true}; else x = {c[1]++, true};
        y = { c[2]++, false };
        if (*(x.first) <= *(y.first)) z = x; else z = y;
        // vacate root into output
        *l++ = *(z.first);
        for (auto i = 1; i < n; ++i) {
            if (z.second) { // min came from c[0] or c[1], so recompute x.
                if (*c[0] <= *c[1]) x = {c[0]++, true}; else x = {c[1]++, true};
            } else { // if it came from y, recompute y and increase the pointer
                y = { c[2]++, false };
            }
            // always recompute z
            if (*(x.first) <= *(y.first)) z = x; else z = y;
            *l++ = *(z.first);
        }
    }



    /**
     * 3way merge using tournament tree; assumes numeric type to be able to have a sentinel value.
     *
     * Merges runs [l..g1) and [g1..g2) and [g2..r) in-place into [l..r)
     * using a buffer at B of length at least r-l+4.
     *
     * This is Willem's code with some experimentally determined modifications that improve readability
     * without affecting performance (on g++).
     */
    template<typename Iter, typename Iter2>
    void merge_3runs_numeric_willem_tuned(Iter l, Iter g1, Iter g2, Iter r, Iter2 B) {
        typedef typename std::iterator_traits<Iter>::value_type T;
        // static_assert(std::numeric_limits<T>::is_specialized, "Needs numeric type (for sentinels)");
        const int n = r - l;
        if (COUNT_MERGE_COSTS) totalMergeCosts += n;
        // Copy all runs to B and append a sentinel value after each.
        std::copy(l, g1, B);
        *(B + (g1 - l)) = plus_inf_sentinel<T>();
        std::copy(g1, g2, B + (g1 - l) + 1);
        *(B + (g2 - l) + 1) = plus_inf_sentinel<T>();
        std::copy(g2, r, B + (g2 - l) + 2);
        *(B + (r - l) + 2) = plus_inf_sentinel<T>();
        if (COUNT_MERGE_COSTS) totalBufferCosts += n+3;

        // initialize pointers to runs in B.
        Iter2 c[3];
        c[0] = B, c[1] = B + (g1 - l) + 1, c[2] = B + (g2 - l) + 2;
        // initialize tournament tree
        //          z
        //       /     \
        //      x       y = c[2]
        //     / \
        // c[0]   c[1]   
        // Internal nodes x,y,z store the current value of the run and whether or not the minimum comes from a/c[1]
        Iter2 x, y;
        std::pair<Iter2, bool> z;
        if (*c[0] <= *c[1]) x = c[0]++; else x = c[1]++;
        y = c[2]++;
        if (*x <= *y) z = {x, true}; else z = {y, false};
        // vacate root into output
        *l++ = *(z.first);
        for (auto i = 1; i < n; ++i) {
            if (z.second) { // min came from c[0] or c[1], so recompute x.
                if (*c[0] <= *c[1]) x = c[0]++; else x = c[1]++;
            } else { // min came from c[2] or c[3], so recompute y.
                y = c[2]++;
            }
            // always recompute z
            if (*x <= *y) z = {x, true}; else z = {y, false};
            *l++ = *(z.first);
        }
    }

    /** Helper methods for merge_4runs_by_stages_split */
    namespace private_stages_split_ {

        template<typename Iter2, number_runs nRuns>
        void initialize_tournament_tree3(std::vector<Iter2> &c, std::vector<Iter2> &e,
                                       std::array<tournament_tree_node<Iter2>, 3> &N) {
            assert(nRuns == 3);
            assert(nRuns == c.size() && nRuns == e.size());
            // tourament tree:
            //      N[0]
            //    /     \
            //  N[1]    N[2]
            //  / \     / \
            // 0   1   2   3
            if (*c[0] <= *c[1]) N[1] = {c[0]++, true}; else N[1] = {c[1]++, true};
            N[2] = {c[2]++, false};
            N[0] = *(N[1].it) <= *(N[2].it) ? N[1] : N[2];
        }

        template<typename Iter2, number_runs nRuns>
        void update_tournament_tree3(std::vector<Iter2> &c, std::vector<Iter2> &e,
                                       std::array<tournament_tree_node<Iter2>, 3> &N) {
            assert(nRuns == 3 );
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
                N[2] = {c[2]++, false};
            }
            // always recompute z
            N[0] = *(N[1].it) <= *(N[2].it) ? N[1] : N[2];
        }


        template<typename Iter, typename Iter2, number_runs nRuns>
        bool do_merge_runs3(Iter & l, Iter const r, std::vector<Iter2> &c, std::vector<Iter2> &e) {
            static_assert(nRuns == TWO || nRuns == THREE,  "nRuns must be 2, 3 or 4");
            if (nRuns == TWO) {
                // simply twoway merge
                while (c[0] < e[0] && c[1] < e[1])
                    *l++ = *c[0] <= *c[1] ? *c[0]++ : *c[1]++;
                while (c[0] < e[0]) *l++ = *c[0]++;
                while (c[1] < e[1]) *l++ = *c[1]++;
                return true;
            } else {
                assert(nRuns == THREE && "nRuns must be 3");
                // use tournament tree
                std::array<tournament_tree_node<Iter2>, 3> N;
                initialize_tournament_tree3<Iter2, nRuns>(c, e, N);
                std::vector<long> nn(nRuns); // run sizes
                while (l < r) {
                    long safe = compute_safe<Iter2, nRuns>(c, e, nn);
                    if (safe > 0) {
                        for (; safe > 0; --safe) {
                            *l++ = *(N[0].it); // output root
                            update_tournament_tree3<Iter2, nRuns>(c, e, N);
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
    void merge_3runs_by_stages_split(Iter l0, Iter g1, Iter g2, Iter r, Iter2 B) {
        using namespace private_stages_split_;
        // Step 0: copy runs to buffer and prepare iterators
        Iter l = l0;
        const auto n = r - l;
        if (COUNT_MERGE_COSTS) totalMergeCosts += n;
        // Copy all runs to B
        std::copy(l, g1, B);
        std::copy(g1, g2, B + (g1 - l));
        std::copy(g2, r, B + (g2 - l));
        if (COUNT_MERGE_COSTS) totalBufferCosts += n;
        *(B+n) = *(B+n-1); // sentinel value so that accesses to endpoints don't fail
        std::vector<Iter2> c {B, B + (g1 - l), B + (g2 - l)}; // current element
        std::vector<Iter2> e {B + (g1 - l), B + (g2 - l), B + n}; // endpoints (for convenience)

        detect_and_remove_empty_runs(c, e);
        while (l < r) {
            switch (c.size()) {
                case 3:
                    if (do_merge_runs3<Iter, Iter2, THREE>(l, r, c, e)) break;
                case 2:
                    if (do_merge_runs3<Iter, Iter2, TWO>(l, r, c, e)) break;
                case 1:
                    return;
                default:
                    assert(false);
                    __builtin_unreachable();
            };
        }
    }






    /** Document which methods have custom 3way method; keep in sync with merge_3runs! */
    template<merging4way_methods mergingMethod>
    bool has_specialized_3way_merge() {
        switch (mergingMethod) {
            case merging4way_methods::WILLEM_WITH_INDICES:
            case merging4way_methods::WILLEM_TUNED:
            case merging4way_methods::GENERAL_BY_STAGES_SPLIT:
                return true;
            case merging4way_methods::FOR_NUMERIC_DATA:
            case merging4way_methods::GENERAL_NO_SENTINELS:
            case merging4way_methods::WILLEM:
            case merging4way_methods::WILLEM_VALUES:
            case merging4way_methods::GENERAL_INDICES:
            case merging4way_methods::GENERAL_BY_STAGES:
            case merging4way_methods::FOR_NUMERIC_DATA_PLAIN_MIN:
                return false;
            default:
                assert(false);
                __builtin_unreachable();
        }
    }

    /**
       * Merges runs [l..g1) and [g1..g2) and [g2..r) in-place into [l..r)
       * using a buffer B.
       */
    template<merging4way_methods mergingMethod, typename Iter, typename Iter2>
    void merge_3runs(Iter l, Iter g1, Iter g2, Iter r, Iter2 B) {
        switch (mergingMethod) {
            case merging4way_methods::WILLEM_WITH_INDICES:
                merge_3runs_numeric_willem_a(l, g1, g2, r, B);
                break;
            case merging4way_methods::WILLEM_TUNED:
                merge_3runs_numeric_willem_tuned(l, g1, g2, r, B);
                break;
            case merging4way_methods::GENERAL_BY_STAGES_SPLIT:
                merge_3runs_by_stages_split(l, g1, g2, r, B);
                break;
            default:
                // use 4way with empty 4th run
                assert(!has_specialized_3way_merge<mergingMethod>());
                merge_4runs<mergingMethod>(l, g1, g2, r, r, B);
        }
    }



}


#endif //MERGESORTS_MERGING_3WAY_H
