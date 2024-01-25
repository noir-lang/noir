import { getSampleContractArtifact } from '../tests/fixtures.js';
import { getArtifactHash } from './artifact_hash.js';

describe('ArtifactHash', () => {
  it('calculates the artifact hash', () => {
    const artifact = getSampleContractArtifact();
    expect(getArtifactHash(artifact).toString()).toMatchInlineSnapshot(
      `"0x1cd31b12181cf7516720f4675ffea13c8c538dc4875232776adb8bbe8364ed5c"`,
    );
  });
});
