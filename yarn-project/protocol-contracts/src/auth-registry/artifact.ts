import { loadContractArtifact } from '@aztec/types/abi';
import { type NoirCompiledContract } from '@aztec/types/noir';

import AuthRegistryJson from '../../artifacts/AuthRegistry.json' assert { type: 'json' };

export const AuthRegistryArtifact = loadContractArtifact(AuthRegistryJson as NoirCompiledContract);
