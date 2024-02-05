import { ContractArtifact } from '@aztec/foundation/abi';

import { getSampleContractArtifact } from '../tests/fixtures.js';
import { getContractClassFromArtifact } from './contract_class.js';
import { packBytecode, packedBytecodeAsFields, packedBytecodeFromFields, unpackBytecode } from './public_bytecode.js';

describe('PublicBytecode', () => {
  let artifact: ContractArtifact;
  beforeAll(() => {
    artifact = getSampleContractArtifact();
  });

  it('packs and unpacks public bytecode', () => {
    const { publicFunctions } = getContractClassFromArtifact(artifact);
    const packedBytecode = packBytecode(publicFunctions);
    const unpackedBytecode = unpackBytecode(packedBytecode);
    expect(unpackedBytecode).toEqual(publicFunctions);
  });

  it('converts small packed bytecode back and forth from fields', () => {
    const packedBytecode = Buffer.from('1234567890abcdef'.repeat(10), 'hex');
    const fields = packedBytecodeAsFields(packedBytecode);
    expect(packedBytecodeFromFields(fields).toString('hex')).toEqual(packedBytecode.toString('hex'));
  });

  it('converts real packed bytecode back and forth from fields', () => {
    const { packedBytecode } = getContractClassFromArtifact(artifact);
    const fields = packedBytecodeAsFields(packedBytecode);
    expect(packedBytecodeFromFields(fields).toString('hex')).toEqual(packedBytecode.toString('hex'));
  });
});
