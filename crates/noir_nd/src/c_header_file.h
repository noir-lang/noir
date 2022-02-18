#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef uint8_t NoirValue[32];

extern void call_func(const uint32_t *name, const NoirValue *inputs, NoirValue *outputs);
