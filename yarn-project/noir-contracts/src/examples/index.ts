// TODO the verification keys in this contracts are mocked ATM
import TestContractJson from './test_contract.json' assert { type: 'json' };
import ZkTokenContractJson from './zk_token_contract.json' assert { type: 'json' };
import ParentJson from './parent_contract.json' assert { type: 'json' };
import ChildJson from './child_contract.json' assert { type: 'json' };
import PublicTokenContractJson from './public_token_contract.json' assert { type: 'json' };
import PublicToPrivateContractJson from './public_private_contract.json' assert { type: 'json' };
import { ContractAbi } from '@aztec/foundation/abi' assert { type: 'json' };
import NonNativeTokenContractJson from './non_native_token_contract.json' assert { type: 'json' };
import AccountContractJson from './account_contract.json' assert { type: 'json' };

export const TestContractAbi = TestContractJson as ContractAbi;
export const ZkTokenContractAbi = ZkTokenContractJson as ContractAbi;
export const ParentAbi = ParentJson as ContractAbi;
export const ChildAbi = ChildJson as ContractAbi;
export const PublicTokenContractAbi = PublicTokenContractJson as ContractAbi;
export const PublicToPrivateContractAbi = PublicToPrivateContractJson as ContractAbi;
export const NonNativeTokenContractAbi = NonNativeTokenContractJson as ContractAbi;
export const AccountContractAbi = AccountContractJson as ContractAbi;
