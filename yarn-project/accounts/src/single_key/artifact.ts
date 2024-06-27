import { type NoirCompiledContract, loadContractArtifact } from '@aztec/aztec.js';

import SchnorrSingleKeyAccountContractJson from '../../artifacts/SchnorrSingleKeyAccount.json' assert { type: 'json' };

export const SchnorrSingleKeyAccountContractArtifact = loadContractArtifact(
  SchnorrSingleKeyAccountContractJson as NoirCompiledContract,
);
