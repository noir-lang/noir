import { abiCoder } from './index.js';

const tests = [
  {
    params: [
      {
        name: 'myMethod',
        type: 'function',
        inputs: [
          {
            type: 'uint256',
            name: 'myNumber',
          },
          {
            type: 'string',
            name: 'myString',
          },
        ],
      },
    ],
    result: '0x24ee0097',
  },
  {
    params: [
      {
        name: 'myMethod',
        type: 'function',
        inputs: [
          {
            type: 'string',
            name: 'myNumber',
          },
          {
            type: 'bytes8',
            name: 'myString',
          },
        ],
      },
    ],
    result: '0x27b00c93',
  },
  {
    params: [
      {
        name: 'Somthing',
        type: 'function',
        inputs: [
          {
            type: 'uint16',
            name: 'myNumber',
          },
          {
            type: 'bytes',
            name: 'myString',
          },
        ],
      },
    ],
    result: '0x724ff7a1',
  },
  {
    params: [
      {
        name: 'something',
        type: 'function',
        inputs: [],
      },
    ],
    result: '0xa7a0d537',
  },
  {
    params: [
      {
        name: 'create',
        type: 'function',
        inputs: [
          {
            name: 'tokenId',
            type: 'uint256',
          },
          {
            name: 'itemOwner',
            type: 'address',
          },
          {
            name: 'keys',
            type: 'bytes32[]',
          },
          {
            name: 'values',
            type: 'bytes32[]',
          },
        ],
      },
    ],
    result: '0x04d36f08',
  },
];

describe('encodeFunctionSignature', () => {
  tests.forEach(test => {
    it('should convert correctly', () => {
      expect(abiCoder.encodeFunctionSignature(...(test.params as [any]))).toEqual(test.result);
    });
  });
});
