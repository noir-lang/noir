import { getSampleContractArtifact } from '../test/fixtures.js';
import { contractArtifactFromBuffer, contractArtifactToBuffer } from './contract_artifact.js';

describe('contract_artifact', () => {
  it('serializes and deserializes an instance', () => {
    const artifact = getSampleContractArtifact();
    const serialized = contractArtifactToBuffer(artifact);
    const deserialized = contractArtifactFromBuffer(serialized);
    expect(deserialized).toEqual(artifact);
  });
});
