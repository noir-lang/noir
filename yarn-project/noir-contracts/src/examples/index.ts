// TODO the verification keys in this contracts are mocked ATM
import TestContractJson from './test_contract.json';
import ZkTokenContractJson from './zk_token_contract.json';
import ParentJson from './parent_contract.json';
import ChildJson from './child_contract.json';
import PublicTokenContractJson from './public_token_contract.json';
import NonNativeTokenContractJson from './non_native_token_contract.json';
import RollupNativeAssetContractJson from './rollup_native_asset_contract.json';
import { ContractAbi } from '@aztec/foundation/abi';

export const TestContractAbi = TestContractJson as ContractAbi;
export const ZkTokenContractAbi = ZkTokenContractJson as ContractAbi;
export const ParentAbi = ParentJson as ContractAbi;
export const ChildAbi = ChildJson as ContractAbi;
export const PublicTokenContractAbi = PublicTokenContractJson as ContractAbi;
export const NonNativeTokenContractAbi = NonNativeTokenContractJson as ContractAbi;
export const RollupNativeAssetContractAbi = RollupNativeAssetContractJson as ContractAbi;
