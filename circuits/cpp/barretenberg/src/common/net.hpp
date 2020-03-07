#pragma once

#ifdef __linux__
#include <endian.h>
#define ntohll be64toh
#define htonll htobe64
#endif