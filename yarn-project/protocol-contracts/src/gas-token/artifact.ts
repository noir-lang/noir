import { loadContractArtifact } from '@aztec/types/abi';
import { NoirCompiledContract } from '@aztec/types/noir';

import GasTokenJson from '../artifacts/GasToken.json' assert { type: 'json' };

export const GasTokenArtifact = loadContractArtifact(GasTokenJson as NoirCompiledContract);
