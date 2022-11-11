//
// Created by seb on 5/10/18.
//

#ifndef C_ALGORITHMS_H
#define C_ALGORITHMS_H

#include <algorithm>
#include <functional>
#include <ostream>
#include <string>
#include <iterator>

namespace algorithms {


	/** superclass for sorting methods */
	template<typename Iterator>
	class sorter {
	protected:
		using elem_t = typename std::iterator_traits<Iterator>::value_type ;
		using diff_t = typename std::iterator_traits<Iterator>::difference_type ;
	public:
		virtual std::string name() const = 0;

		/**
		 * sorts the given collection [start..end).
		 * (We use the STL convention that start is inclusive,
		 * and end is exclusive.)
		 */
		virtual void sort(Iterator begin, Iterator end) = 0;

		void operator()(Iterator begin, Iterator end) {
			sort(begin, end);
		}

		friend std::ostream &operator<<(std::ostream &os, const sorter &sorter) {
			os << sorter.name();
			return os;
		}

		virtual bool is_real_sort() { return true; }

	};

	/** No-operation dummy implementation of sorter */
	template<typename Iterator, bool withBuffer = false>
	struct nop final : sorter<Iterator> {
    private:
        using typename sorter<Iterator>::elem_t;
        std::vector<elem_t> _buffer;
    public:
		void sort(Iterator begin, Iterator end) override {
            if (withBuffer) {
                _buffer.resize(end - begin);
            }
			// do nothing
		}

		std::string name() const override {
			return "nop";
		}

		bool is_real_sort() override {
			return false;
		}
	};


	/** std::sort implementation of sorter */
	template<typename Iterator>
	struct std_sort final : sorter<Iterator> {
		void sort(Iterator start, Iterator end) override {
			std::sort(start, end);
		}

		std::string name() const override {
			return "std::sort";
		}
	};

	/** std::stable_sort implementation of sorter */
	template<typename Iterator>
	struct std_stable_sort final : sorter<Iterator> {
		void sort(Iterator start, Iterator end) override {
			std::stable_sort(start, end);
		}

		std::string name() const override {
			return "std::stable_sort";
		}
	};



}
#endif //C_ALGORITHMS_H
