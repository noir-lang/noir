#pragma once
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/serialize/msgpack.hpp"

#include <array>
#include <cstdint>
#include <vector>

namespace acir_format {

struct BigIntFromLeBytes {
    std::vector<uint32_t> inputs;
    std::vector<uint32_t> modulus;
    uint32_t result;

    // For serialization, update with any new fields
    MSGPACK_FIELDS(inputs, result);
    friend bool operator==(BigIntFromLeBytes const& lhs, BigIntFromLeBytes const& rhs) = default;
};

enum BigIntOperationType { Add, Sub, Mul, Div };

struct BigIntOperation {
    uint32_t lhs;
    uint32_t rhs;
    uint32_t result;
    BigIntOperationType opcode;

    // For serialization, update with any new fields
    MSGPACK_FIELDS(lhs, rhs, opcode, result);
    friend bool operator==(BigIntOperation const& lhs, BigIntOperation const& rhs) = default;
};

struct BigIntToLeBytes {
    uint32_t input;
    std::vector<uint32_t> result;

    // For serialization, update with any new fields
    MSGPACK_FIELDS(input, result);
    friend bool operator==(BigIntToLeBytes const& lhs, BigIntToLeBytes const& rhs) = default;
};

/// Enumerates the supported modulus types for big integer operations.
/// Specifies whether a bigint refers to a BN254/SECP256K1/SECP256R1 Fq or Fr modulus.
enum ModulusId {
    BN254_FQ = 0,
    BN254_FR,
    SECP256K1_FQ,
    SECP256K1_FR,
    SECP256R1_FQ,
    SECP256R1_FR,
    UNKNOWN,
};

/// 256-bit modulus value for a field element
/// The modulus is represented by 4 64-bits limbs
/// Used to define the modulus for big integer operations.
class ModulusParam {
  public:
    uint64_t modulus_0;
    uint64_t modulus_1;
    uint64_t modulus_2;
    uint64_t modulus_3;
};

template <typename Builder> class DSLBigInts {
    using big_bn254_fq = bb::stdlib::bigfield<Builder, bb::Bn254FqParams>;
    using big_bn254_fr = bb::stdlib::bigfield<Builder, bb::Bn254FrParams>;
    using big_secp256k1_fq = bb::stdlib::bigfield<Builder, bb::secp256k1::FqParams>;
    using big_secp256k1_fr = bb::stdlib::bigfield<Builder, bb::secp256k1::FrParams>;
    using big_secp256r1_fq = bb::stdlib::bigfield<Builder, bb::secp256r1::FqParams>;
    using big_secp256r1_fr = bb::stdlib::bigfield<Builder, bb::secp256r1::FrParams>;

  private:
    std::map<uint32_t, big_bn254_fq> m_bn254_fq;
    std::map<uint32_t, big_bn254_fr> m_bn254_fr;
    std::map<uint32_t, big_secp256k1_fq> m_secp256k1_fq;
    std::map<uint32_t, big_secp256k1_fr> m_secp256k1_fr;
    std::map<uint32_t, big_secp256r1_fq> m_secp256r1_fq;
    std::map<uint32_t, big_secp256r1_fr> m_secp256r1_fr;

    Builder* builder;

  public:
    DSLBigInts() = default;

    void set_builder(Builder* ctx) { builder = ctx; }

    ModulusId get_modulus_id(uint32_t bigint_id)
    {
        if (this->m_bn254_fq.contains(bigint_id)) {
            return ModulusId::BN254_FQ;
        }
        if (this->m_bn254_fr.contains(bigint_id)) {
            return ModulusId::BN254_FR;
        }
        if (this->m_secp256k1_fq.contains(bigint_id)) {
            return ModulusId::SECP256K1_FQ;
        }
        if (this->m_secp256k1_fr.contains(bigint_id)) {
            return ModulusId::SECP256K1_FR;
        }
        if (this->m_secp256r1_fq.contains(bigint_id)) {
            return ModulusId::SECP256R1_FQ;
        }
        if (this->m_secp256r1_fr.contains(bigint_id)) {
            return ModulusId::SECP256R1_FR;
        }

        return ModulusId::UNKNOWN;
    }

    /// Set value of the witnesses representing the bigfield element
    /// so that the bigfield value is the input value.
    /// The input value is decomposed into the binary basis for the binary limbs
    /// The input array must be:
    /// the 4 witness index of the binary limbs, and the index of the prime limb
    void set_value(uint256_t value, const std::array<uint32_t, 5> limbs_idx)
    {
        uint256_t limb_modulus = uint256_t(1) << big_bn254_fq::NUM_LIMB_BITS;
        builder->variables[limbs_idx[4]] = value;
        for (uint32_t i = 0; i < 4; i++) {
            uint256_t limb = value % limb_modulus;
            value = (value - limb) / limb_modulus;
            builder->variables[limbs_idx[i]] = limb;
        }
    }

    /// Utility function that retrieve the witness indexes of a bigfield element
    /// for use in set_value()
    void get_witness_idx_of_limbs(uint32_t bigint_id, std::array<uint32_t, 5>& limbs_idx)
    {
        if (m_bn254_fr.contains(bigint_id)) {
            for (uint32_t i = 0; i < 4; i++) {
                limbs_idx[i] = m_bn254_fr[bigint_id].binary_basis_limbs[i].element.witness_index;
            }
            limbs_idx[4] = m_bn254_fr[bigint_id].prime_basis_limb.witness_index;
        } else if (m_bn254_fq.contains(bigint_id)) {
            for (uint32_t i = 0; i < 4; i++) {
                limbs_idx[i] = m_bn254_fq[bigint_id].binary_basis_limbs[i].element.witness_index;
            }
            limbs_idx[4] = m_bn254_fq[bigint_id].prime_basis_limb.witness_index;
        } else if (m_secp256k1_fq.contains(bigint_id)) {
            auto big_field = m_secp256k1_fq[bigint_id];
            for (uint32_t i = 0; i < 4; i++) {
                limbs_idx[i] = big_field.binary_basis_limbs[i].element.witness_index;
            }
            limbs_idx[4] = big_field.prime_basis_limb.witness_index;
        } else if (m_secp256k1_fr.contains(bigint_id)) {
            auto big_field = m_secp256k1_fr[bigint_id];
            for (uint32_t i = 0; i < 4; i++) {
                limbs_idx[i] = big_field.binary_basis_limbs[i].element.witness_index;
            }
            limbs_idx[4] = big_field.prime_basis_limb.witness_index;
        } else if (m_secp256r1_fr.contains(bigint_id)) {
            auto big_field = m_secp256r1_fr[bigint_id];
            for (uint32_t i = 0; i < 4; i++) {
                limbs_idx[i] = big_field.binary_basis_limbs[i].element.witness_index;
            }
            limbs_idx[4] = big_field.prime_basis_limb.witness_index;
        } else if (m_secp256r1_fq.contains(bigint_id)) {
            auto big_field = m_secp256r1_fq[bigint_id];
            for (uint32_t i = 0; i < 4; i++) {
                limbs_idx[i] = big_field.binary_basis_limbs[i].element.witness_index;
            }
            limbs_idx[4] = big_field.prime_basis_limb.witness_index;
        }
    }
    big_bn254_fr bn254_fr(uint32_t bigint_id)
    {
        if (this->m_bn254_fr.contains(bigint_id)) {
            return this->m_bn254_fr[bigint_id];
        }
        ASSERT(false);
        return { 0 };
    }

    void set_bn254_fr(const big_bn254_fr& bigint, uint32_t bigint_id) { this->m_bn254_fr[bigint_id] = bigint; }

    big_bn254_fq bn254_fq(uint32_t bigint_id)
    {
        if (this->m_bn254_fq.contains(bigint_id)) {
            return this->m_bn254_fq[bigint_id];
        }
        ASSERT(false);
        return { 0 };
    }

    void set_bn254_fq(const big_bn254_fq& bigint, uint32_t bigint_id) { this->m_bn254_fq[bigint_id] = bigint; }

    big_secp256r1_fq secp256r1_fq(uint32_t bigint_id)
    {
        if (this->m_secp256r1_fq.contains(bigint_id)) {
            return this->m_secp256r1_fq[bigint_id];
        }
        ASSERT(false);
        return { 0 };
    }

    void set_secp256r1_fq(const big_secp256r1_fq& bigint, uint32_t bigint_id)
    {
        this->m_secp256r1_fq[bigint_id] = bigint;
    }

    big_secp256r1_fr secp256r1_fr(uint32_t bigint_id)
    {
        if (this->m_secp256r1_fr.contains(bigint_id)) {
            return this->m_secp256r1_fr[bigint_id];
        }
        ASSERT(false);
        return { 0 };
    }

    void set_secp256r1_fr(const big_secp256r1_fr& bigint, uint32_t bigint_id)
    {
        this->m_secp256r1_fr[bigint_id] = bigint;
    }

    big_secp256k1_fq secp256k1_fq(uint32_t bigint_id)
    {
        if (this->m_secp256k1_fq.contains(bigint_id)) {
            return this->m_secp256k1_fq[bigint_id];
        }
        ASSERT(false);
        return { 0 };
    }

    void set_secp256k1_fq(const big_secp256k1_fq& bigint, uint32_t bigint_id)
    {
        this->m_secp256k1_fq[bigint_id] = bigint;
    }

    big_secp256k1_fr secp256k1_fr(uint32_t bigint_id)
    {
        if (this->m_secp256k1_fr.contains(bigint_id)) {
            return this->m_secp256k1_fr[bigint_id];
        }
        return { 0 };
    }

    void set_secp256k1_fr(const big_secp256k1_fr& bigint, uint32_t bigint_id)
    {
        this->m_secp256k1_fr[bigint_id] = bigint;
    }
};

template <typename Builder>
void create_bigint_from_le_bytes_constraint(Builder& builder,
                                            const BigIntFromLeBytes& input,
                                            DSLBigInts<Builder>& dsl_bigints);
template <typename Builder>
void create_bigint_to_le_bytes_constraint(Builder& builder,
                                          const BigIntToLeBytes& input,
                                          DSLBigInts<Builder>& dsl_bigints);

template <typename Builder>
void create_bigint_operations_constraint(const BigIntOperation& input, DSLBigInts<Builder>& dsl_bigints, bool);
template <typename Builder>
void create_bigint_addition_constraint(const BigIntOperation& input, DSLBigInts<Builder>& dsl_bigints);
template <typename Builder>
void create_bigint_sub_constraint(const BigIntOperation& input, DSLBigInts<Builder>& dsl_bigints);
template <typename Builder>
void create_bigint_mul_constraint(const BigIntOperation& input, DSLBigInts<Builder>& dsl_bigints);
template <typename Builder>
void create_bigint_div_constraint(const BigIntOperation& input, DSLBigInts<Builder>& dsl_bigints, bool);

} // namespace acir_format