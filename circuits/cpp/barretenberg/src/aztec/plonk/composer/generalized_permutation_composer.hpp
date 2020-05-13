#pragma once
#include "turbo_composer.hpp"

namespace waffle {
class GenPermComposer : public TurboComposer {
  public:
    struct RangeList {
        uint64_t target_range;
        uint32_t range_tag;
        uint32_t tau_tag;
        std::vector<uint32_t> variable_indices;
    };
    GenPermComposer(const size_t size_hint = 0);

    UnrolledTurboProver create_unrolled_prover() override;
    UnrolledTurboVerifier create_unrolled_verifier() override;
    virtual std::shared_ptr<proving_key> compute_proving_key() override;
    virtual std::shared_ptr<verification_key> compute_verification_key() override;
    virtual TurboProver create_prover() override;
    GenPermVerifier create_verifier();

    void create_sort_constraint(const std::vector<uint32_t> variable_index);
    void create_sort_constraint_with_edges(const std::vector<uint32_t> variable_index, const fr, const fr);
    void assign_tag(const uint32_t variable_index, const uint32_t tag)
    {
        ASSERT(tag <= current_tag);
        ASSERT(variable_tags[get_real_variable_index(variable_index)] == DUMMY_TAG);
        variable_tags[get_real_variable_index(variable_index)] = tag;
    }

    uint32_t create_tag(const uint32_t tag_index, const uint32_t tau_index)
    {
        tau.insert({ tag_index, tau_index });
        current_tag++;
        return current_tag;
    }
    uint32_t get_new_tag()
    {
        current_tag++;
        return current_tag;
    }

    RangeList create_range_list(const uint64_t target_range);
    void create_range_constraint(const uint32_t variable_index, const uint64_t target_range);
    void process_range_list(const RangeList& list);

    std::map<uint64_t, RangeList> range_lists;

    static transcript::Manifest create_manifest(const size_t num_public_inputs)
    {
        // add public inputs....
        constexpr size_t g1_size = 64;
        constexpr size_t fr_size = 32;
        const size_t public_input_size = fr_size * num_public_inputs;
        const transcript::Manifest output = transcript::Manifest(
            { transcript::Manifest::RoundManifest(
                  { { "circuit_size", 4, true }, { "public_input_size", 4, true } }, "init", 1),
              transcript::Manifest::RoundManifest({ { "public_inputs", public_input_size, false },
                                                    { "W_1", g1_size, false },
                                                    { "W_2", g1_size, false },
                                                    { "W_3", g1_size, false },
                                                    { "W_4", g1_size, false } },
                                                  "beta",
                                                  2),
              transcript::Manifest::RoundManifest({ { "Z", g1_size, false } }, "alpha", 1),
              transcript::Manifest::RoundManifest({ { "T_1", g1_size, false },
                                                    { "T_2", g1_size, false },
                                                    { "T_3", g1_size, false },
                                                    { "T_4", g1_size, false } },
                                                  "z",
                                                  1),
              transcript::Manifest::RoundManifest(
                  { { "w_1", fr_size, false },       { "w_2", fr_size, false },       { "w_3", fr_size, false },
                    { "w_4", fr_size, false },       { "z_omega", fr_size, false },   { "sigma_1", fr_size, false },
                    { "sigma_2", fr_size, false },   { "sigma_3", fr_size, false },   { "id_1", fr_size, false },
                    { "id_2", fr_size, false },      { "id_3", fr_size, false },      { "id_4", fr_size, false },
                    { "q_arith", fr_size, false },   { "q_ecc_1", fr_size, false },   { "q_c", fr_size, false },
                    { "r", fr_size, false },         { "w_1_omega", fr_size, false }, { "w_2_omega", fr_size, false },
                    { "w_3_omega", fr_size, false }, { "w_4_omega", fr_size, false }, { "t", fr_size, true } },
                  "nu",
                  16,
                  true),
              transcript::Manifest::RoundManifest(
                  { { "PI_Z", g1_size, false }, { "PI_Z_OMEGA", g1_size, false } }, "separator", 1) });
        return output;
    }

    // static transcript::Manifest create_unrolled_manifest(const size_t num_public_inputs)
    // {
    //     // add public inputs....
    //     constexpr size_t g1_size = 64;
    //     constexpr size_t fr_size = 32;
    //     const size_t public_input_size = fr_size * num_public_inputs;
    //     const transcript::Manifest output = transcript::Manifest(
    //         { transcript::Manifest::RoundManifest(
    //               { { "circuit_size", 4, true }, { "public_input_size", 4, true } }, "init", 1),
    //           transcript::Manifest::RoundManifest({ { "public_inputs", public_input_size, false },
    //                                                 { "W_1", g1_size, false },
    //                                                 { "W_2", g1_size, false },
    //                                                 { "W_3", g1_size, false },
    //                                                 { "W_4", g1_size, false } },
    //                                               "beta",
    //                                               2),
    //           transcript::Manifest::RoundManifest({ { "Z", g1_size, false } }, "alpha", 1),
    //           transcript::Manifest::RoundManifest({ { "T_1", g1_size, false },
    //                                                 { "T_2", g1_size, false },
    //                                                 { "T_3", g1_size, false },
    //                                                 { "T_4", g1_size, false } },
    //                                               "z",
    //                                               1),
    //           transcript::Manifest::RoundManifest(
    //               {
    //                   { "w_1", fr_size, false },       { "w_2", fr_size, false },       { "w_3", fr_size, false },
    //                   { "w_4", fr_size, false },       { "w_1_omega", fr_size, false }, { "w_2_omega", fr_size, false
    //                   }, { "w_3_omega", fr_size, false }, { "w_4_omega", fr_size, false }, { "z", fr_size, false },
    //                   { "z_omega", fr_size, false },   { "sigma_1", fr_size, false },   { "sigma_2", fr_size, false
    //                   }, { "sigma_3", fr_size, false },   { "sigma_4", fr_size, false },   { "q_1", fr_size, false },
    //                   { "q_2", fr_size, false },       { "q_3", fr_size, false },       { "q_4", fr_size, false },
    //                   { "q_5", fr_size, false },       { "q_m", fr_size, false },       { "q_c", fr_size, false },
    //                   { "q_arith", fr_size, false },   { "q_logic", fr_size, false },   { "q_range", fr_size, false
    //                   }, { "q_ecc_1", fr_size, false },   { "t", fr_size, true },
    //               },
    //               "nu",koko; ,", 1) });
    //     return output;
    // }
};
} // namespace waffle
