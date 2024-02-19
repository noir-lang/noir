#include "fr.hpp"

#include <benchmark/benchmark.h>

using namespace bb;
using namespace benchmark;

#ifndef DISABLE_ASM
namespace {
void asm_add_with_coarse_reduction(State& state) noexcept
{
    fr x, y;
    for (auto _ : state) {
        DoNotOptimize(fr::asm_add_with_coarse_reduction(x, y));
    }
}
BENCHMARK(asm_add_with_coarse_reduction);

void asm_conditional_negate(State& state) noexcept
{
    fr x;
    for (auto _ : state) {
        fr::asm_conditional_negate(x, true);
    }
}
BENCHMARK(asm_conditional_negate);

void asm_mul_with_coarse_reduction(State& state) noexcept
{
    fr x, y;
    for (auto _ : state) {
        DoNotOptimize(fr::asm_mul_with_coarse_reduction(x, y));
    }
}
BENCHMARK(asm_mul_with_coarse_reduction);

void asm_reduce_once(State& state) noexcept
{
    fr x;
    for (auto _ : state) {
        DoNotOptimize(fr::asm_reduce_once(x));
    }
}
BENCHMARK(asm_reduce_once);

void asm_self_add_with_coarse_reduction(State& state) noexcept
{
    fr x, y;
    for (auto _ : state) {
        fr::asm_self_add_with_coarse_reduction(x, y);
    }
}
BENCHMARK(asm_self_add_with_coarse_reduction);

void asm_self_mul_with_coarse_reduction(State& state) noexcept
{
    fr x, y;
    for (auto _ : state) {
        fr::asm_self_mul_with_coarse_reduction(x, y);
    }
}
BENCHMARK(asm_self_mul_with_coarse_reduction);

void asm_self_reduce_once(State& state) noexcept
{
    fr x;
    for (auto _ : state) {
        fr::asm_self_reduce_once(x);
    }
}
BENCHMARK(asm_self_reduce_once);

void asm_self_sqr_with_coarse_reduction(State& state) noexcept
{
    fr x;
    for (auto _ : state) {
        fr::asm_self_sqr_with_coarse_reduction(x);
    }
}
BENCHMARK(asm_self_sqr_with_coarse_reduction);

void asm_self_sub_with_coarse_reduction(State& state) noexcept
{
    fr x, y;
    for (auto _ : state) {
        fr::asm_self_sub_with_coarse_reduction(x, y);
    }
}
BENCHMARK(asm_self_sub_with_coarse_reduction);

void asm_sqr_with_coarse_reduction(State& state) noexcept
{
    fr x;
    for (auto _ : state) {
        DoNotOptimize(fr::asm_sqr_with_coarse_reduction(x));
    }
}
BENCHMARK(asm_sqr_with_coarse_reduction);

void mul(State& state) noexcept
{
    fr x, y;
    for (auto _ : state) {
        DoNotOptimize(x * y);
    }
}
BENCHMARK(mul);

void self_mul(State& state) noexcept
{
    fr x, y;
    for (auto _ : state) {
        x *= y;
    }
}
BENCHMARK(self_mul);

void add(State& state) noexcept
{
    fr x, y;
    for (auto _ : state) {
        DoNotOptimize(x + y);
    }
}
BENCHMARK(add);

void self_add(State& state) noexcept
{
    fr x, y;
    for (auto _ : state) {
        x += y;
    }
}
BENCHMARK(self_add);

void sub(State& state) noexcept
{
    fr x, y;
    for (auto _ : state) {
        DoNotOptimize(x - y);
    }
}
BENCHMARK(sub);

void self_sub(State& state) noexcept
{
    fr x, y;
    for (auto _ : state) {
        x -= y;
    }
}
BENCHMARK(self_sub);

void invert(State& state) noexcept
{
    fr x;
    for (auto _ : state) {
        DoNotOptimize(x.invert());
    }
}
BENCHMARK(invert);

void self_neg(State& state) noexcept
{
    fr x;
    for (auto _ : state) {
        x.self_neg();
    }
}
BENCHMARK(self_neg);

void self_reduce_once(State& state) noexcept
{
    fr x;
    for (auto _ : state) {
        x.self_reduce_once();
    }
}
BENCHMARK(self_reduce_once);

void self_to_montgomery_form(State& state) noexcept
{
    fr x;
    for (auto _ : state) {
        x.self_to_montgomery_form();
    }
}
BENCHMARK(self_to_montgomery_form);

void self_sqr(State& state) noexcept
{
    fr x;
    for (auto _ : state) {
        x.self_sqr();
    }
}
BENCHMARK(self_sqr);

void sqr(State& state) noexcept
{
    fr x;
    for (auto _ : state) {
        DoNotOptimize(x.sqr());
    }
}
BENCHMARK(sqr);
} // namespace
#endif

// NOLINTNEXTLINE macro invocation triggers style guideline errors from googletest code
BENCHMARK_MAIN();
