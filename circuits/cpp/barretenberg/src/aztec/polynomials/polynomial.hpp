#pragma once
#include "evaluation_domain.hpp"
#include <common/mem.hpp>
#include <common/timer.hpp>
#include <fstream>

namespace barretenberg {
class polynomial {
  public:
    enum Representation { COEFFICIENT_FORM, ROOTS_OF_UNITY, COSET_ROOTS_OF_UNITY, NONE };

    // Creates a read only polynomial using mmap.
    polynomial(std::string const& filename);

    // TODO: add a 'spill' factor when allocating memory - we sometimes needs to extend poly degree by 2/4,
    // if page size = power of two, will trigger unneccesary copies
    polynomial(const size_t initial_size,
               const size_t initial_max_size_hint = DEFAULT_SIZE_HINT,
               const Representation repr = Representation::ROOTS_OF_UNITY);
    polynomial(const polynomial& other, const size_t target_max_size = 0);

    polynomial(polynomial&& other);

    // Takes ownership of given buffer.
    polynomial(fr* buf, const size_t initial_size);

    // Allow polynomials to be entirely reset/dormant
    polynomial();

    polynomial& operator=(polynomial&& other);
    polynomial& operator=(const polynomial& other);
    ~polynomial();

    void clear()
    {
        free();

        mapped = 0;
        coefficients = 0;
        representation = Representation::ROOTS_OF_UNITY;
        initial_size = 0;
        size = 0;
        page_size = DEFAULT_SIZE_HINT;
        max_size = 0;
        allocated_pages = 0;
    }

    bool operator==(polynomial const& rhs) const
    {
        if (size == rhs.size) {

            // If either poly has null coefficients then we are equal only if both are null
            if (coefficients == 0 || rhs.coefficients == 0)
                return coefficients == 0 && rhs.coefficients == 0;

            // Size is equal and both have coefficients, compare
            for (size_t i = 0; i < size; ++i) {
                if (coefficients[i] != rhs.coefficients[i])
                    return false;
            }

            return true;
        }

        return false;
    }

    barretenberg::fr* get_coefficients() const { return coefficients; };
    barretenberg::fr* get_coefficients() { return coefficients; };
    barretenberg::fr* data() { return coefficients; };

    size_t get_size() const { return size; };
    size_t get_max_size() const { return max_size; };

    // Const and non const versions of coefficient accessors
    barretenberg::fr& operator[](const size_t i) const
    {
        ASSERT(!empty());
        return coefficients[i];
    }

    barretenberg::fr& operator[](const size_t i)
    {
        ASSERT(!empty());
        return coefficients[i];
    }

    barretenberg::fr& at(const size_t i) const
    {
        ASSERT(!empty());
        return coefficients[i];
    };

    barretenberg::fr& at(const size_t i)
    {
        ASSERT(!empty());
        return coefficients[i];
    };

    barretenberg::fr evaluate(const barretenberg::fr& z, const size_t target_size) const;
    barretenberg::fr compute_barycentric_evaluation(const barretenberg::fr& z, const evaluation_domain& domain);

    void fft(const evaluation_domain& domain);
    void coset_fft(const evaluation_domain& domain);
    void coset_fft(const evaluation_domain& domain,
                   const evaluation_domain& large_domain,
                   const size_t domain_extension);

    void coset_fft_with_constant(const evaluation_domain& domain, const barretenberg::fr& costant);
    void coset_fft_with_generator_shift(const evaluation_domain& domain, const fr& constant);

    void ifft(const evaluation_domain& domain);
    void ifft_with_constant(const evaluation_domain& domain, const barretenberg::fr& constant);
    void coset_ifft(const evaluation_domain& domain);
    // void coset_ifft_with_constant(const evaluation_domain &domain, const barretenberg::fr &constant);

    barretenberg::fr compute_kate_opening_coefficients(const barretenberg::fr& z);
    void add_lagrange_base_coefficient(const barretenberg::fr& coefficient);
    void add_coefficient(const barretenberg::fr& coefficient);

    void reserve(const size_t new_max_size);
    void resize(const size_t new_size);
    void resize_unsafe(const size_t new_size);

    bool empty() const
    {
        static polynomial poly;
        return *this == poly;
    }

  private:
    void free();
    void zero_memory(const size_t zero_size);
    const static size_t DEFAULT_SIZE_HINT = 1 << 12;
    const static size_t DEFAULT_PAGE_SPILL = 20;
    void add_coefficient_internal(const barretenberg::fr& coefficient);
    void bump_memory(const size_t new_size);

  public:
    bool mapped;
    barretenberg::fr* coefficients;
    Representation representation;
    size_t initial_size;
    size_t size;
    size_t page_size;
    size_t max_size;
    size_t allocated_pages;
};

inline std::ostream& operator<<(std::ostream& os, polynomial const& p)
{
    return os << "[ " << p[0] << ", ... ]";
}

} // namespace barretenberg
