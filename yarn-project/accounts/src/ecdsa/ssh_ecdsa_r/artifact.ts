import { type NoirCompiledContract, loadContractArtifact } from '@aztec/aztec.js';

import EcdsaRAccountContractJson from '../../../artifacts/EcdsaRAccount.json' assert { type: 'json' };

export const EcdsaRAccountContractArtifact = loadContractArtifact(EcdsaRAccountContractJson as NoirCompiledContract);
