#pragma once

#include "barretenberg/crypto/poseidon2/poseidon2.hpp"
#include "barretenberg/stdlib/hash/poseidon2/poseidon2.hpp"
#include "barretenberg/stdlib/primitives/field/field_conversion.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb::stdlib::recursion::honk {

template <typename Builder> struct StdlibTranscriptParams {
    using Fr = stdlib::field_t<Builder>;
    using Proof = std::vector<Fr>;

    static inline Fr hash(const std::vector<Fr>& data)
    {

        ASSERT(!data.empty() && data[0].get_context() != nullptr);

        Builder* builder = data[0].get_context();
        return stdlib::poseidon2<Builder>::hash(*builder, data);
    }

    template <typename T> static inline T convert_challenge(const Fr& challenge)
    {
        Builder* builder = challenge.get_context();
        return bb::stdlib::field_conversion::convert_challenge<Builder, T>(*builder, challenge);
    }

    template <typename T> static constexpr size_t calc_num_bn254_frs()
    {
        return bb::stdlib::field_conversion::calc_num_bn254_frs<Builder, T>();
    }

    template <typename T> static inline T convert_from_bn254_frs(std::span<const Fr> frs)
    {
        ASSERT(!frs.empty() && frs[0].get_context() != nullptr);
        Builder* builder = frs[0].get_context();
        return bb::stdlib::field_conversion::convert_from_bn254_frs<Builder, T>(*builder, frs);
    }

    template <typename T> static inline std::vector<Fr> convert_to_bn254_frs(const T& element)
    {
        Builder* builder = element.get_context();
        return bb::stdlib::field_conversion::convert_to_bn254_frs<Builder, T>(*builder, element);
    }
};

using UltraStdlibTranscript = BaseTranscript<StdlibTranscriptParams<UltraCircuitBuilder>>;
using MegaStdlibTranscript = BaseTranscript<StdlibTranscriptParams<MegaCircuitBuilder>>;
} // namespace bb::stdlib::recursion::honk