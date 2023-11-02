// TODO(David): The cpp code uses a typename. I've used a new module
// to make it more obvious that we have NewContractData being aliased
// as ContractLeafPreimage.
// It may make sense to only use `ContractLeafPreimage` in the codebase.
//
// This is not actually being used anywhere due to Noir not
// resolving import path aliases and re-exports properly.
// If you grep for `ContractLeafPreimage` you can see its usage.
use crate::abis::new_contract_data::NewContractData as ContractLeafPreimage;
