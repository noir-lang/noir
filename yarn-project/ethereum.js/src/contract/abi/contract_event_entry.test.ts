import { keccak256String } from '../../crypto/index.js';
import { EthAddress } from '@aztec/foundation/eth-address';
import { TxHash } from '../../eth_rpc/index.js';
import { ContractEventEntry } from './contract_event_entry.js';

describe('contract', () => {
  describe('contract event entry', () => {
    it('should return the decoded event object with topics', () => {
      const address = EthAddress.fromString('0x11f4d0A3c12e86B4b5F39B213F7E19D048276DAe');
      const signature = 'Changed(address,uint256,uint256,uint256)';

      const contractEventEntry = new ContractEventEntry({
        name: 'Changed',
        type: 'event',
        inputs: [
          { name: 'from', type: 'address', indexed: true },
          { name: 'amount', type: 'uint256', indexed: true },
          { name: 't1', type: 'uint256', indexed: false },
          { name: 't2', type: 'uint256', indexed: false },
        ],
      });

      const result = contractEventEntry.decodeEvent({
        id: '',
        address,
        topics: [
          keccak256String(signature),
          '0x000000000000000000000000' + address.toString().replace('0x', ''),
          '0x0000000000000000000000000000000000000000000000000000000000000001',
        ],
        blockNumber: 3,
        transactionHash: TxHash.fromString('0x1234555555555555555555555555555555555555555555555555555555555555'),
        blockHash: '0x1345',
        transactionIndex: 0,
        logIndex: 4,
        data:
          '0x0000000000000000000000000000000000000000000000000000000000000001' +
          '0000000000000000000000000000000000000000000000000000000000000008',
      });

      expect(result.args.from).toEqual(address);
      expect(result.args.amount).toBe(1n);
      expect(result.args.t1).toBe(1n);
      expect(result.args.t2).toBe(8n);
    });

    const name = 'event1';
    const address = '0xffdDb67890123456789012345678901234567890';

    const tests: any = [
      {
        abi: {
          name,
          type: 'event',
          inputs: [],
        },
        data: {
          logIndex: 1,
          transactionIndex: 16,
          transactionHash: '0x1234567890',
          address,
          blockHash: '0x1234567890',
          blockNumber: 1,
          id: 'log_c71f2e84',
        },
        expected: {
          event: name,
          signature: null,
          args: {},
          logIndex: 1,
          transactionIndex: 16,
          transactionHash: '0x1234567890',
          address,
          blockHash: '0x1234567890',
          blockNumber: 1,
          id: 'log_c71f2e84',
          raw: {
            topics: [],
            data: '',
          },
        },
      },
      {
        abi: {
          name,
          inputs: [
            {
              name: 'a',
              type: 'int',
              indexed: false,
            },
          ],
        },
        data: {
          logIndex: 1,
          transactionIndex: 16,
          transactionHash: '0x1234567890',
          address,
          blockHash: '0x1234567890',
          blockNumber: 1,
          id: 'log_c71f2e84',
          data: '0x0000000000000000000000000000000000000000000000000000000000000001',
        },
        expected: {
          event: name,
          signature: null,
          args: {
            0: 1n,
            a: 1n,
          },
          logIndex: 1,
          transactionIndex: 16,
          transactionHash: '0x1234567890',
          address,
          blockHash: '0x1234567890',
          blockNumber: 1,
          id: 'log_c71f2e84',
          raw: {
            data: '0x0000000000000000000000000000000000000000000000000000000000000001',
            topics: [],
          },
        },
      },
      {
        abi: {
          name,
          inputs: [
            {
              name: 'a',
              type: 'int',
              indexed: false,
            },
            {
              name: 'b',
              type: 'int',
              indexed: true,
            },
            {
              name: 'c',
              type: 'int',
              indexed: false,
            },
            {
              name: 'd',
              type: 'int',
              indexed: true,
            },
          ],
        },
        data: {
          logIndex: 1,
          transactionIndex: 16,
          transactionHash: '0x1234567890',
          address,
          blockHash: '0x1234567890',
          blockNumber: 1,
          id: 'log_c71f2e84',
          data:
            '0x' +
            '0000000000000000000000000000000000000000000000000000000000000001' +
            '0000000000000000000000000000000000000000000000000000000000000004',
          topics: [
            address,
            '0x000000000000000000000000000000000000000000000000000000000000000a',
            '0x0000000000000000000000000000000000000000000000000000000000000010',
          ],
        },
        expected: {
          event: name,
          signature: address,
          args: {
            0: 1n,
            1: 10n,
            2: 4n,
            3: 16n,
            a: 1n,
            b: 10n,
            c: 4n,
            d: 16n,
          },
          logIndex: 1,
          transactionIndex: 16,
          transactionHash: '0x1234567890',
          address,
          blockHash: '0x1234567890',
          blockNumber: 1,
          id: 'log_c71f2e84',
          raw: {
            data:
              '0x' +
              '0000000000000000000000000000000000000000000000000000000000000001' +
              '0000000000000000000000000000000000000000000000000000000000000004',
            topics: [
              address,
              '0x000000000000000000000000000000000000000000000000000000000000000a',
              '0x0000000000000000000000000000000000000000000000000000000000000010',
            ],
          },
        },
      },
      {
        abi: {
          name,
          anonymous: true,
          inputs: [
            {
              name: 'a',
              type: 'int',
              indexed: false,
            },
            {
              name: 'b',
              type: 'int',
              indexed: true,
            },
            {
              name: 'c',
              type: 'int',
              indexed: false,
            },
            {
              name: 'd',
              type: 'int',
              indexed: true,
            },
          ],
        },
        data: {
          logIndex: 1,
          transactionIndex: 16,
          transactionHash: '0x1234567890',
          address,
          blockHash: '0x1234567890',
          blockNumber: 1,
          id: 'log_c71f2e84',
          data:
            '0x' +
            '0000000000000000000000000000000000000000000000000000000000000001' +
            '0000000000000000000000000000000000000000000000000000000000000004',
          topics: [
            '0x000000000000000000000000000000000000000000000000000000000000000a',
            '0x0000000000000000000000000000000000000000000000000000000000000010',
          ],
        },
        expected: {
          event: name,
          signature: null,
          args: {
            0: 1n,
            1: 10n,
            2: 4n,
            3: 16n,
            a: 1n,
            b: 10n,
            c: 4n,
            d: 16n,
          },
          logIndex: 1,
          transactionIndex: 16,
          transactionHash: '0x1234567890',
          address,
          blockHash: '0x1234567890',
          blockNumber: 1,
          id: 'log_c71f2e84',
          raw: {
            data:
              '0x' +
              '0000000000000000000000000000000000000000000000000000000000000001' +
              '0000000000000000000000000000000000000000000000000000000000000004',
            topics: [
              '0x000000000000000000000000000000000000000000000000000000000000000a',
              '0x0000000000000000000000000000000000000000000000000000000000000010',
            ],
          },
        },
      },
    ];

    tests.forEach((test, index) => {
      it('test no: ' + index, () => {
        const contractEventEntry = new ContractEventEntry(test.abi);
        const result = contractEventEntry.decodeEvent(test.data);
        expect(result).toEqual(test.expected);
      });
    });
  });
});
