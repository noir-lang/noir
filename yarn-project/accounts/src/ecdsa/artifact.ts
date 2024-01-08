import { ContractArtifact } from '@aztec/aztec.js';

import EcdsaAccountContractJson from '../artifacts/EcdsaAccount.json' assert { type: 'json' };

export const EcdsaAccountContractArtifact = EcdsaAccountContractJson as ContractArtifact;
