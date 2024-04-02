import { loadContractArtifact } from '@aztec/types/abi';
import { type NoirCompiledContract } from '@aztec/types/noir';

import MultiCallEntrypoint from '../artifacts/MultiCallEntrypoint.json' assert { type: 'json' };

export const MultiCallEntrypointArtifact = loadContractArtifact(MultiCallEntrypoint as NoirCompiledContract);
