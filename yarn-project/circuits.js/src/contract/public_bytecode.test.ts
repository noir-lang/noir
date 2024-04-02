import { type ContractArtifact } from '@aztec/foundation/abi';

import { getBenchmarkContractArtifact } from '../tests/fixtures.js';
import { getContractClassFromArtifact } from './contract_class.js';
import { packBytecode, unpackBytecode } from './public_bytecode.js';

describe('PublicBytecode', () => {
  let artifact: ContractArtifact;
  beforeAll(() => {
    artifact = getBenchmarkContractArtifact();
  });

  it('packs and unpacks public bytecode', () => {
    const { publicFunctions } = getContractClassFromArtifact(artifact);
    const packedBytecode = packBytecode(publicFunctions);
    const unpackedBytecode = unpackBytecode(packedBytecode);
    expect(unpackedBytecode).toEqual(publicFunctions);
  });
});
