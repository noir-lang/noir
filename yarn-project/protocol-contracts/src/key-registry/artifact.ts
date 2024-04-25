import { loadContractArtifact } from '@aztec/types/abi';
import { type NoirCompiledContract } from '@aztec/types/noir';

import KeyRegistryJson from '../artifacts/KeyRegistry.json' assert { type: 'json' };

export const KeyRegistryArtifact = loadContractArtifact(KeyRegistryJson as NoirCompiledContract);
