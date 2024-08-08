import { type NoirCompiledContract, loadContractArtifact } from '@aztec/aztec.js';

import EcdsaKAccountContractJson from '../../../artifacts/EcdsaKAccount.json' assert { type: 'json' };

export const EcdsaKAccountContractArtifact = loadContractArtifact(EcdsaKAccountContractJson as NoirCompiledContract);
