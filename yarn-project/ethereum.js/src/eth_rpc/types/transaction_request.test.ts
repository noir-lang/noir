import { EthAddress } from '../../eth_address/index.js';
import { toRawTransactionRequest } from './transaction_request.js';

const tests = [
  {
    input: {
      data: Buffer.from('34234bf23bf423', 'hex'),
      value: 100n,
      from: EthAddress.fromString('0x11f4d0A3c12e86B4b5F39B213F7E19D048276DAe'),
      to: EthAddress.fromString('0x00c5496aee77c1ba1f0854206a26dda82a81d6d8'),
      nonce: 1000,
      gas: 1000,
      maxFeePerGas: 1000n,
      maxPriorityFeePerGas: 1001n,
    },
    result: {
      data: '0x34234bf23bf423',
      value: '0x64',
      from: '0x11f4d0a3c12e86b4b5f39b213f7e19d048276dae',
      to: '0x00c5496aee77c1ba1f0854206a26dda82a81d6d8',
      nonce: '0x3e8',
      gas: '0x3e8',
      maxFeePerGas: '0x3e8',
      maxPriorityFeePerGas: '0x3e9',
    },
  },
  {
    input: {
      data: Buffer.from('34234bf23bf423', 'hex'),
      value: 100n,
      from: EthAddress.fromString('00c5496aee77c1ba1f0854206a26dda82a81d6d8'),
      to: EthAddress.fromString('0x11f4d0A3c12e86B4b5F39B213F7E19D048276DAe'),
    },
    result: {
      data: '0x34234bf23bf423',
      value: '0x64',
      from: '0x00c5496aee77c1ba1f0854206a26dda82a81d6d8',
      to: '0x11f4d0a3c12e86b4b5f39b213f7e19d048276dae',
    },
  },
  {
    input: {
      data: Buffer.from('34234bf23bf423', 'hex'),
      value: 100n,
      from: EthAddress.fromString('00c5496aee77c1ba1f0854206a26dda82a81d6d8'),
      to: EthAddress.fromString('0x11f4d0A3c12e86B4b5F39B213F7E19D048276DAe'),
      gas: 1000,
    },
    result: {
      data: '0x34234bf23bf423',
      value: '0x64',
      from: '0x00c5496aee77c1ba1f0854206a26dda82a81d6d8',
      to: '0x11f4d0a3c12e86b4b5f39b213f7e19d048276dae',
      gas: '0x3e8',
    },
  },
];

describe('formatters', () => {
  describe('toRawTransactionRequest', () => {
    tests.forEach(test => {
      it('should return the correct value', () => {
        expect(toRawTransactionRequest(test.input)).toEqual(test.result);
      });
    });
  });
});
