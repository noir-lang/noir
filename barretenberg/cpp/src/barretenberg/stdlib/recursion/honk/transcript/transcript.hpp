#pragma once

#include "barretenberg/transcript/transcript.hpp"

namespace bb::stdlib::recursion::honk {

template <typename Builder> struct StdlibTranscriptParams {
    using Fr = stdlib::field_t<Builder>;
    using Proof = std::vector<Fr>;
    static inline Fr hash(const std::vector<Fr>& data)
    {
        if constexpr (std::is_same_v<Builder, GoblinUltraCircuitBuilder>) {
            ASSERT(!data.empty() && data[0].get_context() != nullptr);
            Builder* builder = data[0].get_context();
            return stdlib::poseidon2<Builder>::hash(*builder, data);
        } else {
            using NativeFr = bb::fr;
            ASSERT(!data.empty() && data[0].get_context() != nullptr);
            Builder* builder = data[0].get_context();

            // call the native hash on the data
            std::vector<NativeFr> native_data;
            native_data.reserve(data.size());
            for (const auto& fr : data) {
                native_data.push_back(fr.get_value());
            }
            NativeFr hash_value = crypto::Poseidon2<crypto::Poseidon2Bn254ScalarFieldParams>::hash(native_data);

            Fr hash_field_ct = Fr::from_witness(builder, hash_value);
            return hash_field_ct;
        }
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
using GoblinUltraStdlibTranscript = BaseTranscript<StdlibTranscriptParams<GoblinUltraCircuitBuilder>>;
} // namespace bb::stdlib::recursion::honk