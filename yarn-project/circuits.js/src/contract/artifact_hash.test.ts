import { getSampleContractArtifact } from '../tests/fixtures.js';
import { computeArtifactHash } from './artifact_hash.js';

describe('ArtifactHash', () => {
  it('calculates the artifact hash', () => {
    const artifact = getSampleContractArtifact();
    expect(computeArtifactHash(artifact).toString()).toMatchInlineSnapshot(
      `"0x10d144027c5d0dddb7336f9becb14db882c0f4e48cfab674f1871458f81838ca"`,
    );
  });
});
