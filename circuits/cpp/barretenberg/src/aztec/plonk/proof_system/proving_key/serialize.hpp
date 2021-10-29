#pragma once
#include "proving_key.hpp"
#include <polynomials/serialize.hpp>
#include <common/throw_or_abort.hpp>

namespace waffle {

template <typename B> inline void read(B& buf, proving_key_data& key)
{
    using serialize::read;
    read(buf, key.composer_type);
    read(buf, key.n);
    read(buf, key.num_public_inputs);
    read(buf, key.constraint_selectors);
    read(buf, key.constraint_selector_ffts);
    read(buf, key.permutation_selectors);
    read(buf, key.permutation_selectors_lagrange_base);
    read(buf, key.permutation_selector_ffts);
    read(buf, key.contains_recursive_proof);
    read(buf, key.recursive_proof_public_input_indices);
}

template <typename B> inline void write(B& buf, proving_key_data const& key)
{
    using serialize::write;
    write(buf, key.composer_type);
    write(buf, key.n);
    write(buf, key.num_public_inputs);
    write(buf, key.constraint_selectors);
    write(buf, key.constraint_selector_ffts);
    write(buf, key.permutation_selectors);
    write(buf, key.permutation_selectors_lagrange_base);
    write(buf, key.permutation_selector_ffts);
    write(buf, key.contains_recursive_proof);
    write(buf, key.recursive_proof_public_input_indices);
}

template <typename B> inline void write(B& buf, proving_key const& key)
{
    using serialize::write;
    write(buf, key.composer_type);
    write(buf, static_cast<uint32_t>(key.n));
    write(buf, static_cast<uint32_t>(key.num_public_inputs));
    write(buf, key.constraint_selectors);
    write(buf, key.constraint_selector_ffts);
    write(buf, key.permutation_selectors);
    write(buf, key.permutation_selectors_lagrange_base);
    write(buf, key.permutation_selector_ffts);
    write(buf, key.contains_recursive_proof);
    write(buf, key.recursive_proof_public_input_indices);
}

template <typename B> inline void read_mmap(B& it, std::string const& path, proving_key_data& key)
{
    using serialize::read;
    size_t file_num = 0;
    read(it, key.composer_type);
    read(it, key.n);
    read(it, key.num_public_inputs);
    for (auto map : { &key.constraint_selectors,
                      &key.constraint_selector_ffts,
                      &key.permutation_selectors,
                      &key.permutation_selectors_lagrange_base,
                      &key.permutation_selector_ffts }) {
        map->clear();
        uint32_t size;
        read(it, size);
        for (size_t i = 0; i < size; ++i) {
            std::string name;
            read(it, name);
            map->emplace(name, barretenberg::polynomial(format(path, "/", file_num++, "_", name)));
        }
    }
    read(it, key.contains_recursive_proof);
    read(it, key.recursive_proof_public_input_indices);
}

template <typename B> inline void write_mmap(B& buf, std::string const& path, proving_key const& key)
{
    using serialize::write;
    size_t file_num = 0;
    write(buf, key.composer_type);
    write(buf, static_cast<uint32_t>(key.n));
    write(buf, static_cast<uint32_t>(key.num_public_inputs));
    for (auto map : { &key.constraint_selectors,
                      &key.constraint_selector_ffts,
                      &key.permutation_selectors,
                      &key.permutation_selectors_lagrange_base,
                      &key.permutation_selector_ffts }) {
        write(buf, static_cast<uint32_t>(map->size()));
        for (auto& value : *map) {
            auto filename = format(path, "/", file_num++, "_", value.first);
            std::cerr << "Writing: " << filename << std::endl;
            write(buf, value.first);
            auto& p = value.second;
            auto size = p.get_size();
            std::ofstream os(filename);
            os.write((char*)&p[0], (std::streamsize)(size * sizeof(barretenberg::fr)));
            if (!os.good()) {
                throw_or_abort(format("Failed to write: ", filename));
            }
        }
    }
    write(buf, key.contains_recursive_proof);
    write(buf, key.recursive_proof_public_input_indices);
}

} // namespace waffle
