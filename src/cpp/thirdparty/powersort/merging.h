/** @author Sebastian Wild (wild@liverpool.ac.uk) */

#ifndef MERGESORTS_MERGING_H
#define MERGESORTS_MERGING_H

#include <algorithm>

namespace algorithms {

#ifdef COUNT_MERGECOST
	const bool COUNT_MERGE_COSTS = true;
#else
	const bool COUNT_MERGE_COSTS = false;
#endif
	long long totalMergeCosts = 0;
	long long totalBufferCosts = 0;

    /**
     * A sentinel value used by some merging method;
     * this value must be strictly larger than any value in the input.
     */
    template<typename T>
    T plus_inf_sentinel() {
        if constexpr (std::numeric_limits<T>::is_specialized) {
			if (std::numeric_limits<T>::has_infinity)
				return std::numeric_limits<T>::infinity();
			else
				return std::numeric_limits<T>::max();
		} else {
			throw std::runtime_error{"plus_inf_sentinel not possible for this type"};
		}
    }



    enum merging_methods {
        UNSTABLE_BITONIC_MERGE  /** @deprecated */,
        UNSTABLE_BITONIC_MERGE_MANUAL_COPY  /** @deprecated not faster */,
        UNSTABLE_BITONIC_MERGE_BRANCHLESS  /** @deprecated not faster */,
        COPY_SMALLER,
        COPY_BOTH,
        // COPY_BOTH_WITH_SENTINELS
    };

    std::string to_string(merging_methods mergingMethod) {
        switch (mergingMethod) {
            case UNSTABLE_BITONIC_MERGE:
                return "UNSTABLE_BITONIC_MERGE";
            case UNSTABLE_BITONIC_MERGE_MANUAL_COPY:
                return "UNSTABLE_BITONIC_MERGE_MANUAL_COPY";
            case UNSTABLE_BITONIC_MERGE_BRANCHLESS:
                return "UNSTABLE_BITONIC_MERGE_BRANCHLESS";
            case COPY_SMALLER:
                return "COPY_SMALLER";
            case COPY_BOTH:
                return "COPY_BOTH";
            // case COPY_BOTH_WITH_SENTINELS:
            //     return "COPY_BOTH_WITH_SENTINELS";
            default:
                assert(false);
                __builtin_unreachable();
        }
    }

	/**
	 * Merges runs [l..m) and [m..r) in-place into [l..r)
	 * based on Sedgewick's bitonic merge (Program 8.2 in Algorithms in C++)
	 * using b as temporary storage.
	 * buffer space at b must be at least r-l.
	 *
	 * This method is not stable as is;
	 * it could be made so using an infinity-sentinel between the runs.
	 */
	template<typename Iter, typename Iter2>
	void merge_runs_bitonic(Iter l, Iter m, Iter r, Iter2 B) {
		if (COUNT_MERGE_COSTS) totalMergeCosts += (r-l);
		std::copy_backward(l,m,B+(m-l));
        std::reverse_copy(m,r,B+(m-l));
        if (COUNT_MERGE_COSTS) totalBufferCosts += (r-l);
        auto i = B, j = B+(r-l-1);
		for (auto k = l; k < r; ++k)
			*k = *j < *i ? *j-- : *i++;
	}

	/**
	 * Merges runs [l..m-1] and [m..r) in-place into [l..r)
	 * based on Sedgewick's bitonic merge (Program 8.2 in Algorithms in C++)
	 * using b as temporary storage.
	 * buffer space at b must be at least r-l.
	 *
	 * (same as above, but with manual copy in loops; slightly slower than above)
	 */
	template<typename Iter, typename Iter2>
	void merge_runs_bitonic_manual_copy(Iter l, Iter m, Iter r, Iter2 B) {
		Iter i1, j1; Iter2 b;
		if (COUNT_MERGE_COSTS) totalMergeCosts += (r-l);
		for (i1 = m-1, b = B+(m-1-l); i1 >= l;) *b-- = *i1--;
		for (j1 = r, b = B+(m-l); j1 > m;) *b++ = *--j1;
        if (COUNT_MERGE_COSTS) totalBufferCosts += (r-l);
		auto i = B, j = B+(r-l-1);
		for (auto k = l; k < r; ++k)
			*k = *j < *i ? *j-- : *i++;
	}

	/**
	 * Merges runs [l..m-1] and [m..r) in-place into [l..r)
	 * based on Sedgewick's bitonic merge (Program 8.2 in Algorithms in C++)
	 * using b as temporary storage.
	 * buffer space at b must be at least r-l.
	 *
	 * (same as above but with branchless assignments; a good bit slower,
	 * and apparently not needed; recent compilers seem to compile above
	 * to branchless code, as well.)
	 */
	template<typename Iter, typename Iter2>
	void merge_runs_bitonic_branchless(Iter l, Iter m, Iter r, Iter2 B) {
		if (COUNT_MERGE_COSTS) totalMergeCosts += (r-l);
		std::copy_backward(l,m,B+(m-l));
		std::reverse_copy(m,r,B+(m-l));
        if (COUNT_MERGE_COSTS) totalBufferCosts += (r-l);
		Iter2 i = B, j = B+(r-l-1);
		for (auto k = l; k < r; ++k) {
			bool const cmp = *j < *i;
			*k = cmp ? *j : *i;
			j -= cmp ? 1 : 0;
			i += cmp ? 0 : 1;
		}
	}

	/**
	 * Merges runs A[l..m-1] and A[m..r) in-place into A[l..r)
	 * by copying the shorter run into temporary storage B and
	 * merging back into A.
	 * B must have space at least min(m-l,r-m+1)
	 */
	template<typename Iter, typename Iter2>
	void merge_runs_copy_half(Iter l, Iter m, Iter r, Iter2 B) {
		auto n1 = m-l, n2 = r-m;
		if (COUNT_MERGE_COSTS) totalMergeCosts += (n1+n2);
        if (n1 <= n2) {
            std::copy(l,m,B);
            if (COUNT_MERGE_COSTS) totalBufferCosts += (m-l);
            auto c1 = B, e1 = B + n1;
            auto c2 = m, e2 = r, o = l;
            while (c1 < e1 && c2 < e2)
                *o++ = *c1 <= *c2 ? *c1++ : *c2++;
            while (c1 < e1) *o++ = *c1++;
        } else {
            std::copy(m,r,B);
            if (COUNT_MERGE_COSTS) totalBufferCosts += (r-m);
            auto c1 = m-1, s1 = l, o = r-1;
            auto c2 = B+n2-1, s2 = B;
            while (c1 >= s1 && c2 >= s2)
                *o-- = *c1 <= *c2 ? *c2-- : *c1--;
            while (c2 >= s2) *o-- = *c2--;
        }
	}

	/**
	 * Merges runs A[l..m) and A[m..r) in-place into A[l..r)
	 * by copying both to buffer B and merging back into A.
	 * B must have space at least r-l.
	 */
	template<typename Iter, typename Iter2>
	void merge_runs_basic(Iter l, Iter m, Iter r, Iter2 B) {
		auto n1 = m-l, n2 = r-m;
		if (COUNT_MERGE_COSTS) totalMergeCosts += (n1+n2);
        std::copy(l,r,B);
        if (COUNT_MERGE_COSTS) totalBufferCosts += (n1+n2);
        auto c1 = B, e1 = B + n1, c2 = e1, e2 = e1 + n2;
        auto o = l;
        while (c1 < e1 && c2 < e2)
            *o++ = *c1 <= *c2 ? *c1++ : *c2++;
        while (c1 < e1) *o++ = *c1++;
        while (c2 < e2) *o++ = *c2++;
	}

	/**
	 * Merges runs A[l..m) and A[m..r) in-place into A[l..r)
	 * by copying both to buffer B and merging back into A, using sentinels to speed up inner loops.
	 * B must have space at least r-l + 2 and Iter must support a sentinel value.
	 */
	// template<typename Iter, typename Iter2>
	// void merge_runs_basic_sentinels(Iter l, Iter m, Iter r, Iter2 B) {
    //     typedef typename std::iterator_traits<Iter>::value_type T;
    //     static_assert(std::numeric_limits<T>::is_specialized, "Needs numeric type (for sentinels)");
    //     auto n1 = m-l, n2 = r-m;
	// 	if (COUNT_MERGE_COSTS) totalMergeCosts += (n1+n2);
    //     std::copy(l, m, B);
    //     *(B + (m - l)) = plus_inf_sentinel<T>();
    //     std::copy(m, r, B + (m - l + 1));
    //     *(B + (r - l) + 1) = plus_inf_sentinel<T>();
    //     if (COUNT_MERGE_COSTS) totalBufferCosts += (n1+n2+2);
    //     auto c1 = B, c2 = B + (m - l + 1), o = l;
    //     while (o < r) *o++ = *c1 <= *c2 ? *c1++ : *c2++;
	// }



#ifdef USE_OLD_RUN_DETECTION_LOOPS_WITH_IF_IN_BODY
/** returns maximal i <= end s.t. [begin,i) is weakly increasing */
	template<typename Iterator>
	Iterator weaklyIncreasingPrefix(Iterator begin, Iterator end) {
		while (begin + 1 < end)
			if (*begin <= *(begin + 1)) ++begin;
			else break;
		return begin + 1;
	}

	/** returns minimal i >= begin s.t. [i, end) is weakly increasing */
	template<typename Iterator>
	Iterator weaklyIncreasingSuffix(Iterator begin, Iterator end) {
		while (end - 1 > begin)
			if (*(end - 2) <= *(end - 1)) --end;
			else break;
		return end - 1;
	}

	template<typename Iterator>
	Iterator strictlyDecreasingPrefix(Iterator begin, Iterator end) {
		while (begin + 1 < end)
			if (*begin > *(begin + 1)) ++begin;
			else break;
		return begin + 1;
	}

	template<typename Iterator>
	Iterator strictlyDecreasingSuffix(Iterator begin, Iterator end) {
		while (end - 1 > begin)
			if (*(end - 2) > *(end - 1)) --end;
			else break;
		return end - 1;
	}
#else
	/** returns maximal i <= end s.t. [begin,i) is weakly increasing */
	template<typename Iterator>
	Iterator weaklyIncreasingPrefix(Iterator begin, Iterator end) {
		while (begin + 1 < end && *begin <= *(begin + 1)) ++begin;
		return begin + 1;
	}

	/** returns minimal i >= begin s.t. [i, end) is weakly increasing */
	template<typename Iterator>
	Iterator weaklyIncreasingSuffix(Iterator begin, Iterator end) {
		while (end - 1 > begin && *(end - 2) <= *(end - 1)) --end;
		return end - 1;
	}

	template<typename Iterator>
	Iterator strictlyDecreasingPrefix(Iterator begin, Iterator end) {
		while (begin + 1 < end &&  *begin > *(begin + 1)) ++begin;
		return begin + 1;
	}

	template<typename Iterator>
	Iterator strictlyDecreasingSuffix(Iterator begin, Iterator end) {
		while (end - 1 > begin && *(end - 2) > *(end - 1)) --end;
		return end - 1;
	}
#endif // USE_OLD_RUN_DETECTION_LOOPS_WITH_IF_IN_BODY

	template<typename Iterator>
	Iterator extend_and_reverse_run_right(Iterator begin, Iterator end) {
		Iterator j = begin;
		if (j == end) return j;
		if (j+1 == end) return j+1;
		if (*j > *(j+1)) {
			j = strictlyDecreasingPrefix(begin, end);
			std::reverse(begin, j);
		} else {
			j = weaklyIncreasingPrefix(begin, end);
		}
		return j;
	}


    /** Merges runs [l..m) and [m..r) in-place into [l..r) */
    template<merging_methods mergingMethod,
            typename Iter, typename Iter2>
    void merge_runs(Iter l, Iter m, Iter r, Iter2 B) {
        switch(mergingMethod) {
            case UNSTABLE_BITONIC_MERGE:
                return merge_runs_bitonic(l, m, r, B);
            case UNSTABLE_BITONIC_MERGE_MANUAL_COPY:
                return merge_runs_bitonic_manual_copy(l, m, r, B);
            case UNSTABLE_BITONIC_MERGE_BRANCHLESS:
                return merge_runs_bitonic_branchless(l, m, r, B);
            case COPY_SMALLER:
                return merge_runs_copy_half(l, m, r, B);
            case COPY_BOTH:
                return merge_runs_basic(l, m, r, B);
            // case COPY_BOTH_WITH_SENTINELS:
            //     return merge_runs_basic_sentinels(l, m, r, B);
            default:
                assert(false);
                __builtin_unreachable();
        }
    }



}

#endif //MERGESORTS_MERGING_H
