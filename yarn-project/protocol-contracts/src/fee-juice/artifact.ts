import { loadContractArtifact } from '@aztec/types/abi';
import { type NoirCompiledContract } from '@aztec/types/noir';

import FeeJuiceJson from '../../artifacts/FeeJuice.json' assert { type: 'json' };

export const FeeJuiceArtifact = loadContractArtifact(FeeJuiceJson as NoirCompiledContract);
