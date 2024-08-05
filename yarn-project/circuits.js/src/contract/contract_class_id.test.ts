import { Fr } from '@aztec/foundation/fields';
import { type ContractClass } from '@aztec/types/contracts';

import { FunctionSelector, computeContractClassId } from '../index.js';

describe('ContractClass', () => {
  describe('getContractClassId', () => {
    it('calculates the contract class id', () => {
      const contractClass: ContractClass = {
        version: 1,
        artifactHash: Fr.fromString('0x1234'),
        packedBytecode: Buffer.from('123456789012345678901234567890', 'hex'),
        privateFunctions: [
          {
            selector: FunctionSelector.fromString('0x12345678'),
            vkHash: Fr.fromString('0x1234'),
          },
        ],
        publicFunctions: [
          {
            selector: FunctionSelector.fromString('0x12345678'),
            bytecode: Buffer.from('123456789012345678901234567890', 'hex'),
          },
        ],
      };

      expect(computeContractClassId(contractClass).toString()).toMatchInlineSnapshot(
        `"0x174bf0daff21f2b8b88f7d2b7ef7ed6a7b083c979a2996a4e78963ad4b84ff8d"`,
      );
    });
  });
});
