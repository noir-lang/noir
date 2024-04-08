import { getBenchmarkContractArtifact } from '../tests/fixtures.js';
import { computeArtifactHash } from './artifact_hash.js';

describe('ArtifactHash', () => {
  it('calculates the artifact hash', () => {
    const artifact = getBenchmarkContractArtifact();
    expect(computeArtifactHash(artifact).toString()).toMatchInlineSnapshot(
      `"0x0698cf658c2b1672eb0bbb6df8fc6760e8c34af4c7779f8206dab211b3b28814"`,
    );
  });
});
