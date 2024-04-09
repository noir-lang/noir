import { getBenchmarkContractArtifact } from '../tests/fixtures.js';
import { computeArtifactHash } from './artifact_hash.js';

describe('ArtifactHash', () => {
  it('calculates the artifact hash', () => {
    const artifact = getBenchmarkContractArtifact();
    expect(computeArtifactHash(artifact).toString()).toMatchInlineSnapshot(
      `"0x2000991c126ffc45142660f7c3ad26f65aeb853a3b9c7e5906ba8205a2556128"`,
    );
  });
});
