#include "polynomial.hpp"
#include "polynomial_arithmetic.hpp"
#include <common/assert.hpp>
#include <common/mem.hpp>
#include <common/throw_or_abort.hpp>
#include <sys/stat.h>
#include <fcntl.h>
#ifndef __wasm__
#include <sys/mman.h>
#endif

namespace {
size_t clamp(size_t target, size_t step)
{
    size_t res = (target / step) * step;
    if (res < target)
        res += step;
    return res;
}
} // namespace

namespace barretenberg {

/**
 * Constructors / Destructors
 **/
polynomial::polynomial(std::string const& filename)
    : mapped(true)
    , representation(ROOTS_OF_UNITY)
    , page_size(DEFAULT_SIZE_HINT)
    , allocated_pages(0)
{
    struct stat st;
    if (stat(filename.c_str(), &st) != 0) {
        throw_or_abort("Filename not found: " + filename);
    }
    size_t len = (size_t)st.st_size;
    size = len / sizeof(fr);
    max_size = size;
    int fd = open(filename.c_str(), O_RDONLY);
#ifndef __wasm__
    coefficients = (fr*)aligned_alloc(32, len);
    coefficients = (fr*)mmap(0, len, PROT_READ, MAP_PRIVATE, fd, 0);
#else
    ::read(fd, (void*)coefficients, len);
#endif
    close(fd);
}

polynomial::polynomial(const size_t initial_size, const size_t initial_max_size_hint, const Representation repr)
    : mapped(false)
    , coefficients(nullptr)
    , representation(repr)
    , size(initial_size)
    , page_size(DEFAULT_SIZE_HINT)
    , max_size(0)
    , allocated_pages(0)
{
    ASSERT(page_size != 0);
    size_t target_max_size = std::max(initial_size, initial_max_size_hint + DEFAULT_PAGE_SPILL);
    if (target_max_size > 0) {
        coefficients = (fr*)(aligned_alloc(32, sizeof(fr) * target_max_size));
        max_size = target_max_size;
    }
    zero_memory(max_size);
}

polynomial::polynomial(const polynomial& other, const size_t target_max_size)
    : mapped(false)
    , representation(other.representation)
    , size(other.size)
    , page_size(other.page_size)
    , max_size(std::max(clamp(target_max_size, page_size + DEFAULT_PAGE_SPILL), other.max_size))
    , allocated_pages(max_size / page_size)
{
    ASSERT(page_size != 0);
    coefficients = (fr*)(aligned_alloc(32, sizeof(fr) * max_size));

    if (other.coefficients != nullptr) {
        memcpy(static_cast<void*>(coefficients), static_cast<void*>(other.coefficients), sizeof(fr) * size);
    }
    zero_memory(max_size);
}

polynomial& polynomial::operator=(const polynomial& other)
{
    ASSERT(page_size != 0);
    mapped = false;
    representation = other.representation;
    page_size = other.page_size;
    size = other.size;
    allocated_pages = (max_size / page_size);
    if (other.max_size > max_size) {
        bump_memory(other.max_size);
    }
    if (other.coefficients != nullptr) {
        memcpy(static_cast<void*>(coefficients), static_cast<void*>(other.coefficients), sizeof(fr) * size);
    }
    zero_memory(max_size);
    return *this;
}

polynomial::polynomial(polynomial&& other, const size_t target_max_size)
    : mapped(other.mapped)
    , coefficients(other.coefficients)
    , representation(other.representation)
    , size(other.size)
    , page_size(other.page_size)
    , max_size(std::max(clamp(target_max_size, page_size + DEFAULT_PAGE_SPILL), other.max_size))
    , allocated_pages(max_size / page_size)
{
    ASSERT(page_size != 0);
    other.coefficients = nullptr;
    if (max_size > other.max_size) {
        bump_memory(max_size);
    }
    zero_memory(max_size);
}

polynomial& polynomial::operator=(polynomial&& other)
{
    free();

    mapped = other.mapped;
    coefficients = other.coefficients;
    representation = other.representation;
    page_size = other.page_size;
    max_size = other.max_size;
    allocated_pages = other.allocated_pages;
    size = other.size;
    ASSERT(page_size != 0);

    if (max_size > other.max_size) {
        bump_memory(max_size);
    }
    other.coefficients = nullptr;
    zero_memory(max_size);
    return *this;
}

polynomial::~polynomial()
{
    free();
}

// #######

fr polynomial::evaluate(const fr& z, const size_t target_size) const
{
    return polynomial_arithmetic::evaluate(coefficients, z, target_size);
}

void polynomial::zero_memory(const size_t zero_size)
{
    size_t delta = zero_size - size;
    if (delta > 0) {
        memset(static_cast<void*>(&coefficients[size]), 0, sizeof(fr) * delta);
    }
}

void polynomial::bump_memory(const size_t new_size_hint)
{
    ASSERT(!mapped);
    size_t new_size = (new_size_hint / page_size) * page_size;

    while (new_size < new_size_hint) {
        new_size += page_size;
    }

    fr* new_memory = (fr*)(aligned_alloc(32, sizeof(fr) * new_size));
    if (coefficients != nullptr) {
        memcpy(new_memory, coefficients, sizeof(fr) * size);
        aligned_free(coefficients);
    }
    coefficients = new_memory;
    allocated_pages = new_size / page_size;
    max_size = new_size;
}

void polynomial::add_coefficient_internal(const fr& coefficient)
{
    ASSERT(!mapped);
    if (size + 1 > max_size) {
        bump_memory((allocated_pages + 1) * page_size);
    }
    fr::__copy(coefficient, coefficients[size]);
    ++size;
}

void polynomial::add_lagrange_base_coefficient(const fr& coefficient)
{
    ASSERT(representation == Representation::ROOTS_OF_UNITY);
    add_coefficient_internal(coefficient);
}

void polynomial::add_coefficient(const fr& coefficient)
{
    ASSERT(representation == Representation::COEFFICIENT_FORM);
    add_coefficient_internal(coefficient);
}

void polynomial::free()
{
    if (coefficients != nullptr) {
#ifndef __wasm__
        if (mapped) {
            munmap(coefficients, size * sizeof(fr));
        } else {
            aligned_free(coefficients);
        }
#else
        aligned_free(coefficients);
#endif
    }
    coefficients = nullptr;
}

void polynomial::reserve(const size_t new_max_size)
{
    ASSERT(!mapped);
    if (new_max_size > max_size) {
        bump_memory(new_max_size);
        memset(static_cast<void*>(&coefficients[size]), 0, sizeof(fr) * (new_max_size - max_size));
    }
}

void polynomial::resize(const size_t new_size)
{
    ASSERT(!mapped);
    ASSERT(new_size > size);

    if (new_size > max_size) {
        bump_memory(new_size);
    }

    fr* back = &coefficients[size];
    memset(static_cast<void*>(back), 0, sizeof(fr) * (new_size - size));
    size = new_size;
}

// does not zero out memory
void polynomial::resize_unsafe(const size_t new_size)
{
    ASSERT(!mapped);
    // ASSERT(new_size > size);

    if (new_size > max_size) {
        bump_memory(new_size);
    }
    size = new_size;
}

/**
 * FFTs
 **/

void polynomial::fft(const evaluation_domain& domain)
{
    if (domain.size > max_size) {
        bump_memory(domain.size);
    }

    // (ZERO OUT MEMORY!)
    // TODO: wait, do we still need this?
    // memset(static_cast<void*>(back), 0, sizeof(fr) * (new_size - size));
    polynomial_arithmetic::fft(coefficients, domain);
    size = domain.size;
}

void polynomial::coset_fft(const evaluation_domain& domain)
{
    if (domain.size > max_size) {
        bump_memory(domain.size);
    }

    polynomial_arithmetic::coset_fft(coefficients, domain);
    size = domain.size;
}

void polynomial::coset_fft(const evaluation_domain& domain,
                           const evaluation_domain& large_domain,
                           const size_t domain_extension)
{
    if ((domain.size * domain_extension) > max_size) {
        bump_memory(domain.size * domain_extension);
    }

    polynomial_arithmetic::coset_fft(coefficients, domain, large_domain, domain_extension);
    size = (domain.size * domain_extension);
}

void polynomial::coset_fft_with_constant(const evaluation_domain& domain, const fr& constant)
{
    if (domain.size > max_size) {
        bump_memory(domain.size);
    }

    polynomial_arithmetic::coset_fft_with_constant(coefficients, domain, constant);
    size = domain.size;
}

void polynomial::coset_fft_with_generator_shift(const evaluation_domain& domain, const fr& constant)
{
    if (domain.size > max_size) {
        bump_memory(domain.size);
    }

    polynomial_arithmetic::coset_fft_with_generator_shift(coefficients, domain, constant);
    size = domain.size;
}

void polynomial::ifft(const evaluation_domain& domain)
{
    if (domain.size > max_size) {
        bump_memory(domain.size);
    }

    polynomial_arithmetic::ifft(coefficients, domain);
    size = domain.size;
}

void polynomial::ifft_with_constant(const evaluation_domain& domain, const barretenberg::fr& constant)
{
    if (domain.size > max_size) {
        bump_memory(domain.size);
    }

    polynomial_arithmetic::ifft_with_constant(coefficients, domain, constant);
    size = domain.size;
}

void polynomial::coset_ifft(const evaluation_domain& domain)
{
    if (domain.size > max_size) {
        bump_memory(domain.size);
    }

    polynomial_arithmetic::coset_ifft(coefficients, domain);
    size = domain.size;
}

// void polynomial::coset_ifft_with_constant(const evaluation_domain &domain, const fr &constant)
// {
//     if (domain.size > max_size)
//     {
//         bump_memory(domain.size);
//     }

//     polynomial_arithmetic::coset_ifft_with_constant(coefficients, domain, constant);
// }

fr polynomial::compute_kate_opening_coefficients(const barretenberg::fr& z)
{
    return polynomial_arithmetic::compute_kate_opening_coefficients(coefficients, coefficients, z, size);
}

fr polynomial::compute_barycentric_evaluation(const barretenberg::fr& z, const evaluation_domain& domain)
{
    return polynomial_arithmetic::compute_barycentric_evaluation(coefficients, domain.size, z, domain);
}

} // namespace barretenberg