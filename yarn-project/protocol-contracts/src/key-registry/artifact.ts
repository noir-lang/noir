import { loadContractArtifact } from '@aztec/types/abi';
import { type NoirCompiledContract } from '@aztec/types/noir';

import NewKeyRegistryJson from '../../artifacts/NewKeyRegistry.json' assert { type: 'json' };

export const KeyRegistryArtifact = loadContractArtifact(NewKeyRegistryJson as NoirCompiledContract);
