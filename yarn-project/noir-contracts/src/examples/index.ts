// TODO the verification keys in this contracts are mocked ATM
import { ContractAbi } from '@aztec/foundation/abi' assert { type: 'json' };

import TestContractJson from './test_contract.json' assert { type: 'json' };
import ZkTokenContractJson from './zk_token_contract.json' assert { type: 'json' };
import ParentJson from './parent_contract.json' assert { type: 'json' };
import ChildJson from './child_contract.json' assert { type: 'json' };
import PublicTokenContractJson from './public_token_contract.json' assert { type: 'json' };
import NonNativeTokenContractJson from './non_native_token_contract.json' assert { type: 'json' };
import EcdsaAccountContractJson from './ecdsa_account_contract.json' assert { type: 'json' };
import SchnorrAccountContractJson from './schnorr_account_contract.json' assert { type: 'json' };
import GullibleAccountContractJson from './gullible_account_contract.json' assert { type: 'json' };
import UniswapContractJson from './uniswap_contract.json' assert { type: 'json' };

export const TestContractAbi = TestContractJson as ContractAbi;
export const ZkTokenContractAbi = ZkTokenContractJson as ContractAbi;
export const ParentAbi = ParentJson as ContractAbi;
export const ChildAbi = ChildJson as ContractAbi;
export const PublicTokenContractAbi = PublicTokenContractJson as ContractAbi;
export const NonNativeTokenContractAbi = NonNativeTokenContractJson as ContractAbi;
export const EcdsaAccountContractAbi = EcdsaAccountContractJson as ContractAbi;
export const SchnorrAccountContractAbi = SchnorrAccountContractJson as ContractAbi;
export const GullibleAccountContractAbi = GullibleAccountContractJson as ContractAbi;
export const UniswapContractAbi = UniswapContractJson as ContractAbi;
