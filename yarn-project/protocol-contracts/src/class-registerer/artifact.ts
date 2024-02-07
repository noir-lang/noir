import { loadContractArtifact } from '@aztec/types/abi';
import { NoirCompiledContract } from '@aztec/types/noir';

import ContractClassRegistererJson from '../artifacts/ContractClassRegisterer.json' assert { type: 'json' };

export const ContractClassRegistererArtifact = loadContractArtifact(
  ContractClassRegistererJson as NoirCompiledContract,
);
