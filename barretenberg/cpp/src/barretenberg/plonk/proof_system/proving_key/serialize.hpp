#pragma once
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/common/throw_or_abort.hpp"
#include "barretenberg/crypto/sha256/sha256.hpp"
#include "barretenberg/polynomials/serialize.hpp"
#include "proving_key.hpp"
#include <fcntl.h>
#include <ios>
#include <sys/stat.h>

namespace bb::plonk {

// Read the pre-computed polynomials
template <typename B> inline void read(B& any, proving_key_data& key)
{
    using serialize::read;
    using std::read;

    read(any, static_cast<uint32_t&>(key.circuit_type));
    read(any, static_cast<uint32_t&>(key.circuit_size));
    read(any, static_cast<uint32_t&>(key.num_public_inputs));

    uint32_t amount = 0;
    read(any, (uint32_t&)amount);

    for (size_t next = 0; next < amount; ++next) {
        std::string label;
        bb::polynomial value;

        read(any, label);
        read(any, value);

        key.polynomial_store.put(label, std::move(value));
    }

    read(any, key.contains_recursive_proof);
    read(any, key.recursive_proof_public_input_indices);
    read(any, key.memory_read_records);
    read(any, key.memory_write_records);
}

// Write the pre-computed polynomials
template <typename B> inline void write(B& buf, proving_key const& key)
{
    using serialize::write;
    write(buf, static_cast<uint32_t>(key.circuit_type));
    write(buf, static_cast<uint32_t>(key.circuit_size));
    write(buf, static_cast<uint32_t>(key.num_public_inputs));

    // Write only the pre-computed polys from the store
    PrecomputedPolyList precomputed_poly_list(key.circuit_type);
    size_t num_polys = precomputed_poly_list.size();
    write(buf, static_cast<uint32_t>(num_polys));

    for (size_t i = 0; i < num_polys; ++i) {
        std::string poly_id = precomputed_poly_list[i];
        const bb::polynomial& value = ((proving_key&)key).polynomial_store.get(poly_id);
        write(buf, poly_id);
        write(buf, value);
    }

    write(buf, key.contains_recursive_proof);
    write(buf, key.recursive_proof_public_input_indices);
    write(buf, key.memory_read_records);
    write(buf, key.memory_write_records);
}

template <typename B> inline void read_from_file(B& is, std::string const& path, proving_key_data& key)
{
    using serialize::read;

    size_t file_num = 0;
    read(is, key.circuit_type);
    read(is, key.circuit_size);
    read(is, key.num_public_inputs);

    uint32_t size;
    read(is, size);
    for (size_t i = 0; i < size; ++i) {
        std::string name;
        read(is, name);
        std::string filepath = format(path, "/", file_num++, "_", name);

        struct stat st;
        if (stat(filepath.c_str(), &st) != 0) {
            throw_or_abort("Filename not found: " + filepath);
        }
        size_t file_size = (size_t)st.st_size;
        size_t num_fields = file_size / 32;
        bb::polynomial value(num_fields);

        // Open the file and read the data directly into the polynomial memory.
        std::ifstream file(filepath, std::ios::binary);
        if (file) {
            file.read(reinterpret_cast<char*>(value.data().get()), (std::streamsize)file_size);
            file.close();
        } else {
            throw_or_abort("Failed to open file: " + filepath);
        }

        key.polynomial_store.put(name, std::move(value));
    }
    read(is, key.contains_recursive_proof);
    read(is, key.recursive_proof_public_input_indices);
    read(is, key.memory_read_records);
    read(is, key.memory_write_records);
}

template <typename B> inline void write_to_file(B& os, std::string const& path, proving_key& key)
{
    using serialize::write;

    size_t file_num = 0;
    write(os, static_cast<uint32_t>(key.circuit_type));
    write(os, static_cast<uint32_t>(key.circuit_size));
    write(os, static_cast<uint32_t>(key.num_public_inputs));

    // Write only the pre-computed polys from the store
    PrecomputedPolyList precomputed_poly_list(key.circuit_type);
    size_t num_polys = precomputed_poly_list.size();
    write(os, static_cast<uint32_t>(num_polys));

    for (size_t i = 0; i < num_polys; ++i) {
        std::string poly_id = precomputed_poly_list[i];
        auto filename = format(path, "/", file_num++, "_", poly_id);
        write(os, poly_id);
        auto value = key.polynomial_store.get(poly_id);
        auto size = value.size();
        std::ofstream ofs(filename);
        ofs.write((char*)value.data().get(), (std::streamsize)(size * sizeof(bb::fr)));
        if (!ofs.good()) {
            throw_or_abort(format("Failed to write: ", filename));
        }
    }
    write(os, key.contains_recursive_proof);
    write(os, key.recursive_proof_public_input_indices);
    write(os, key.memory_read_records);
    write(os, key.memory_write_records);
}

} // namespace bb::plonk
