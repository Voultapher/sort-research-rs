/** @author Sebastian Wild (wild@liverpool.ac.uk) */

#ifndef MERGESORTS_INSERTIONSORT_H
#define MERGESORTS_INSERTIONSORT_H

#include <cstddef>
#include <iterator>
#include <algorithm>
#include <cassert>

namespace algorithms
{

	/**
	 * sorts [begin,end) using insertionsort, assuming that [begin,beginUnsorted)
	 * is already in order.
	 **/
	template<typename Iter>
	void insertionsort(Iter begin, Iter end, Iter beginUnsorted)
	{
		assert(begin <= beginUnsorted && begin <= end);
		for (Iter i = beginUnsorted; i < end; ++i) {
			Iter j = i; const auto v = *i;
			while (v < *(j-1)) {
				*j = *(j-1);
				--j;
				if (j <= begin) break;
			}
			*j = v;
		}
	}

	/**
	 * sorts [begin,end) using insertionsort, assuming that the first
	 * nPresorted elements are already in sorted order.
	 **/
	template<typename Iter>
	inline void insertionsort(Iter begin, Iter end, size_t nPresorted = 1)
	{
		insertionsort(begin, end, begin + nPresorted);
	}


	/**
	 * sorts [begin,end) using binary insertionsort, 
	 * assuming that [begin,beginUnsorted) is already in order.
	 **/
	template<typename Iter>
	void binary_insertionsort(Iter begin, Iter end, Iter beginUnsorted) {
		assert(begin <= beginUnsorted && begin <= end);
		for (Iter i = std::max(beginUnsorted, begin+1); i < end; ++i) {
			assert(begin <= i);
			const auto pivot = *i;
			Iter const pos = std::upper_bound(begin, i, pivot, std::less<>());
			for (auto p = i; p > pos; --p) *p = *(p - 1);
			*pos = pivot;
		}

	}

	/**
	 * sorts [begin,end) using insertionsort, assuming that the first
	 * nPresorted elements are already in sorted order.
	 **/
	template<typename Iter>
	inline void binary_insertionsort(Iter begin, Iter end, size_t nPresorted = 1)
	{
		binary_insertionsort(begin, end, begin + nPresorted);
	}

}

#endif //MERGESORTS_INSERTIONSORT_H
