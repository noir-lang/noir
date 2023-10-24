#pragma once

#include "circuit_types.hpp"
#include "native_types.hpp"

#include <barretenberg/barretenberg.hpp>

#include <array>

namespace aztec3::utils::types {

using plonk::stdlib::witness_t;

namespace {

template <typename Builder> using CT = aztec3::utils::types::CircuitTypes<Builder>;
using NT = aztec3::utils::types::NativeTypes;

}  // namespace

/// TODO: Lots of identical functions here (but for their in/out types). Can we use templates? I couldn't figure out how
/// to keep the NT:: or CT:: prefixes with templates.
template <typename Builder> typename CT<Builder>::boolean to_ct(Builder& builder, typename NT::boolean const& e)
{
    return typename CT<Builder>::boolean(witness_t<Builder>(&builder, e));
};

template <typename Builder> typename CT<Builder>::fr to_ct(Builder& builder, typename NT::fr const& e)
{
    return typename CT<Builder>::fr(witness_t<Builder>(&builder, e));
};

template <typename Builder> typename CT<Builder>::fq to_ct(Builder& builder, typename NT::fq const& e)
{
    return typename CT<Builder>::fq(witness_t<Builder>(&builder, e));
};

template <typename Builder> typename CT<Builder>::address to_ct(Builder& builder, typename NT::address const& e)
{
    return typename CT<Builder>::address(witness_t<Builder>(&builder, e));
};

template <typename Builder> typename CT<Builder>::uint32 to_ct(Builder& builder, typename NT::uint32 const& e)
{
    return typename CT<Builder>::uint32(witness_t<Builder>(&builder, e));
};

template <typename Builder>
typename CT<Builder>::grumpkin_point to_ct(Builder& builder, typename NT::grumpkin_point const& e)
{
    return plonk::stdlib::cycle_group<Builder>::from_witness(builder, e);
};

template <typename Builder> typename CT<Builder>::bn254_point to_ct(Builder& builder, typename NT::bn254_point const& e)
{
    return CT<Builder>::bn254_point::from_witness(&builder, e);
};

template <typename Builder>
typename CT<Builder>::ecdsa_signature to_ct(Builder& builder, typename NT::ecdsa_signature const& e)
{
    return CT<Builder>::ecdsa_signature::template from_witness<Builder>(&builder, e);
};

template <typename Builder>
std::optional<typename CT<Builder>::boolean> to_ct(Builder& builder, std::optional<typename NT::boolean> const& e)
{
    return e ? std::make_optional<typename CT<Builder>::boolean>(to_ct(builder, *e)) : std::nullopt;
};

template <typename Builder>
std::optional<typename CT<Builder>::fr> to_ct(Builder& builder, std::optional<typename NT::fr> const& e)
{
    return e ? std::make_optional<typename CT<Builder>::fr>(to_ct(builder, *e)) : std::nullopt;
};

template <typename Builder>
std::optional<typename CT<Builder>::address> to_ct(Builder& builder, std::optional<typename NT::address> const& e)
{
    return e ? std::make_optional<typename CT<Builder>::address>(to_ct(builder, *e)) : std::nullopt;
};

template <typename Builder>
std::optional<typename CT<Builder>::grumpkin_point> to_ct(Builder& builder,
                                                          std::optional<typename NT::grumpkin_point> const& e)
{
    return e ? std::make_optional<typename CT<Builder>::grumpkin_point>(to_ct(builder, *e)) : std::nullopt;
};

template <typename Builder>
std::optional<typename CT<Builder>::ecdsa_signature> to_ct(Builder& builder,
                                                           std::optional<typename NT::ecdsa_signature> const& e)
{
    return e ? std::make_optional<typename CT<Builder>::ecdsa_signature>(to_ct(&builder, e)) : std::nullopt;
};

template <typename Builder>
std::vector<typename CT<Builder>::fr> to_ct(Builder& builder, std::vector<typename NT::fr> const& vec)
{
    auto ref_to_ct = [&](typename NT::fr const& e) { return to_ct(builder, e); };

    return map(vec, ref_to_ct);
};

template <typename Builder>
std::optional<std::vector<typename CT<Builder>::fr>> to_ct(Builder& builder,
                                                           std::optional<std::vector<typename NT::fr>> const& vec)
{
    auto ref_to_ct = [&](typename NT::fr const& e) { return to_ct(builder, e); };

    return vec ? std::make_optional<std::vector<typename CT<Builder>::fr>>(map(*vec, ref_to_ct)) : std::nullopt;
};

template <typename Builder, std::size_t SIZE>
std::array<typename CT<Builder>::fr, SIZE> to_ct(Builder& builder, std::array<typename NT::fr, SIZE> const& arr)
{
    auto ref_to_ct = [&](typename NT::fr const& e) { return to_ct(builder, e); };

    return map(arr, ref_to_ct);
};


template <typename Builder, std::size_t SIZE> std::array<std::optional<typename CT<Builder>::fr>, SIZE> to_ct(
    Builder& builder, std::array<std::optional<typename NT::fr>, SIZE> const& arr)
{
    auto ref_to_ct = [&](std::optional<typename NT::fr> const& e) { return to_ct(builder, e); };

    return map(arr, ref_to_ct);
};

/**
 * @brief Convert from an array of any native types (NT_TYPE) to array of circuit types (CT_TYPE)
 */
template <typename Builder, typename CT_TYPE, typename NT_TYPE, std::size_t SIZE>
std::array<CT_TYPE, SIZE> to_ct(Builder& builder, std::array<NT_TYPE, SIZE> const& arr)
{
    auto ref_to_ct = [&](NT_TYPE const& e) { return e.to_circuit_type(builder); };

    return map(arr, ref_to_ct);
};

/**
 * @brief Convert from an array of any native types (NT_TYPE) to array of circuit types (CT_TYPE).
 * Allow array entries to be optional.
 */
template <typename Builder, typename CT_TYPE, typename NT_TYPE, std::size_t SIZE>
std::array<std::optional<CT_TYPE>, SIZE> to_ct(Builder& builder, std::array<std::optional<NT_TYPE>, SIZE> const& arr)
{
    auto ref_to_ct = [&](std::optional<NT_TYPE> const& e) { return e.to_circuit_type(builder); };

    return map(arr, ref_to_ct);
};

// to_nt() below ********************************

template <typename Builder> typename NT::boolean to_nt(typename CT<Builder>::boolean const& e)
{
    return e.get_value();
};

template <typename Builder> typename NT::fr to_nt(typename CT<Builder>::fr const& e)
{
    return e.get_value();
};

template <typename Builder> typename NT::fq to_nt(typename CT<Builder>::fq const& e)
{
    return e.get_value();
};

template <typename Builder> typename NT::address to_nt(typename CT<Builder>::address const& e)
{
    return NT::address(e.address_.get_value());  // TODO: add get_value() method to address types.
};

template <typename Builder> typename NT::uint32 to_nt(typename CT<Builder>::uint32 const& e)
{
    NT::uint256 const e_256 = e.get_value();
    NT::uint64 const e_64 = e_256.data[0];  // TODO: check that this endianness is correct!
    auto const e_32 = static_cast<NT::uint32>(e_64);
    return e_32;
};

template <typename Builder> typename NT::grumpkin_point to_nt(typename CT<Builder>::grumpkin_point const& e)
{
    return NT::grumpkin_point{ e.x.get_value(), e.y.get_value() };
};

template <typename Builder> typename NT::bn254_point to_nt(typename CT<Builder>::bn254_point const& e)
{
    return e.get_value();
};

template <typename Builder> typename NT::ecdsa_signature to_nt(typename CT<Builder>::ecdsa_signature const& e)
{
    std::vector<uint8_t> r_bytes = e.r.get_value();
    std::vector<uint8_t> s_bytes = e.s.get_value();
    const uint8_t v_byte = e.v.get_value();

    std::array<uint8_t, 32> r_array;
    std::array<uint8_t, 32> s_array;
    std::copy(r_bytes.begin(), r_bytes.end(), r_array.begin());
    std::copy(s_bytes.begin(), s_bytes.end(), s_array.begin());

    return NT::ecdsa_signature{ r_array, s_array, v_byte };
};

template <typename Builder>
std::optional<typename NT::boolean> to_nt(std::optional<typename CT<Builder>::boolean> const& e)
{
    return e ? std::make_optional<typename NT::boolean>(to_nt<Builder>(*e)) : std::nullopt;
};

template <typename Builder> std::optional<typename NT::fr> to_nt(std::optional<typename CT<Builder>::fr> const& e)
{
    return e ? std::make_optional<typename NT::fr>(to_nt<Builder>(*e)) : std::nullopt;
};

template <typename Builder>
std::optional<typename NT::address> to_nt(std::optional<typename CT<Builder>::address> const& e)
{
    return e ? std::make_optional<typename NT::address>(to_nt<Builder>(*e)) : std::nullopt;
};

template <typename Builder>
std::optional<typename NT::grumpkin_point> to_nt(std::optional<typename CT<Builder>::grumpkin_point> const& e)
{
    return e ? std::make_optional<typename NT::grumpkin_point>(to_nt<Builder>(*e)) : std::nullopt;
};

template <typename Builder>
std::optional<typename NT::ecdsa_signature> to_nt(std::optional<typename CT<Builder>::ecdsa_signature> const& e)
{
    return e ? std::make_optional<typename NT::ecdsa_signature>(to_nt<Builder>(*e)) : std::nullopt;
};

template <typename Builder> std::vector<typename NT::fr> to_nt(std::vector<typename CT<Builder>::fr> const& vec)
{
    auto ref_to_nt = [&](typename CT<Builder>::fr const& e) { return to_nt<Builder>(e); };

    return map(vec, ref_to_nt);
};

template <typename Builder>
std::optional<std::vector<typename NT::fr>> to_nt(std::optional<std::vector<typename CT<Builder>::fr>> const& vec)
{
    auto ref_to_nt = [&](typename CT<Builder>::fr const& e) { return to_nt<Builder>(e); };

    return vec ? std::make_optional<std::vector<typename NT::fr>>(map(*vec, ref_to_nt)) : std::nullopt;
};

template <typename Builder, std::size_t SIZE>
std::array<typename NT::fr, SIZE> to_nt(std::array<typename CT<Builder>::fr, SIZE> const& arr)
{
    auto ref_to_nt = [&](typename CT<Builder>::fr const& e) { return to_nt<Builder>(e); };

    return map(arr, ref_to_nt);
};

// template <typename Builder, std::size_t SIZE>
// std::optional<std::array<typename NT::fr, SIZE>> to_nt(
//     std::optional<std::array<typename CT<Builder>::fr, SIZE>> const& arr)
// {
//     auto ref_to_nt = [&](typename CT<Builder>::fr const& e) { return to_nt(e); };

//     return arr ? std::make_optional<std::array<typename NT::fr, SIZE>>(map(arr, ref_to_nt)) : std::nullopt;
// };

template <typename Builder, std::size_t SIZE> std::array<std::optional<typename NT::fr>, SIZE> to_nt(
    std::array<std::optional<typename CT<Builder>::fr>, SIZE> const& arr)
{
    auto ref_to_nt = [&](std::optional<typename CT<Builder>::fr> const& e) { return to_nt<Builder>(e); };

    return map(arr, ref_to_nt);
};


}  // namespace aztec3::utils::types
