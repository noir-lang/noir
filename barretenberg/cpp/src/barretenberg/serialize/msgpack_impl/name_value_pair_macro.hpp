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
                      _31,                                                                                             \
                      _32,                                                                                             \
                      _33,                                                                                             \
                      _34,                                                                                             \
                      _35,                                                                                             \
                      _36,                                                                                             \
                      _37,                                                                                             \
                      _38,                                                                                             \
                      _39,                                                                                             \
                      _40,                                                                                             \
                      N,                                                                                               \
                      ...)                                                                                             \
    N
// AD: support for 40 fields!? one may ask. Well, after 30 not being enough...
#define VA_NARGS(...)                                                                                                  \
    VA_NARGS_IMPL(__VA_ARGS__,                                                                                         \
                  40,                                                                                                  \
                  39,                                                                                                  \
                  38,                                                                                                  \
                  37,                                                                                                  \
                  36,                                                                                                  \
                  35,                                                                                                  \
                  34,                                                                                                  \
                  33,                                                                                                  \
                  32,                                                                                                  \
                  31,                                                                                                  \
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
#define _NVP31(x, ...) _NVP1(x), _NVP30(__VA_ARGS__)
#define _NVP32(x, ...) _NVP1(x), _NVP31(__VA_ARGS__)
#define _NVP33(x, ...) _NVP1(x), _NVP32(__VA_ARGS__)
#define _NVP34(x, ...) _NVP1(x), _NVP33(__VA_ARGS__)
#define _NVP35(x, ...) _NVP1(x), _NVP34(__VA_ARGS__)
#define _NVP36(x, ...) _NVP1(x), _NVP35(__VA_ARGS__)
#define _NVP37(x, ...) _NVP1(x), _NVP36(__VA_ARGS__)
#define _NVP38(x, ...) _NVP1(x), _NVP37(__VA_ARGS__)
#define _NVP39(x, ...) _NVP1(x), _NVP38(__VA_ARGS__)
#define _NVP40(x, ...) _NVP1(x), _NVP39(__VA_ARGS__)

#define CONCAT(a, b) a##b
#define _NVP_N(n) CONCAT(_NVP, n)
#define NVP(...) _NVP_N(VA_NARGS(__VA_ARGS__))(__VA_ARGS__)
// end of #define NVP
