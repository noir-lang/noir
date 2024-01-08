import { ContractArtifact } from '@aztec/aztec.js';

import SchnorrSingleKeyAccountContractJson from '../artifacts/SchnorrSingleKeyAccount.json' assert { type: 'json' };

export const SchnorrSingleKeyAccountContractArtifact = SchnorrSingleKeyAccountContractJson as ContractArtifact;
