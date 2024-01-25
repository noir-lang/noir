import { Fr } from '@aztec/foundation/fields';
import { ContractClass } from '@aztec/types/contracts';

import { FunctionSelector, getContractClassId } from '../index.js';

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
            isInternal: false,
          },
        ],
        publicFunctions: [
          {
            selector: FunctionSelector.fromString('0x12345678'),
            bytecode: Buffer.from('123456789012345678901234567890', 'hex'),
            isInternal: false,
          },
        ],
      };

      expect(getContractClassId(contractClass).toString()).toMatchInlineSnapshot(
        `"0x1b436781f84669144ec383d6ea5f49b05ccba5c6221ebeb86085443c2a859202"`,
      );
    });
  });
});
