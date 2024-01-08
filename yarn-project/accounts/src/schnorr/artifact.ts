import { ContractArtifact } from '@aztec/aztec.js';

import SchnorrAccountContractJson from '../artifacts/SchnorrAccount.json' assert { type: 'json' };

export const SchnorrAccountContractArtifact = SchnorrAccountContractJson as ContractArtifact;
