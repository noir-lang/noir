import { getSampleContractArtifact } from '../tests/fixtures.js';
import { computeArtifactHash } from './artifact_hash.js';

describe('ArtifactHash', () => {
  it('calculates the artifact hash', () => {
    const artifact = getSampleContractArtifact();
    expect(computeArtifactHash(artifact).toString()).toMatchInlineSnapshot(
      `"0x1cf6d98fcb8e56b65f077265ebc3f10ec7ce9fe85c8603a5a0ce09434d94dd53"`,
    );
  });
});
