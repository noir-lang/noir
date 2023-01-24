#pragma once
#include "proving_key.hpp"
#include <polynomials/serialize.hpp>
#include <common/throw_or_abort.hpp>
#include <common/serialize.hpp>

namespace waffle {

// Read the pre-computed polynomials
template <typename B> inline void read(B& any, proving_key_data& key)
{
    using serialize::read;
    using std::read;

    read(any, key.composer_type);
    read(any, (uint32_t&)key.n);
    read(any, (uint32_t&)key.num_public_inputs);

    uint32_t amount = 0;
    read(any, (uint32_t&)amount);

    for (size_t next = 0; next < amount; ++next) {
        std::string label;
        barretenberg::polynomial value;

        read(any, label);
        read(any, value);

        key.polynomial_cache.put(label, std::move(value));
    }

    read(any, key.contains_recursive_proof);
    read(any, key.recursive_proof_public_input_indices);
}

// Write the pre-computed polynomials
template <typename B> inline void write(B& buf, proving_key const& key)
{
    using serialize::write;
    write(buf, key.composer_type);
    write(buf, (uint32_t)key.n);
    write(buf, (uint32_t)key.num_public_inputs);

    // Write only the pre-computed polys from the store
    PrecomputedPolyList precomputed_poly_list(key.composer_type);
    size_t num_polys = precomputed_poly_list.size();
    write(buf, static_cast<uint32_t>(num_polys));

    for (size_t i = 0; i < num_polys; ++i) {
        std::string poly_id = precomputed_poly_list[i];
        const barretenberg::polynomial& value = ((proving_key&)key).polynomial_cache.get(poly_id);
        write(buf, poly_id);
        write(buf, value);
    }

    write(buf, key.contains_recursive_proof);
    write(buf, key.recursive_proof_public_input_indices);
}

template <typename B> inline void read_mmap(B& is, std::string const& path, proving_key_data& key)
{
    using serialize::read;

    size_t file_num = 0;
    read(is, key.composer_type);
    read(is, key.n);
    read(is, key.num_public_inputs);

    uint32_t size;
    read(is, size);
    for (size_t i = 0; i < size; ++i) {
        std::string name;
        read(is, name);
        barretenberg::polynomial value(format(path, "/", file_num++, "_", name));
        key.polynomial_cache.put(name, std::move(value));
    }
    read(is, key.contains_recursive_proof);
    read(is, key.recursive_proof_public_input_indices);
}

// inline void write_mmap(std::ostream& os, std::string const& path, proving_key const& key)
template <typename B> inline void write_mmap(B& os, std::string const& path, proving_key const& key)
{
    using serialize::write;

    size_t file_num = 0;
    write(os, key.composer_type);
    write(os, static_cast<uint32_t>(key.n));
    write(os, static_cast<uint32_t>(key.num_public_inputs));

    // Write only the pre-computed polys from the store
    PrecomputedPolyList precomputed_poly_list(key.composer_type);
    size_t num_polys = precomputed_poly_list.size();
    write(os, static_cast<uint32_t>(num_polys));

    for (size_t i = 0; i < num_polys; ++i) {
        std::string poly_id = precomputed_poly_list[i];
        auto filename = format(path, "/", file_num++, "_", poly_id);
        write(os, poly_id);
        const barretenberg::polynomial& value = ((proving_key&)key).polynomial_cache.get(poly_id);
        auto size = value.get_size();
        std::ofstream ofs(filename);
        ofs.write((char*)&value[0], (std::streamsize)(size * sizeof(barretenberg::fr)));
        if (!ofs.good()) {
            throw_or_abort(format("Failed to write: ", filename));
        }
    }
    write(os, key.contains_recursive_proof);
    write(os, key.recursive_proof_public_input_indices);
}

} // namespace waffle
