import { Fr } from '@aztec/foundation/fields';
import { toFriendlyJSON } from '@aztec/foundation/serialize';

import { getSampleContractArtifact } from '../tests/fixtures.js';
import { createContractClassFromArtifact } from './contract_class.js';

describe('ContractClass', () => {
  it('creates a contract class from a contract compilation artifact', () => {
    const contractClass = createContractClassFromArtifact({
      ...getSampleContractArtifact(),
      artifactHash: Fr.fromString('0x1234'),
    });
    expect(toFriendlyJSON(contractClass)).toMatchSnapshot();
  });
});
