import { type NoirCompiledContract, loadContractArtifact } from '@aztec/aztec.js';

import EcdsaAccountContractJson from '../artifacts/EcdsaAccount.json' assert { type: 'json' };

export const EcdsaAccountContractArtifact = loadContractArtifact(EcdsaAccountContractJson as NoirCompiledContract);
