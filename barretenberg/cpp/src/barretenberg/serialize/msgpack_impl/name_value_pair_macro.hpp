#pragma once

/* start of #define NVP:
 * expands to name-value pairs (NVPs), e.g. NVP(x,y,z) -> "x", x, "y", y", "z", z
 * used in msgpack serialization. */

// hacky counting of variadic macro params:
#define VA_NARGS_IMPL(_1,                                                                                              \
                      _2,                                                                                              \
                      _3,                                                                                              \
                      _4,                                                                                              \
                      _5,                                                                                              \
                      _6,                                                                                              \
                      _7,                                                                                              \
                      _8,                                                                                              \
                      _9,                                                                                              \
                      _10,                                                                                             \
                      _11,                                                                                             \
                      _12,                                                                                             \
                      _13,                                                                                             \
                      _14,                                                                                             \
                      _15,                                                                                             \
                      _16,                                                                                             \
                      _17,                                                                                             \
                      _18,                                                                                             \
                      _19,                                                                                             \
                      _20,                                                                                             \
                      _21,                                                                                             \
                      _22,                                                                                             \
                      _23,                                                                                             \
                      _24,                                                                                             \
                      _25,                                                                                             \
                      _26,                                                                                             \
                      _27,                                                                                             \
                      _28,                                                                                             \
                      _29,                                                                                             \
                      _30,                                                                                             \
                      N,                                                                                               \
                      ...)                                                                                             \
    N
// AD: support for 30 fields!? one may ask. Well, after 20 not being enough...
#define VA_NARGS(...)                                                                                                  \
    VA_NARGS_IMPL(__VA_ARGS__,                                                                                         \
                  30,                                                                                                  \
                  29,                                                                                                  \
                  28,                                                                                                  \
                  27,                                                                                                  \
                  26,                                                                                                  \
                  25,                                                                                                  \
                  24,                                                                                                  \
                  23,                                                                                                  \
                  22,                                                                                                  \
                  21,                                                                                                  \
                  20,                                                                                                  \
                  19,                                                                                                  \
                  18,                                                                                                  \
                  17,                                                                                                  \
                  16,                                                                                                  \
                  15,                                                                                                  \
                  14,                                                                                                  \
                  13,                                                                                                  \
                  12,                                                                                                  \
                  11,                                                                                                  \
                  10,                                                                                                  \
                  9,                                                                                                   \
                  8,                                                                                                   \
                  7,                                                                                                   \
                  6,                                                                                                   \
                  5,                                                                                                   \
                  4,                                                                                                   \
                  3,                                                                                                   \
                  2,                                                                                                   \
                  1)

// name-value pair expansion for variables
// used in msgpack map expansion
// n<=3 case
#define _NVP1(x) #x, x
#define _NVP2(x, y) #x, x, #y, y
#define _NVP3(x, y, z) #x, x, #y, y, #z, z
// n>3 cases
#define _NVP4(x, ...) _NVP1(x), _NVP3(__VA_ARGS__)
#define _NVP5(x, ...) _NVP1(x), _NVP4(__VA_ARGS__)
#define _NVP6(x, ...) _NVP1(x), _NVP5(__VA_ARGS__)
#define _NVP7(x, ...) _NVP1(x), _NVP6(__VA_ARGS__)
#define _NVP8(x, ...) _NVP1(x), _NVP7(__VA_ARGS__)
#define _NVP9(x, ...) _NVP1(x), _NVP8(__VA_ARGS__)
#define _NVP10(x, ...) _NVP1(x), _NVP9(__VA_ARGS__)
#define _NVP11(x, ...) _NVP1(x), _NVP10(__VA_ARGS__)
#define _NVP12(x, ...) _NVP1(x), _NVP11(__VA_ARGS__)
#define _NVP13(x, ...) _NVP1(x), _NVP12(__VA_ARGS__)
#define _NVP14(x, ...) _NVP1(x), _NVP13(__VA_ARGS__)
#define _NVP15(x, ...) _NVP1(x), _NVP14(__VA_ARGS__)
#define _NVP16(x, ...) _NVP1(x), _NVP15(__VA_ARGS__)
#define _NVP17(x, ...) _NVP1(x), _NVP16(__VA_ARGS__)
#define _NVP18(x, ...) _NVP1(x), _NVP17(__VA_ARGS__)
#define _NVP19(x, ...) _NVP1(x), _NVP18(__VA_ARGS__)
#define _NVP20(x, ...) _NVP1(x), _NVP19(__VA_ARGS__)
#define _NVP21(x, ...) _NVP1(x), _NVP20(__VA_ARGS__)
#define _NVP22(x, ...) _NVP1(x), _NVP21(__VA_ARGS__)
#define _NVP23(x, ...) _NVP1(x), _NVP22(__VA_ARGS__)
#define _NVP24(x, ...) _NVP1(x), _NVP23(__VA_ARGS__)
#define _NVP25(x, ...) _NVP1(x), _NVP24(__VA_ARGS__)
#define _NVP26(x, ...) _NVP1(x), _NVP25(__VA_ARGS__)
#define _NVP27(x, ...) _NVP1(x), _NVP26(__VA_ARGS__)
#define _NVP28(x, ...) _NVP1(x), _NVP27(__VA_ARGS__)
#define _NVP29(x, ...) _NVP1(x), _NVP28(__VA_ARGS__)
#define _NVP30(x, ...) _NVP1(x), _NVP29(__VA_ARGS__)

#define CONCAT(a, b) a##b
#define _NVP_N(n) CONCAT(_NVP, n)
#define NVP(...) _NVP_N(VA_NARGS(__VA_ARGS__))(__VA_ARGS__)
// end of #define NVP
