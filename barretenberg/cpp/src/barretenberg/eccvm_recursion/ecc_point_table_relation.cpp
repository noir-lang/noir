#include "barretenberg/eccvm_recursion/eccvm_recursive_flavor.hpp"
#include "barretenberg/flavor/relation_definitions.hpp"
#include "barretenberg/relations/ecc_vm/ecc_point_table_relation_impl.hpp"
#include "barretenberg/stdlib/primitives/bigfield/bigfield.hpp"

namespace bb {
template class ECCVMPointTableRelationImpl<stdlib::bigfield<UltraCircuitBuilder, bb::Bn254FqParams>>;
template class ECCVMPointTableRelationImpl<stdlib::bigfield<MegaCircuitBuilder, bb::Bn254FqParams>>;
DEFINE_SUMCHECK_VERIFIER_RELATION_CLASS(ECCVMPointTableRelationImpl, ECCVMRecursiveFlavor_<UltraCircuitBuilder>);
DEFINE_SUMCHECK_VERIFIER_RELATION_CLASS(ECCVMPointTableRelationImpl, ECCVMRecursiveFlavor_<MegaCircuitBuilder>);
} // namespace bb
