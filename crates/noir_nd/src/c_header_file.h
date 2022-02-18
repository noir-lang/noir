#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct C_ExternFuncCall {
  const char *name;
  const uint8_t (*inputs)[32];
  uint8_t (*outputs)[32];
} C_ExternFuncCall;

extern void call_func(struct C_ExternFuncCall x);
