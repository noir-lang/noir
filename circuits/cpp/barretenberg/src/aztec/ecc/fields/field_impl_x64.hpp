#pragma once

#if (BBERG_NO_ASM == 0)
#include "asm_macros.hpp"

namespace barretenberg {

/*
asm_butterfly(field& left, field& root) noexcept
{
    __asm__(MUL("32(%0)", "40(%0)", "48(%0)", "56(%0)", "%1")
            // r12, r13, r14, r15 contains b*omega
            // we want a + b*omega
            // and a - b*omega
            "xorq %%rdx, %%rdx \n\t"
            "movq %%r12, %%r8 \n\t"
            "movq %%r13, %%r9 \n\t"
            "movq %%r14, %%r10 \n\t"
            "movq %%r15, %%r11 \n\t"
            // "adcxq (%0), %%r8 \n\t"
            // "adcxq 8(%0), %%r9 \n\t"
            // "adcxq 16(%0), %%r10 \n\t"
            // "adcxq 24(%0), %%r11 \n\t"
            "subq 32(%0), %%r8 \n\t"
            "sbbq 40(%0), %%r9 \n\t"
            "sbbq 48(%0), %%r10 \n\t"
            "sbbq 56(%0), %%r11 \n\t"
            "sbbq [%zero_reference], %%rdx \n\t"
            "xorq %%rdi, %%rdi\n\t"
            "movq %%rdx, %%rdi \n\t"
            "andq $not_modulus_0, %%rdi \n\t"
            "adoxq %%rdi, %%r8 \n\t"
            "adcxq 0(%0), %%r12 \n\t"
            "movq %%r8, 32(%0) \n\t"
            "movq %%rdx, %%rdi \n\t"
            "andq $not_modulus_1, %%rdi \n\t"
            "adoxq %%rdi, %%r9 \n\t"
            "adcxq 8(%0), %%r13 \n\t"
            "movq %%r9, 32(%0) \n\t"
            "movq %%rdx, %%rdi \n\t"
            "andq $not_modulus_2, %%rdi \n\t"
            "adoxq %%rdi, %%r10 \n\t"
            "adcxq 16(%0), %%r14 \n\t"
            "movq %%r10, 32(%0) \n\t"
            "movq %%rdx, %%rdi \n\t"
            "andq $not_modulus_3, %%rdi \n\t"
            "adoxq %%rdi, %%r11 \n\t"
            "adcxq 24(%0), %%r1 \n\t"
            "movq %%r8, 32(%0) \n\t"

            :
            : "%r"(&a),
              "%r"(&b),
              "r"(&r),
              [modulus_0] "m"(modulus_0),
              [modulus_1] "m"(modulus_1),
              [modulus_2] "m"(modulus_2),
              [modulus_3] "m"(modulus_3),
              [r_inv] "m"(r_inv),
              [zero_reference] "m"(zero_reference)
            : "%rdx", "%rdi", "%r8", "%r9", "%r10", "%r11", "%r12", "%r13", "%r14", "%r15", "cc", "memory");
    return r;
}
*/
template <class T> field<T> field<T>::asm_mul_with_coarse_reduction(const field& a, const field& b) noexcept
{
    field r;
    constexpr uint64_t r_inv = T::r_inv;
    constexpr uint64_t modulus_0 = modulus.data[0];
    constexpr uint64_t modulus_1 = modulus.data[1];
    constexpr uint64_t modulus_2 = modulus.data[2];
    constexpr uint64_t modulus_3 = modulus.data[3];
    constexpr uint64_t zero_ref = 0;

    /**
     * Registers: rax:rdx = multiplication accumulator
     *            %r12, %r13, %r14, %r15, %rax: work registers for `r`
     *            %r8, %r9, %rdi, %rsi: scratch registers for multiplication results
     *            %r10: zero register
     *            %0: pointer to `a`
     *            %1: pointer to `b`
     *            %2: pointer to `r`
     **/
    __asm__(MUL("0(%0)", "8(%0)", "16(%0)", "24(%0)", "%1")
                STORE_FIELD_ELEMENT("%2", "%%r12", "%%r13", "%%r14", "%%r15")
            :
            : "%r"(&a),
              "%r"(&b),
              "r"(&r),
              [ modulus_0 ] "m"(modulus_0),
              [ modulus_1 ] "m"(modulus_1),
              [ modulus_2 ] "m"(modulus_2),
              [ modulus_3 ] "m"(modulus_3),
              [ r_inv ] "m"(r_inv),
              [ zero_reference ] "m"(zero_ref)
            : "%rdx", "%rdi", "%r8", "%r9", "%r10", "%r11", "%r12", "%r13", "%r14", "%r15", "cc", "memory");
    return r;
}

template <class T> void field<T>::asm_self_mul_with_coarse_reduction(const field& a, const field& b) noexcept
{
    constexpr uint64_t r_inv = T::r_inv;
    constexpr uint64_t modulus_0 = modulus.data[0];
    constexpr uint64_t modulus_1 = modulus.data[1];
    constexpr uint64_t modulus_2 = modulus.data[2];
    constexpr uint64_t modulus_3 = modulus.data[3];
    constexpr uint64_t zero_ref = 0;
    /**
     * Registers: rax:rdx = multiplication accumulator
     *            %r12, %r13, %r14, %r15, %rax: work registers for `r`
     *            %r8, %r9, %rdi, %rsi: scratch registers for multiplication results
     *            %r10: zero register
     *            %0: pointer to `a`
     *            %1: pointer to `b`
     *            %2: pointer to `r`
     **/
    __asm__(MUL("0(%0)", "8(%0)", "16(%0)", "24(%0)", "%1")
                STORE_FIELD_ELEMENT("%0", "%%r12", "%%r13", "%%r14", "%%r15")
            :
            : "r"(&a),
              "r"(&b),
              [ modulus_0 ] "m"(modulus_0),
              [ modulus_1 ] "m"(modulus_1),
              [ modulus_2 ] "m"(modulus_2),
              [ modulus_3 ] "m"(modulus_3),
              [ r_inv ] "m"(r_inv),
              [ zero_reference ] "m"(zero_ref)
            : "%rdx", "%rdi", "%r8", "%r9", "%r10", "%r11", "%r12", "%r13", "%r14", "%r15", "cc", "memory");
}

template <class T> field<T> field<T>::asm_sqr_with_coarse_reduction(const field& a) noexcept
{
    field r;
    constexpr uint64_t r_inv = T::r_inv;
    constexpr uint64_t modulus_0 = modulus.data[0];
    constexpr uint64_t modulus_1 = modulus.data[1];
    constexpr uint64_t modulus_2 = modulus.data[2];
    constexpr uint64_t modulus_3 = modulus.data[3];
    constexpr uint64_t zero_ref = 0;

    /**
     * Registers: rax:rdx = multiplication accumulator
     *            %r12, %r13, %r14, %r15, %rax: work registers for `r`
     *            %r8, %r9, %rdi, %rsi: scratch registers for multiplication results
     *            %[zero_reference]: memory location of zero value
     *            %0: pointer to `a`
     *            %[r_ptr]: memory location of pointer to `r`
     **/
    __asm__(SQR("%0")
            // "movq %[r_ptr], %%rsi                   \n\t"
            STORE_FIELD_ELEMENT("%1", "%%r12", "%%r13", "%%r14", "%%r15")
            :
            : "r"(&a),
              "r"(&r),
              [ zero_reference ] "m"(zero_ref),
              [ modulus_0 ] "m"(modulus_0),
              [ modulus_1 ] "m"(modulus_1),
              [ modulus_2 ] "m"(modulus_2),
              [ modulus_3 ] "m"(modulus_3),
              [ r_inv ] "m"(r_inv)
            : "%rcx", "%rdx", "%rdi", "%r8", "%r9", "%r10", "%r11", "%r12", "%r13", "%r14", "%r15", "cc", "memory");
    return r;
}

template <class T> void field<T>::asm_self_sqr_with_coarse_reduction(const field& a) noexcept
{
    constexpr uint64_t r_inv = T::r_inv;
    constexpr uint64_t modulus_0 = modulus.data[0];
    constexpr uint64_t modulus_1 = modulus.data[1];
    constexpr uint64_t modulus_2 = modulus.data[2];
    constexpr uint64_t modulus_3 = modulus.data[3];
    constexpr uint64_t zero_ref = 0;

    /**
     * Registers: rax:rdx = multiplication accumulator
     *            %r12, %r13, %r14, %r15, %rax: work registers for `r`
     *            %r8, %r9, %rdi, %rsi: scratch registers for multiplication results
     *            %[zero_reference]: memory location of zero value
     *            %0: pointer to `a`
     *            %[r_ptr]: memory location of pointer to `r`
     **/
    __asm__(SQR("%0")
            // "movq %[r_ptr], %%rsi                   \n\t"
            STORE_FIELD_ELEMENT("%0", "%%r12", "%%r13", "%%r14", "%%r15")
            :
            : "r"(&a),
              [ zero_reference ] "m"(zero_ref),
              [ modulus_0 ] "m"(modulus_0),
              [ modulus_1 ] "m"(modulus_1),
              [ modulus_2 ] "m"(modulus_2),
              [ modulus_3 ] "m"(modulus_3),
              [ r_inv ] "m"(r_inv)
            : "%rcx", "%rdx", "%rdi", "%r8", "%r9", "%r10", "%r11", "%r12", "%r13", "%r14", "%r15", "cc", "memory");
}

template <class T> field<T> field<T>::asm_add_with_coarse_reduction(const field& a, const field& b) noexcept
{
    field r;

    constexpr uint64_t twice_not_modulus_0 = twice_not_modulus.data[0];
    constexpr uint64_t twice_not_modulus_1 = twice_not_modulus.data[1];
    constexpr uint64_t twice_not_modulus_2 = twice_not_modulus.data[2];
    constexpr uint64_t twice_not_modulus_3 = twice_not_modulus.data[3];

    __asm__(CLEAR_FLAGS("%%r12") LOAD_FIELD_ELEMENT("%0", "%%r12", "%%r13", "%%r14", "%%r15")
                ADD_REDUCE("%1",
                           "%[twice_not_modulus_0]",
                           "%[twice_not_modulus_1]",
                           "%[twice_not_modulus_2]",
                           "%[twice_not_modulus_3]") STORE_FIELD_ELEMENT("%2", "%%r12", "%%r13", "%%r14", "%%r15")
            :
            : "%r"(&a),
              "%r"(&b),
              "r"(&r),
              [ twice_not_modulus_0 ] "m"(twice_not_modulus_0),
              [ twice_not_modulus_1 ] "m"(twice_not_modulus_1),
              [ twice_not_modulus_2 ] "m"(twice_not_modulus_2),
              [ twice_not_modulus_3 ] "m"(twice_not_modulus_3)
            : "%r8", "%r9", "%r10", "%r11", "%r12", "%r13", "%r14", "%r15", "cc", "memory");
    return r;
}

template <class T> void field<T>::asm_self_add_with_coarse_reduction(const field& a, const field& b) noexcept
{
    constexpr uint64_t twice_not_modulus_0 = twice_not_modulus.data[0];
    constexpr uint64_t twice_not_modulus_1 = twice_not_modulus.data[1];
    constexpr uint64_t twice_not_modulus_2 = twice_not_modulus.data[2];
    constexpr uint64_t twice_not_modulus_3 = twice_not_modulus.data[3];

    __asm__(CLEAR_FLAGS("%%r12") LOAD_FIELD_ELEMENT("%0", "%%r12", "%%r13", "%%r14", "%%r15")
                ADD_REDUCE("%1",
                           "%[twice_not_modulus_0]",
                           "%[twice_not_modulus_1]",
                           "%[twice_not_modulus_2]",
                           "%[twice_not_modulus_3]") STORE_FIELD_ELEMENT("%0", "%%r12", "%%r13", "%%r14", "%%r15")
            :
            : "r"(&a),
              "r"(&b),
              [ twice_not_modulus_0 ] "m"(twice_not_modulus_0),
              [ twice_not_modulus_1 ] "m"(twice_not_modulus_1),
              [ twice_not_modulus_2 ] "m"(twice_not_modulus_2),
              [ twice_not_modulus_3 ] "m"(twice_not_modulus_3)
            : "%r8", "%r9", "%r10", "%r11", "%r12", "%r13", "%r14", "%r15", "cc", "memory");
}

template <class T> field<T> field<T>::asm_sub_with_coarse_reduction(const field& a, const field& b) noexcept
{
    field r;

    constexpr uint64_t twice_modulus_0 = twice_modulus.data[0];
    constexpr uint64_t twice_modulus_1 = twice_modulus.data[1];
    constexpr uint64_t twice_modulus_2 = twice_modulus.data[2];
    constexpr uint64_t twice_modulus_3 = twice_modulus.data[3];

    __asm__(
        CLEAR_FLAGS("%%r12") LOAD_FIELD_ELEMENT("%0", "%%r12", "%%r13", "%%r14", "%%r15") SUB("%1")
            REDUCE_FIELD_ELEMENT("%[twice_modulus_0]", "%[twice_modulus_1]", "%[twice_modulus_2]", "%[twice_modulus_3]")
                STORE_FIELD_ELEMENT("%2", "%%r12", "%%r13", "%%r14", "%%r15")
        :
        : "r"(&a),
          "r"(&b),
          "r"(&r),
          [ twice_modulus_0 ] "m"(twice_modulus_0),
          [ twice_modulus_1 ] "m"(twice_modulus_1),
          [ twice_modulus_2 ] "m"(twice_modulus_2),
          [ twice_modulus_3 ] "m"(twice_modulus_3)
        : "%r8", "%r9", "%r10", "%r11", "%r12", "%r13", "%r14", "%r15", "cc", "memory");
    return r;
}

template <class T> void field<T>::asm_self_sub_with_coarse_reduction(const field& a, const field& b) noexcept
{
    constexpr uint64_t twice_modulus_0 = twice_modulus.data[0];
    constexpr uint64_t twice_modulus_1 = twice_modulus.data[1];
    constexpr uint64_t twice_modulus_2 = twice_modulus.data[2];
    constexpr uint64_t twice_modulus_3 = twice_modulus.data[3];

    __asm__(
        CLEAR_FLAGS("%%r12") LOAD_FIELD_ELEMENT("%0", "%%r12", "%%r13", "%%r14", "%%r15") SUB("%1")
            REDUCE_FIELD_ELEMENT("%[twice_modulus_0]", "%[twice_modulus_1]", "%[twice_modulus_2]", "%[twice_modulus_3]")
                STORE_FIELD_ELEMENT("%0", "%%r12", "%%r13", "%%r14", "%%r15")
        :
        : "r"(&a),
          "r"(&b),
          [ twice_modulus_0 ] "m"(twice_modulus_0),
          [ twice_modulus_1 ] "m"(twice_modulus_1),
          [ twice_modulus_2 ] "m"(twice_modulus_2),
          [ twice_modulus_3 ] "m"(twice_modulus_3)
        : "%r8", "%r9", "%r10", "%r11", "%r12", "%r13", "%r14", "%r15", "cc", "memory");
}

template <class T> void field<T>::asm_conditional_negate(field& r, const uint64_t predicate) noexcept
{
    constexpr uint64_t twice_modulus_0 = twice_modulus.data[0];
    constexpr uint64_t twice_modulus_1 = twice_modulus.data[1];
    constexpr uint64_t twice_modulus_2 = twice_modulus.data[2];
    constexpr uint64_t twice_modulus_3 = twice_modulus.data[3];

    __asm__(CLEAR_FLAGS("%%r8") LOAD_FIELD_ELEMENT(
                "%1", "%%r8", "%%r9", "%%r10", "%%r11") "movq %[twice_modulus_0], %%r12 \n\t"
                                                        "movq %[twice_modulus_1], %%r13 \n\t"
                                                        "movq %[twice_modulus_2], %%r14 \n\t"
                                                        "movq %[twice_modulus_3], %%r15 \n\t"
                                                        "subq %%r8, %%r12 \n\t"
                                                        "sbbq %%r9, %%r13 \n\t"
                                                        "sbbq %%r10, %%r14 \n\t"
                                                        "sbbq %%r11, %%r15 \n\t"
                                                        "btq $0, %0 \n\t"
                                                        "cmovcq %%r12, %%r8 \n\t"
                                                        "cmovcq %%r13, %%r9 \n\t"
                                                        "cmovcq %%r14, %%r10 \n\t"
                                                        "cmovcq %%r15, %%r11 \n\t" STORE_FIELD_ELEMENT(
                                                            "%1", "%%r8", "%%r9", "%%r10", "%%r11")
            :
            : "r"(predicate),
              "r"(&r),
              [ twice_modulus_0 ] "i"(twice_modulus_0),
              [ twice_modulus_1 ] "i"(twice_modulus_1),
              [ twice_modulus_2 ] "i"(twice_modulus_2),
              [ twice_modulus_3 ] "i"(twice_modulus_3)
            : "%r8", "%r9", "%r10", "%r11", "%r12", "%r13", "%r14", "%r15", "cc", "memory");
}

template <class T> field<T> field<T>::asm_reduce_once(const field& a) noexcept
{
    field r;

    constexpr uint64_t not_modulus_0 = not_modulus.data[0];
    constexpr uint64_t not_modulus_1 = not_modulus.data[1];
    constexpr uint64_t not_modulus_2 = not_modulus.data[2];
    constexpr uint64_t not_modulus_3 = not_modulus.data[3];

    __asm__(CLEAR_FLAGS("%%r12") LOAD_FIELD_ELEMENT("%0", "%%r12", "%%r13", "%%r14", "%%r15")
                REDUCE_FIELD_ELEMENT("%[not_modulus_0]", "%[not_modulus_1]", "%[not_modulus_2]", "%[not_modulus_3]")
                    STORE_FIELD_ELEMENT("%1", "%%r12", "%%r13", "%%r14", "%%r15")
            :
            : "r"(&a),
              "r"(&r),
              [ not_modulus_0 ] "m"(not_modulus_0),
              [ not_modulus_1 ] "m"(not_modulus_1),
              [ not_modulus_2 ] "m"(not_modulus_2),
              [ not_modulus_3 ] "m"(not_modulus_3)
            : "%r8", "%r9", "%r10", "%r11", "%r12", "%r13", "%r14", "%r15", "cc", "memory");
    return r;
}

template <class T> void field<T>::asm_self_reduce_once(const field& a) noexcept
{
    constexpr uint64_t not_modulus_0 = not_modulus.data[0];
    constexpr uint64_t not_modulus_1 = not_modulus.data[1];
    constexpr uint64_t not_modulus_2 = not_modulus.data[2];
    constexpr uint64_t not_modulus_3 = not_modulus.data[3];

    __asm__(CLEAR_FLAGS("%%r12") LOAD_FIELD_ELEMENT("%0", "%%r12", "%%r13", "%%r14", "%%r15")
                REDUCE_FIELD_ELEMENT("%[not_modulus_0]", "%[not_modulus_1]", "%[not_modulus_2]", "%[not_modulus_3]")
                    STORE_FIELD_ELEMENT("%0", "%%r12", "%%r13", "%%r14", "%%r15")
            :
            : "r"(&a),
              [ not_modulus_0 ] "m"(not_modulus_0),
              [ not_modulus_1 ] "m"(not_modulus_1),
              [ not_modulus_2 ] "m"(not_modulus_2),
              [ not_modulus_3 ] "m"(not_modulus_3)
            : "%r8", "%r9", "%r10", "%r11", "%r12", "%r13", "%r14", "%r15", "cc", "memory");
}
} // namespace barretenberg
#endif