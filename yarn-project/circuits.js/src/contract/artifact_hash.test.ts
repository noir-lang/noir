import { getBenchmarkContractArtifact } from '../tests/fixtures.js';
import { computeArtifactHash } from './artifact_hash.js';

describe('ArtifactHash', () => {
  it('calculates the artifact hash', () => {
    const artifact = getBenchmarkContractArtifact();
    expect(computeArtifactHash(artifact).toString()).toMatchInlineSnapshot(
      `"0x077417e7303df9113d0d0947b9dc8a06903cd8aa963a31891e2f28ce121d753e"`,
    );
  });
});
