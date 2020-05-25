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
    polynomial(const size_t initial_size = 0,
               const size_t initial_max_size_hint = DEFAULT_SIZE_HINT,
               const Representation repr = Representation::ROOTS_OF_UNITY);
    polynomial(const polynomial& other, const size_t target_max_size = 0);
    polynomial(polynomial&& other, const size_t target_max_size = 0);
    polynomial& operator=(polynomial&& other);
    polynomial& operator=(const polynomial& other);
    ~polynomial();

    bool operator==(polynomial const& rhs) const
    {
        bool eq = size == rhs.size;
        for (size_t i = 0; i < size; ++i) {
            eq &= coefficients[i] == rhs.coefficients[i];
        }
        return eq;
    }

    barretenberg::fr& operator[](const size_t i) const { return coefficients[i]; }

    // void copy(const polynomial &other, const size_t target_max_size = 0);
    barretenberg::fr* get_coefficients() const { return coefficients; };
    size_t get_size() const { return size; };
    size_t get_max_size() const { return max_size; };
    barretenberg::fr& at(const size_t i) const { return coefficients[i]; };
    barretenberg::fr evaluate(const barretenberg::fr& z, const size_t target_size) const;

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

  private:
    void free();
    void zero_memory(const size_t zero_size);
    const static size_t DEFAULT_SIZE_HINT = 1 << 12;
    const static size_t DEFAULT_PAGE_SPILL = 20;
    void add_coefficient_internal(const barretenberg::fr& coefficient);
    void bump_memory(const size_t new_size);

    bool mapped;
    barretenberg::fr* coefficients;
    Representation representation;
    size_t size;
    size_t page_size;
    size_t max_size;
    size_t allocated_pages;
};

template <typename B> inline void read(B& buf, polynomial& p)
{
    p = polynomial();
    uint32_t size;
    ::read(buf, size);
    p.reserve(size);
    for (size_t i = 0; i < size; ++i) {
        fr coeff;
        read(buf, coeff);
        p.add_coefficient(coeff);
    }
}

template <typename B> inline void write(B& buf, polynomial const& p)
{
    auto size = p.get_size();
    ::write(buf, static_cast<uint32_t>(size));

    for (size_t i = 0; i < size; ++i) {
        write(buf, p[i]);
    }
}

// Highly optimised read for loading large keys from disk.
template <> inline void read(std::istream& is, polynomial& p)
{
    p = polynomial();
    uint32_t size;
    ::read(is, size);
    p.resize_unsafe(size);

    auto len = (std::streamsize)(size * sizeof(fr));
    is.read((char*)&p[0], len);

#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
    for (size_t i = 0; i < size; ++i) {
        fr& c = p[i];
        std::swap(c.data[3], c.data[0]);
        std::swap(c.data[2], c.data[1]);
        c.data[3] = ntohll(c.data[3]);
        c.data[2] = ntohll(c.data[2]);
        c.data[1] = ntohll(c.data[1]);
        c.data[0] = ntohll(c.data[0]);
        c = c.to_montgomery_form();
    }
}

template <> inline void write(std::ostream& os, polynomial const& p)
{
    auto size = p.get_size();
    auto len = size * sizeof(fr);
    ::write(os, static_cast<uint32_t>(size));
    fr* cbuf = (fr*)aligned_alloc(64, len);
    memcpy(cbuf, &p[0], len);

#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
    for (size_t i = 0; i < size; ++i) {
        fr& c = cbuf[i];
        c = c.from_montgomery_form();
        std::swap(c.data[3], c.data[0]);
        std::swap(c.data[2], c.data[1]);
        c.data[3] = htonll(c.data[3]);
        c.data[2] = htonll(c.data[2]);
        c.data[1] = htonll(c.data[1]);
        c.data[0] = htonll(c.data[0]);
    }

    os.write((char*)cbuf, (std::streamsize)len);
    aligned_free(cbuf);
}

} // namespace barretenberg
