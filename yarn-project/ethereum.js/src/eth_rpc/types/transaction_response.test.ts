import { EthAddress } from '@aztec/foundation/eth-address';
import { hexToBuffer } from '../../hex_string/index.js';
import { fromRawTransactionResponse } from './transaction_response.js';

describe('formatters', () => {
  describe('transactionResponseFormatter', () => {
    it('should return the correct value', () => {
      expect(
        fromRawTransactionResponse({
          input: '0x3454645634534',
          from: '0x11f4d0A3c12e86B4b5F39B213F7E19D048276DAe',
          to: '0x11f4d0a3c12e86b4b5f39b213f7e19d048276dae',
          value: '0x3e8',
          gas: '0x3e8',
          gasPrice: '0x3e8',
          nonce: '0xb',
          transactionIndex: '0x1',
          blockNumber: '0x3e8',
          blockHash: '0xc9b9cdc2092a9d6589d96662b1fd6949611163fb3910cf8a173cd060f17702f9',
          hash: '0xd9b9cdc2092a9d6589d96662b1fd6949611163fb3910cf8a173cd060f17702f9',
          type: '0x0',
          v: 'v',
          r: 'r',
          s: 's',
        }),
      ).toEqual({
        input: hexToBuffer('0x3454645634534'),
        from: EthAddress.fromString('0x11f4d0A3c12e86B4b5F39B213F7E19D048276DAe'),
        to: EthAddress.fromString('0x11f4d0A3c12e86B4b5F39B213F7E19D048276DAe'),
        value: 1000n,
        gas: 1000,
        gasPrice: 1000n,
        nonce: 11,
        blockNumber: 1000,
        blockHash: '0xc9b9cdc2092a9d6589d96662b1fd6949611163fb3910cf8a173cd060f17702f9',
        transactionIndex: 1,
        hash: '0xd9b9cdc2092a9d6589d96662b1fd6949611163fb3910cf8a173cd060f17702f9',
        type: 0,
        v: 'v',
        r: 'r',
        s: 's',
      });
    });

    it('should return the correct value, when null values are present', () => {
      expect(
        fromRawTransactionResponse({
          input: '0x3454645634534',
          from: '0x11f4d0a3c12e86b4b5f39b213f7e19d048276dae',
          to: null,
          value: '0x3e8',
          gas: '0x3e8',
          gasPrice: '0x3e8',
          nonce: '0xb',
          transactionIndex: null,
          blockNumber: null,
          blockHash: null,
          hash: '0xd9b9cdc2092a9d6589d96662b1fd6949611163fb3910cf8a173cd060f17702f9',
          type: '0x0',
          v: 'v',
          r: 'r',
          s: 's',
        }),
      ).toEqual({
        input: hexToBuffer('0x3454645634534'),
        from: EthAddress.fromString('0x11f4d0A3c12e86B4b5F39B213F7E19D048276DAe'),
        to: null,
        value: 1000n,
        gas: 1000,
        gasPrice: 1000n,
        nonce: 11,
        blockNumber: null,
        blockHash: null,
        transactionIndex: null,
        hash: '0xd9b9cdc2092a9d6589d96662b1fd6949611163fb3910cf8a173cd060f17702f9',
        type: 0,
        v: 'v',
        r: 'r',
        s: 's',
      });
    });
  });
});
