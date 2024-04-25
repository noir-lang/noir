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
        `"0x0fd34f4f2d6d6a7fc61d8fb8e0c9a411354856fa86c568e4c9e0935b367dc69d"`,
      );
    });
  });
});
