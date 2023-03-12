import { ContractAbi } from './contract_abi.js';

describe('contract', () => {
  describe('contract-abi', () => {
    it('should correctly decode tx data', () => {
      const testContractAbi = new ContractAbi([
        {
          name: 'myOtherMethod',
          type: 'function',
          inputs: [
            {
              type: 'uint16',
              name: 'myNumberdd',
            },
            {
              type: 'bytes32',
              name: 'myBytes',
            },
          ],
        },
        {
          name: 'hasALotOfParams',
          inputs: [
            {
              name: 'var1',
              type: 'bytes32',
            },
            {
              name: 'var2',
              type: 'string',
            },
            {
              name: 'var3',
              type: 'bytes32[]',
            },
          ],
          outputs: [
            {
              name: 'owner',
              type: 'address',
            },
          ],
          constant: false,
          payable: false,
          type: 'function',
        },
      ]);

      const input = [
        '0x1111111111111111111111111111111111111111111111111111111111111111',
        'Hello World',
        [
          '0x2222222222222222222222222222222222222222222222222222222222222222',
          '0x3333333333333333333333333333333333333333333333333333333333333333',
        ],
      ];

      const encoded = testContractAbi.functions[1].encodeABI(input);
      const result = testContractAbi.decodeFunctionData(encoded);

      expect(result).not.toBeUndefined();
      expect(result![0]).toBe(input[0]);
      expect(result![1]).toBe(input[1]);
      expect(result![2]).toEqual(input[2]);
    });
  });
});
