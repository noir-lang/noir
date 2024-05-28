#include "barretenberg/eccvm_recursion/eccvm_recursive_flavor.hpp"
#include "barretenberg/relations/ecc_vm/ecc_lookup_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_msm_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_point_table_relation.hpp"
#include "barretenberg/stdlib/primitives/bigfield/bigfield.hpp"
#include <gtest/gtest.h>
namespace bb {

// TODO(https://github.com/AztecProtocol/barretenberg/issues/997): Actually create consistency tests for ECCVM relations
class EccRelationsConsistency : public testing::Test {
  public:
    /**
     * @brief Validate that we can instantiate ECCVM relations on bigfield and that they return the same values as their
     * native counterpart instantiated on grumpkin::fr.
     *
     * @tparam Relation the Relation whose instantiation on stdlib and native we want to test
     */
    template <template <typename> class Relation> static void validate_relation_execution()
    {
        auto builder = UltraCircuitBuilder();
        using RecursiveFlavor = ECCVMRecursiveFlavor_<UltraCircuitBuilder>;
        using RecursiveRelation = Relation<typename RecursiveFlavor::FF>;
        const RelationParameters<typename RecursiveFlavor::FF> parameters;
        RecursiveFlavor::AllValues input_elements;
        for (auto& element : input_elements.get_all()) {
            element = 4;
        }
        typename RecursiveRelation::SumcheckArrayOfValuesOverSubrelations accumulator;
        std::fill(accumulator.begin(), accumulator.end(), typename RecursiveRelation::FF(0));
        RecursiveRelation::accumulate(accumulator, input_elements, parameters, 1);

        using NativeFlavor = ECCVMFlavor;
        using NativeRelation = Relation<typename NativeFlavor::FF>;
        const RelationParameters<typename NativeFlavor::FF> native_parameters;
        NativeFlavor::AllValues native_input_elements;
        for (auto& element : native_input_elements.get_all()) {
            element = 4;
        }
        typename NativeRelation::SumcheckArrayOfValuesOverSubrelations native_accumulator;
        std::fill(native_accumulator.begin(), native_accumulator.end(), typename NativeRelation::FF(0));
        NativeRelation::accumulate(native_accumulator, native_input_elements, native_parameters, 1);

        for (auto [val, native_val] : zip_view(accumulator, native_accumulator)) {
            EXPECT_EQ(bb::fq((val.get_value() % uint512_t(bb::fq::modulus)).lo), uint256_t(native_val));
        }
    };
};

TEST_F(EccRelationsConsistency, RecursiveToNativeConsistency)
{

    validate_relation_execution<ECCVMLookupRelation>();
    validate_relation_execution<ECCVMSetRelation>();
    validate_relation_execution<ECCVMMSMRelation>();
    validate_relation_execution<ECCVMPointTableRelation>();
    validate_relation_execution<ECCVMTranscriptRelation>();
    validate_relation_execution<ECCVMWnafRelation>();
    validate_relation_execution<ECCVMBoolsRelation>();
}
} // namespace bb