import { hexToBuffer } from '../hex_string/index.js';
import { ContractAbi, ContractAbiDefinition } from './abi/index.js';
import { decodeErrorFromContract } from './decode_error.js';

describe('decode_error', () => {
  it('should decode error', () => {
    const abi: ContractAbiDefinition = [
      {
        inputs: [
          {
            internalType: 'bytes32',
            name: 'oldStateHash',
            type: 'bytes32',
          },
          {
            internalType: 'bytes32',
            name: 'newStateHash',
            type: 'bytes32',
          },
        ],
        name: 'INCORRECT_STATE_HASH',
        type: 'error',
      },
    ];
    const data = hexToBuffer(
      '0x34fddf40160e1512008ebe521f7650fddad39c8a4f092fc451263be0190d631da26d345f88b748f29261d3cc053519106d94b965cd94d9143d58104f9becb80814d6917c',
    );

    const error = decodeErrorFromContract(new ContractAbi(abi), data);

    expect(error).not.toBeUndefined();
    expect(error).toEqual({
      name: 'INCORRECT_STATE_HASH',
      params: [
        '0x160e1512008ebe521f7650fddad39c8a4f092fc451263be0190d631da26d345f',
        '0x88b748f29261d3cc053519106d94b965cd94d9143d58104f9becb80814d6917c',
      ],
      message:
        'INCORRECT_STATE_HASH(0x160e1512008ebe521f7650fddad39c8a4f092fc451263be0190d631da26d345f,0x88b748f29261d3cc053519106d94b965cd94d9143d58104f9becb80814d6917c)',
    });
  });
});
