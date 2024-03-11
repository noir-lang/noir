import { Fr } from '@aztec/foundation/fields';
import { toFriendlyJSON } from '@aztec/foundation/serialize';
import { setupCustomSnapshotSerializers } from '@aztec/foundation/testing';

import { getSampleContractArtifact } from '../tests/fixtures.js';
import { getContractClassFromArtifact } from './contract_class.js';

describe('ContractClass', () => {
  setupCustomSnapshotSerializers(expect);
  it('creates a contract class from a contract compilation artifact', () => {
    const contractClass = getContractClassFromArtifact({
      ...getSampleContractArtifact(),
      artifactHash: Fr.fromString('0x1234'),
    });
    expect(toFriendlyJSON(contractClass)).toMatchSnapshot();
  });
});
