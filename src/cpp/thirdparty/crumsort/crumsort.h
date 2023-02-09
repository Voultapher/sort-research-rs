/*
        Copyright (C) 2014-2022 Igor van den Hoven ivdhoven@gmail.com
*/

/*
        Permission is hereby granted, free of charge, to any person obtaining
        a copy of this software and associated documentation files (the
        "Software"), to deal in the Software without restriction, including
        without limitation the rights to use, copy, modify, merge, publish,
        distribute, sublicense, and/or sell copies of the Software, and to
        permit persons to whom the Software is furnished to do so, subject to
        the following conditions:

        The above copyright notice and this permission notice shall be
        included in all copies or substantial portions of the Software.

        THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
        EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
        MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
        IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
        CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
        TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
        SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

/*
        crumsort 1.1.5.2
*/

#ifndef CRUMSORT_H
#define CRUMSORT_H

//#define cmp(a,b) (*(a) > *(b))

#ifndef QUADSORT_H
#include "quadsort.h"
#endif

#include <assert.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef int CMPFUNC(const void* a, const void* b);

//////////////////////////////////////////////////////////
//┌────────────────────────────────────────────────────┐//
//│                █████┐    ██████┐ ██████┐████████┐  │//
//│               ██┌──██┐   ██┌──██┐└─██┌─┘└──██┌──┘  │//
//│               └█████┌┘   ██████┌┘  ██│     ██│     │//
//│               ██┌──██┐   ██┌──██┐  ██│     ██│     │//
//│               └█████┌┘   ██████┌┘██████┐   ██│     │//
//│                └────┘    └─────┘ └─────┘   └─┘     │//
//└────────────────────────────────────────────────────┘//
//////////////////////////////////////////////////////////

#undef VAR
#undef FUNC
#undef STRUCT

#define VAR char
#define FUNC(NAME) NAME##8
#define STRUCT(NAME) struct NAME##8

#include "crumsort.c"

//////////////////////////////////////////////////////////
//┌────────────────────────────────────────────────────┐//
//│           ▄██┐   █████┐    ██████┐ ██████┐████████┐│//
//│          ████│  ██┌───┘    ██┌──██┐└─██┌─┘└──██┌──┘│//
//│          └─██│  ██████┐    ██████┌┘  ██│     ██│   │//
//│            ██│  ██┌──██┐   ██┌──██┐  ██│     ██│   │//
//│          ██████┐└█████┌┘   ██████┌┘██████┐   ██│   │//
//│          └─────┘ └────┘    └─────┘ └─────┘   └─┘   │//
//└────────────────────────────────────────────────────┘//
//////////////////////////////////////////////////////////

#undef VAR
#undef FUNC
#undef STRUCT

#define VAR short
#define FUNC(NAME) NAME##16
#define STRUCT(NAME) struct NAME##16

#include "crumsort.c"

//////////////////////////////////////////////////////////
// ┌───────────────────────────────────────────────────┐//
// │       ██████┐ ██████┐    ██████┐ ██████┐████████┐ │//
// │       └────██┐└────██┐   ██┌──██┐└─██┌─┘└──██┌──┘ │//
// │        █████┌┘ █████┌┘   ██████┌┘  ██│     ██│    │//
// │        └───██┐██┌───┘    ██┌──██┐  ██│     ██│    │//
// │       ██████┌┘███████┐   ██████┌┘██████┐   ██│    │//
// │       └─────┘ └──────┘   └─────┘ └─────┘   └─┘    │//
// └───────────────────────────────────────────────────┘//
//////////////////////////////////////////////////////////

#undef VAR
#undef FUNC
#undef STRUCT

#define VAR int
#define FUNC(NAME) NAME##32
#define STRUCT(NAME) struct NAME##32

#include "crumsort.c"

//////////////////////////////////////////////////////////
// ┌───────────────────────────────────────────────────┐//
// │        █████┐ ██┐  ██┐   ██████┐ ██████┐████████┐ │//
// │       ██┌───┘ ██│  ██│   ██┌──██┐└─██┌─┘└──██┌──┘ │//
// │       ██████┐ ███████│   ██████┌┘  ██│     ██│    │//
// │       ██┌──██┐└────██│   ██┌──██┐  ██│     ██│    │//
// │       └█████┌┘     ██│   ██████┌┘██████┐   ██│    │//
// │        └────┘      └─┘   └─────┘ └─────┘   └─┘    │//
// └───────────────────────────────────────────────────┘//
//////////////////////////////////////////////////////////

#undef VAR
#undef FUNC
#undef STRUCT

#define VAR long long
#define FUNC(NAME) NAME##64
#define STRUCT(NAME) struct NAME##64

#include "crumsort.c"

//////////////////////////////////////////////////////////
//┌────────────────────────────────────────────────────┐//
//│  ▄██┐  ██████┐  █████┐    ██████┐ ██████┐████████┐ │//
//│ ████│  └────██┐██┌──██┐   ██┌──██┐└─██┌─┘└──██┌──┘ │//
//│ └─██│   █████┌┘└█████┌┘   ██████┌┘  ██│     ██│    │//
//│   ██│  ██┌───┘ ██┌──██┐   ██┌──██┐  ██│     ██│    │//
//│ ██████┐███████┐└█████┌┘   ██████┌┘██████┐   ██│    │//
//│ └─────┘└──────┘ └────┘    └─────┘ └─────┘   └─┘    │//
//└────────────────────────────────────────────────────┘//
//////////////////////////////////////////////////////////

#undef VAR
#undef FUNC
#undef STRUCT

#define VAR long double
#define FUNC(NAME) NAME##128
#define STRUCT(NAME) struct NAME##128

#include "crumsort.c"

///////////////////////////////////////////////////////////////////////////
//┌─────────────────────────────────────────────────────────────────────┐//
//│ ██████┐██████┐ ██┐   ██┐███┐  ███┐███████┐ ██████┐ ██████┐ ████████┐│//
//│██┌────┘██┌──██┐██│   ██│████┐████│██┌────┘██┌───██┐██┌──██┐└──██┌──┘│//
//│██│     ██████┌┘██│   ██│██┌███┌██│███████┐██│   ██│██████┌┘   ██│   │//
//│██│     ██┌──██┐██│   ██│██│└█┌┘██│└────██│██│   ██│██┌──██┐   ██│   │//
//│└██████┐██│  ██│└██████┌┘██│ └┘ ██│███████│└██████┌┘██│  ██│   ██│   │//
//│ └─────┘└─┘  └─┘ └─────┘ └─┘    └─┘└──────┘ └─────┘ └─┘  └─┘   └─┘   │//
//└─────────────────────────────────────────────────────────────────────┘//
//////////////////////////////////////////////////////////////////////////

void crumsort(void* array, size_t nmemb, size_t size, CMPFUNC* cmp) {
  if (nmemb < 2) {
    return;
  }

  switch (size) {
    case sizeof(char):
      return crumsort8(static_cast<char*>(array), nmemb, cmp);

    case sizeof(short):
      return crumsort16(static_cast<short*>(array), nmemb, cmp);

    case sizeof(int):
      return crumsort32(static_cast<int*>(array), nmemb, cmp);

    case sizeof(long long):
      return crumsort64(static_cast<long long*>(array), nmemb, cmp);

    // case sizeof(long double):
    //   return crumsort128(static_cast<long double*>(array), nmemb, cmp);

    default:
      return assert(size == sizeof(char) || size == sizeof(short) ||
                    size == sizeof(int) || size == sizeof(long long) ||
                    size == sizeof(long double));
  }
}

#undef VAR
#undef FUNC
#undef STRUCT

#endif
