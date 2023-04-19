import { EthAddress } from '@aztec/foundation';
import { EthereumRpc } from '../eth_rpc/ethereum_rpc.js';
import { mock } from 'jest-mock-extended';
import { EthereumProvider } from '../provider/ethereum_provider.js';
import { sha3 } from '../crypto/index.js';
import { bufferToHex, hexToBuffer } from '../hex_string/index.js';
import { Contract } from './contract.js';
import { TestContract } from './fixtures/TestContract.js';
import TestContractAbi from './fixtures/TestContractAbi.js';
import { TestNoCtorContract } from './fixtures/TestNoCtorContract.js';
import { RawTransactionReceipt, TxHash } from '../eth_rpc/index.js';
import { ContractAbi } from './abi/index.js';

describe('contract', () => {
  const address = EthAddress.fromString('0x11f4d0A3c12e86B4b5F39B213F7E19D048276DAe');
  const addressLowercase = address.toString().toLowerCase();
  const addressUnprefixedLowercase = addressLowercase.slice(2);
  const address2 = EthAddress.fromString('0x5555567890123456789012345678901234567891');
  const address2Lowercase = address2.toString().toLowerCase();
  let eth: EthereumRpc;
  let mockEthereumProvider: ReturnType<typeof mock<EthereumProvider>>;

  beforeEach(() => {
    mockEthereumProvider = mock<EthereumProvider>();
    eth = new EthereumRpc(mockEthereumProvider);
  });

  describe('instantiation', () => {
    it('should construct without address', () => {
      const contract = new TestContract(eth);
      expect(contract.address).toEqual(EthAddress.ZERO);
    });

    it('should transform address from checksum addresses', () => {
      const contract = new TestContract(eth, address);
      expect(contract.address).toBe(address);
    });

    it('should transform address to checksum address', () => {
      const contract = new TestContract(eth, address);
      expect(contract.address).toBe(address);
    });
  });

  // describe('event', () => {
  //   const signature = 'Changed(address,uint256,uint256,uint256)';

  //   function emitData(delayMs: number = 0, extend?: object) {
  //     setTimeout(() => {
  //       mockEthereumProvider.emit('notification', {
  //         subscription: '0x123',
  //         result: {
  //           address: addressLowercase,
  //           topics: [
  //             sha3(signature),
  //             '0x000000000000000000000000' + addressLowercase.replace('0x', ''),
  //             '0x0000000000000000000000000000000000000000000000000000000000000001',
  //           ],
  //           blockNumber: '0x3',
  //           transactionIndex: '0x0',
  //           transactionHash: '0x1234',
  //           blockHash: '0x1345',
  //           logIndex: '0x4',
  //           data:
  //             '0x0000000000000000000000000000000000000000000000000000000000000001' +
  //             '0000000000000000000000000000000000000000000000000000000000000008',
  //           ...extend,
  //         },
  //       });
  //     }, delayMs);
  //   }

  //   function mockEthSubscribe() {
  //     mockEthereumProvider.send.mockImplementationOnce(async (method, params) => {
  //       expect(method).toBe('eth_subscribe');
  //       expect(params[1]).toEqual({
  //         topics: [sha3(signature), '0x000000000000000000000000' + addressUnprefixedLowercase, null],
  //         address: addressLowercase,
  //       });

  //       emitData(10);

  //       return '0x123';
  //     });
  //   }

  //   it('should create event subscription', done => {
  //     mockEthSubscribe();

  //     const contract = new TestContract(eth, address);

  //     const event = contract.events.Changed({ filter: { from: address } }, (err, result) => {
  //       if (err) {
  //         return done(err);
  //       }
  //       expect(result.args.from).toEqual(address);
  //       expect(result.args.amount).toBe('1');
  //       expect(result.args.t1).toBe('1');
  //       expect(result.args.t2).toBe('8');

  //       event.unsubscribe();
  //       done();
  //     });
  //   });

  //   it('should create event from the events object using a signature and callback', done => {
  //     mockEthSubscribe();

  //     const contract = new TestContract(eth, address);

  //     const event = contract.events['0x792991ed5ba9322deaef76cff5051ce4bedaaa4d097585970f9ad8f09f54e651'](
  //       { filter: { from: address } },
  //       (err, result) => {
  //         expect(result.args.from).toEqual(address);
  //         expect(result.args.amount).toBe('1');
  //         expect(result.args.t1).toBe('1');
  //         expect(result.args.t2).toBe('8');

  //         event.unsubscribe();
  //         done();
  //       },
  //     );
  //   });

  //   it('should create event from the events object using event name and parameters', done => {
  //     mockEthSubscribe();

  //     const contract = new TestContract(eth, address);

  //     const event = contract.events[signature]({ filter: { from: address } }, (err, result) => {
  //       expect(result.args.from).toEqual(address);
  //       expect(result.args.amount).toBe('1');
  //       expect(result.args.t1).toBe('1');
  //       expect(result.args.t2).toBe('8');

  //       event.unsubscribe();
  //       done();
  //     });
  //   });

  //   it('should create event from the events object and use the fromBlock option', done => {
  //     mockEthereumProvider.send.mockImplementationOnce(method => {
  //       expect(method).toBe('eth_getLogs');
  //       return [
  //         {
  //           address: addressLowercase,
  //           topics: [
  //             sha3(signature),
  //             '0x000000000000000000000000' + addressLowercase.replace('0x', ''),
  //             '0x0000000000000000000000000000000000000000000000000000000000000002',
  //           ],
  //           blockNumber: '0x3',
  //           transactionHash: '0x1234',
  //           transactionIndex: '0x0',
  //           blockHash: '0x1345',
  //           logIndex: '0x4',
  //           data:
  //             '0x0000000000000000000000000000000000000000000000000000000000000002' +
  //             '0000000000000000000000000000000000000000000000000000000000000009',
  //         },
  //         {
  //           address: addressLowercase,
  //           topics: [
  //             sha3(signature),
  //             '0x000000000000000000000000' + addressLowercase.replace('0x', ''),
  //             '0x0000000000000000000000000000000000000000000000000000000000000003',
  //           ],
  //           blockNumber: '0x4',
  //           transactionHash: '0x1235',
  //           transactionIndex: '0x1',
  //           blockHash: '0x1346',
  //           logIndex: '0x1',
  //           data:
  //             '0x0000000000000000000000000000000000000000000000000000000000000004' +
  //             '0000000000000000000000000000000000000000000000000000000000000005',
  //         },
  //       ] as RawLogResponse[];
  //     });

  //     mockEthSubscribe();

  //     const contract = new TestContract(eth, address);
  //     let count = 0;

  //     const event = contract.events.Changed({ fromBlock: 0, filter: { from: address } }).on('data', result => {
  //       count++;

  //       if (count === 1) {
  //         expect(result.args.from).toEqual(address);
  //         expect(result.args.amount).toBe('2');
  //         expect(result.args.t1).toBe('2');
  //         expect(result.args.t2).toBe('9');
  //       }
  //       if (count === 2) {
  //         expect(result.args.from).toEqual(address);
  //         expect(result.args.amount).toBe('3');
  //         expect(result.args.t1).toBe('4');
  //         expect(result.args.t2).toBe('5');
  //       }
  //       if (count === 3) {
  //         expect(result.args.from).toEqual(address);
  //         expect(result.args.amount).toBe('1');
  //         expect(result.args.t1).toBe('1');
  //         expect(result.args.t2).toBe('8');

  //         event.unsubscribe();
  //         done();
  //       }
  //     });
  //   });

  //   it('should create event using the function and unsubscribe after one log received', async () => {
  //     mockEthSubscribe();

  //     let count = 0;

  //     const contract = new TestContract(eth, address);

  //     await new Promise(resolve => {
  //       contract.once('Changed', { filter: { from: address } }, (err, result, sub) => {
  //         count++;
  //         resolve();
  //       });
  //     });

  //     // Emit a second.
  //     mockEthereumProvider.emit('notification', {
  //       subscription: '0x123',
  //       result: {
  //         blockHash: '0x1345',
  //       },
  //     });

  //     expect(count).toBe(1);
  //   });

  //   it('should create event subscription and fire the changed event, if log.removed = true', done => {
  //     mockEthSubscribe();
  //     emitData(200, { removed: true });

  //     let count = 1;
  //     const contract = new TestContract(eth, address);

  //     contract.events
  //       .Changed({ filter: { from: address } })
  //       .on('data', result => {
  //         expect(count).toBe(1);
  //         count++;
  //       })
  //       .on('changed', result => {
  //         expect(result.removed).toBe(true);
  //         expect(count).toBe(2);
  //         done();
  //       });
  //   });

  //   it('should create all event filter and receive two logs', done => {
  //     mockEthereumProvider.send.mockImplementationOnce(async (method, params) => {
  //       expect(method).toBe('eth_subscribe');
  //       expect(params[1]).toEqual({
  //         topics: [],
  //         address: addressLowercase,
  //       });

  //       emitData(100);
  //       emitData(200, {
  //         topics: [
  //           sha3('Unchanged(uint256,address,uint256)'),
  //           '0x0000000000000000000000000000000000000000000000000000000000000002',
  //           '0x000000000000000000000000' + address.toString().replace('0x', ''),
  //         ],
  //         data: '0x0000000000000000000000000000000000000000000000000000000000000005',
  //       });

  //       return '0x123';
  //     });

  //     const contract = new TestContract(eth, address);

  //     let count = 0;
  //     const event = contract.events.allEvents({}, (_, result) => {
  //       count++;

  //       if (count === 1 && result.event === 'Changed') {
  //         expect(result.args.from).toEqual(address);
  //         expect(result.args.amount).toBe('1');
  //         expect(result.args.t1).toBe('1');
  //         expect(result.args.t2).toBe('8');
  //       }
  //       if (count === 2 && result.event === 'Unchanged') {
  //         expect(result.args.addressFrom).toEqual(address);
  //         expect(result.args.value).toBe('2');
  //         expect(result.args.t1).toBe('5');

  //         event.unsubscribe();
  //         done();
  //       }
  //     });
  //   });
  // });

  describe('balance call', () => {
    const signature = 'balance(address)';

    it('should encode a function call', () => {
      const contract = new TestContract(eth, address);

      const result = contract.methods.balance(address).encodeABI();

      expect(bufferToHex(result)).toBe(
        sha3(signature).slice(0, 10) + '000000000000000000000000' + addressUnprefixedLowercase,
      );
    });

    it('should encode a constructor call with data', () => {
      const contract = new TestContract(eth, address);

      const result = contract.deployBytecode('0x1234', address, 10).encodeABI();

      expect(bufferToHex(result)).toBe(
        '0x1234' +
          '000000000000000000000000' +
          addressLowercase.replace('0x', '') +
          '000000000000000000000000000000000000000000000000000000000000000a',
      );
    });

    it('should estimate a function', async () => {
      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_estimateGas');
        expect(params).toEqual([
          {
            data: sha3(signature).slice(0, 10) + '000000000000000000000000' + addressLowercase.replace('0x', ''),
            to: addressLowercase,
          },
        ]);
        return Promise.resolve('0x0000000000000000000000000000000000000000000000000000000000000032');
      });

      const contract = new TestContract(eth, address);

      const res = await contract.methods.balance(address).estimateGas();
      expect(res).toBe(50);
    });

    it('should estimate the constructor', async () => {
      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_estimateGas');
        expect(params).toEqual([
          {
            data:
              '0x1234000000000000000000000000' +
              addressLowercase.replace('0x', '') +
              '0000000000000000000000000000000000000000000000000000000000000032',
          },
        ]);
        return Promise.resolve('0x000000000000000000000000000000000000000000000000000000000000000a');
      });

      const contract = new TestContract(eth, address);

      const res = await contract.deployBytecode('0x1234', address, 50).estimateGas();
      expect(res).toBe(10);
    });

    it('should send with many parameters', async () => {
      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_call');
        expect(params).toEqual([
          {
            data: '0x817a9dc00000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000000a68656c6c6f776f726c64000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002ff245453450000000000000000000000000000000000000000000000000000005345000000000000000000000000000000000000000000000000000000000000',
            to: addressLowercase,
          },
          'latest',
        ]);
        return Promise.resolve('0x000000000000000000000000' + addressLowercase.replace('0x', ''));
      });

      const contract = new TestContract(eth, address);

      const res = await contract.methods
        .hasALotOfParams(1, 'helloworld', [hexToBuffer('0xff24545345'), hexToBuffer('0x5345')])
        .call();
      expect(res).toEqual(address);
    });

    it('should send overload functions with zero parameters', async () => {
      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_call');
        expect(params).toEqual([
          {
            data: '0xbb853481',
            to: addressLowercase,
          },
          'latest',
        ]);
        return Promise.resolve('0x0000000000000000000000000000000000000000000000000000000000000005');
      });

      const contract = new TestContract(eth, address);
      const res = await contract.methods.overloadedFunction().call();
      expect(res).toBe(5n);
    });

    it('should send overload functions with one parameters', async () => {
      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_call');
        expect(params).toEqual([
          {
            data: '0x533678270000000000000000000000000000000000000000000000000000000000000006',
            to: addressLowercase,
          },
          'latest',
        ]);
        return Promise.resolve('0x0000000000000000000000000000000000000000000000000000000000000006');
      });

      const contract = new TestContract(eth, address);

      const res = await contract.methods.overloadedFunction(6n).call();
      expect(res).toBe(6n);
    });

    it('should call constant function', async () => {
      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_call');
        expect(params).toEqual([
          {
            data: sha3(signature).slice(0, 10) + '000000000000000000000000' + addressLowercase.replace('0x', ''),
            to: addressLowercase,
          },
          'latest',
        ]);
        return Promise.resolve('0x0000000000000000000000000000000000000000000000000000000000000032');
      });

      const contract = new TestContract(eth, address);

      const res = await contract.methods.balance(address).call();
      expect(res).toBe(50n);
    });

    it('should call constant function with default block', async () => {
      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_call');
        expect(params).toEqual([
          {
            data: sha3(signature).slice(0, 10) + '000000000000000000000000' + addressLowercase.replace('0x', ''),
            to: addressLowercase,
          },
          '0xb',
        ]);
        return Promise.resolve('0x0000000000000000000000000000000000000000000000000000000000000032');
      });

      const contract = new TestContract(eth, address);

      const res = await contract.methods.balance(address).call({}, 11);
      expect(res).toBe(50n);
    });

    it('should call constant concurrently', async () => {
      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_call');
        expect(params).toEqual([
          {
            data:
              sha3('balance(address)').slice(0, 10) + '000000000000000000000000' + addressLowercase.replace('0x', ''),
            to: addressLowercase,
          },
          'latest',
        ]);
        return Promise.resolve('0x000000000000000000000000000000000000000000000000000000000000000a');
      });

      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_call');
        expect(params).toEqual([
          {
            data: sha3('owner()').slice(0, 10),
            to: addressLowercase,
          },
          'latest',
        ]);
        return Promise.resolve('0x00000000000000000000000011f4d0a3c12e86b4b5f39b213f7e19d048276dae');
      });

      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_call');
        expect(params).toEqual([
          {
            data: sha3('getStr()').slice(0, 10),
            to: addressLowercase,
          },
          'latest',
        ]);
        return Promise.resolve(
          '0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000848656c6c6f212521000000000000000000000000000000000000000000000000',
        );
      });

      const contract = new TestContract(eth, address);

      const [m1, m2, m3] = await Promise.all([
        contract.methods.balance(address).call(),
        contract.methods.owner().call(),
        contract.methods.getStr().call(),
      ]);

      expect(m1).toBe(10n);
      expect(m2).toEqual(address);
      expect(m3).toBe('Hello!%!');
    });

    it('should return an error when returned string is 0x', async () => {
      const signature = 'getStr()';

      const contract = new TestContract(eth, address);

      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_call');
        expect(params).toEqual([
          {
            data: sha3(signature).slice(0, 10),
            to: addressLowercase,
            from: address2,
          },
          'latest',
        ]);
        return Promise.resolve('0x');
      });

      await expect(contract.methods.getStr().call({ from: address2 })).rejects.toBeInstanceOf(Error);
    });

    it('should return an empty string when 0x0', async () => {
      const signature = 'getStr()';

      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_call');
        expect(params).toEqual([
          {
            data: sha3(signature).slice(0, 10),
            to: addressLowercase,
            from: address2.toString().toLowerCase(),
          },
          'latest',
        ]);
        return Promise.resolve(
          '0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000',
        );
      });

      const contract = new TestContract(eth, address);

      const result = await contract.methods.getStr().call({ from: address2 });
      expect(result).toBe('');
    });
  });

  describe('send', () => {
    const signature = sha3('mySend(address,uint256)').slice(0, 10);

    /**
     * Sets up the initial state for the mock Ethereum provider by resolving the required RPC calls.
     * This function helps prepare the test environment before executing test cases related to contract methods.
     * The bootstrap function configures the responses for eth_sendTransaction, eth_blockNumber,
     * and eth_getTransactionReceipt in the mock Ethereum provider.
     */
    function bootstrap() {
      // eth_sendTransaction
      mockEthereumProvider.request.mockResolvedValueOnce(
        '0x1234000000000000000000000000000000000000000000000000000000056789',
      );

      // eth_blockNumber
      mockEthereumProvider.request.mockResolvedValueOnce('0xa');

      // eth_getTransactionReceipt
      mockEthereumProvider.request.mockResolvedValueOnce({
        from: address2Lowercase,
        to: addressLowercase,
        contractAddress: null,
        cumulativeGasUsed: '0xa',
        transactionIndex: '0x3',
        transactionHash: '0x1234555555555555555555555555555555555555555555555555555555555555',
        blockNumber: '0xa',
        blockHash: '0x1234',
        gasUsed: '0x0',
        status: '0x1',
        logs: [
          {
            address: addressLowercase,
            topics: [
              sha3('Unchanged(uint256,address,uint256)'),
              '0x0000000000000000000000000000000000000000000000000000000000000002',
              '0x000000000000000000000000' + addressLowercase.replace('0x', ''),
            ],
            blockNumber: '0xa',
            transactionHash: '0x1234555555555555555555555555555555555555555555555555555555555555',
            transactionIndex: '0x0',
            blockHash: '0x1345',
            logIndex: '0x0',
            data: '0x0000000000000000000000000000000000000000000000000000000000000005',
          },
          {
            address: addressLowercase,
            topics: [
              sha3('Changed(address,uint256,uint256,uint256)'),
              '0x000000000000000000000000' + addressLowercase.replace('0x', ''),
              '0x0000000000000000000000000000000000000000000000000000000000000001',
            ],
            blockNumber: '0xa',
            transactionHash: '0x1234555555555555555555555555555555555555555555555555555555555555',
            transactionIndex: '0x0',
            blockHash: '0x1345',
            logIndex: '0x1',
            data:
              '0x0000000000000000000000000000000000000000000000000000000000000001' +
              '0000000000000000000000000000000000000000000000000000000000000008',
          },
          {
            address: address2Lowercase,
            topics: [sha3('IgnoredDueToUnmatchingAddress()')],
            blockNumber: '0xa',
            transactionHash: '0x1234555555555555555555555555555555555555555555555555555555555555',
            transactionIndex: '0x0',
            blockHash: '0x1345',
            logIndex: '0x2',
            data: '0x',
          },
        ],
      });
    }

    it('should create correct receipt', async () => {
      bootstrap();

      const contract = new TestContract(eth, address);

      const receipt = await contract.methods
        .mySend(address, 10n)
        .send({ from: address2, maxFeePerGas: 21345678654321n })
        .getReceipt();

      expect(receipt).toEqual({
        from: address2,
        to: address,
        cumulativeGasUsed: 10,
        transactionIndex: 3,
        transactionHash: TxHash.fromString('0x1234555555555555555555555555555555555555555555555555555555555555'),
        blockNumber: 10,
        blockHash: '0x1234',
        gasUsed: 0,
        contractAddress: undefined,
        status: true,
        logs: expect.any(Array),
        anonymousLogs: expect.any(Array),
        events: expect.any(Object),
      });
    });

    it('should correctly filter receipts anonymous logs', async () => {
      bootstrap();

      const contract = new TestContract(eth, address);

      const receipt = await contract.methods
        .mySend(address, 10n)
        .send({ from: address2, maxFeePerGas: 21345678654321n })
        .getReceipt();

      expect(receipt.anonymousLogs).toMatchObject([
        {
          address: address2,
          topics: [sha3('IgnoredDueToUnmatchingAddress()')],
          blockNumber: 10,
          transactionHash: TxHash.fromString('0x1234555555555555555555555555555555555555555555555555555555555555'),
          transactionIndex: 0,
          blockHash: '0x1345',
          logIndex: 2,
          data: '0x',
        },
      ]);
    });

    it('should correctly extract receipts events', async () => {
      bootstrap();

      const contract = new TestContract(eth, address);

      const receipt = await contract.methods
        .mySend(address, 10n)
        .send({ from: address2, maxFeePerGas: 21345678654321n })
        .getReceipt();

      expect(receipt.events).toMatchObject({
        Unchanged: [
          {
            address,
            blockNumber: 10,
            transactionHash: TxHash.fromString('0x1234555555555555555555555555555555555555555555555555555555555555'),
            blockHash: '0x1345',
            logIndex: 0,
            transactionIndex: 0,
            args: expect.any(Object),
            event: 'Unchanged',
            signature: '0xf359668f205d0b5cfdc20d11353e05f633f83322e96f15486cbb007d210d66e5',
            raw: {
              topics: [
                '0xf359668f205d0b5cfdc20d11353e05f633f83322e96f15486cbb007d210d66e5',
                '0x0000000000000000000000000000000000000000000000000000000000000002',
                '0x000000000000000000000000' + addressUnprefixedLowercase,
              ],
              data: '0x0000000000000000000000000000000000000000000000000000000000000005',
            },
          },
        ],
        Changed: [
          {
            address,
            blockNumber: 10,
            transactionHash: TxHash.fromString('0x1234555555555555555555555555555555555555555555555555555555555555'),
            blockHash: '0x1345',
            logIndex: 1,
            transactionIndex: 0,
            args: expect.any(Object),
            event: 'Changed',
            signature: '0x792991ed5ba9322deaef76cff5051ce4bedaaa4d097585970f9ad8f09f54e651',
            raw: {
              topics: [
                '0x792991ed5ba9322deaef76cff5051ce4bedaaa4d097585970f9ad8f09f54e651',
                '0x000000000000000000000000' + addressUnprefixedLowercase,
                '0x0000000000000000000000000000000000000000000000000000000000000001',
              ],
              data: '0x00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000008',
            },
          },
        ],
      });
    });

    it('should correctly decode receipts event logs', async () => {
      bootstrap();

      const contract = new TestContract(eth, address);

      const receipt = await contract.methods
        .mySend(address, 10n)
        .send({ from: address2, maxFeePerGas: 21345678654321n })
        .getReceipt();

      expect(receipt.events!.Changed[0].args).toEqual({
        0: address,
        1: 1n,
        2: 1n,
        3: 8n,
        from: address,
        amount: 1n,
        t1: 1n,
        t2: 8n,
      });

      expect(receipt.events!.Unchanged[0].args).toEqual({
        0: 2n,
        1: address,
        2: 5n,
        value: 2n,
        addressFrom: address,
        t1: 5n,
      });
    });

    it('should correctly decode multiple of the same event log', async () => {
      // eth_sendTransaction
      mockEthereumProvider.request.mockResolvedValueOnce(
        '0x1234000000000000000000000000000000000000000000000000000000056789',
      );

      // eth_blockNumber
      mockEthereumProvider.request.mockResolvedValueOnce('0xa');

      mockEthereumProvider.request.mockResolvedValueOnce({
        from: address2Lowercase,
        to: addressLowercase,
        contractAddress: null,
        cumulativeGasUsed: '0xa',
        transactionIndex: '0x3',
        transactionHash: '0x1234555555555555555555555555555555555555555555555555555555555555',
        blockNumber: '0xa',
        blockHash: '0x1234',
        gasUsed: '0x0',
        status: '0x1',
        logs: [
          {
            address: address.toString(),
            topics: [
              sha3('Changed(address,uint256,uint256,uint256)'),
              '0x000000000000000000000000' + addressLowercase.replace('0x', ''),
              '0x0000000000000000000000000000000000000000000000000000000000000001',
            ],
            blockNumber: '0xa',
            transactionHash: '0x1234555555555555555555555555555555555555555555555555555555555555',
            transactionIndex: '0x0',
            blockHash: '0x1345',
            logIndex: '0x4',
            data:
              '0x0000000000000000000000000000000000000000000000000000000000000001' +
              '0000000000000000000000000000000000000000000000000000000000000008',
          },
          {
            address: address.toString(),
            topics: [
              sha3('Changed(address,uint256,uint256,uint256)'),
              '0x000000000000000000000000' + addressLowercase.replace('0x', ''),
              '0x0000000000000000000000000000000000000000000000000000000000000002',
            ],
            blockNumber: '0xa',
            transactionHash: '0x1234555555555555555555555555555555555555555555555555555555555555',
            transactionIndex: '0x0',
            blockHash: '0x1345',
            logIndex: '0x5',
            data:
              '0x0000000000000000000000000000000000000000000000000000000000000001' +
              '0000000000000000000000000000000000000000000000000000000000000008',
          },
        ],
      });

      const contract = new TestContract(eth, address);

      const receipt = await contract.methods
        .mySend(address, 10n)
        .send({ from: address2, maxFeePerGas: 21345678654321n })
        .getReceipt();

      expect(receipt.events).toMatchObject({
        Changed: [
          {
            address,
            blockNumber: 10,
            transactionHash: TxHash.fromString('0x1234555555555555555555555555555555555555555555555555555555555555'),
            blockHash: '0x1345',
            logIndex: 4,
            transactionIndex: 0,
            args: {
              0: address,
              1: 1n,
              2: 1n,
              3: 8n,
              from: address,
              amount: 1n,
              t1: 1n,
              t2: 8n,
            },
            event: 'Changed',
            signature: '0x792991ed5ba9322deaef76cff5051ce4bedaaa4d097585970f9ad8f09f54e651',
            raw: {
              topics: [
                '0x792991ed5ba9322deaef76cff5051ce4bedaaa4d097585970f9ad8f09f54e651',
                '0x000000000000000000000000' + addressLowercase.replace('0x', ''),
                '0x0000000000000000000000000000000000000000000000000000000000000001',
              ],
              data: '0x00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000008',
            },
          },
          {
            address,
            blockNumber: 10,
            transactionHash: TxHash.fromString('0x1234555555555555555555555555555555555555555555555555555555555555'),
            blockHash: '0x1345',
            logIndex: 5,
            transactionIndex: 0,
            args: {
              0: address,
              1: 2n,
              2: 1n,
              3: 8n,
              from: address,
              amount: 2n,
              t1: 1n,
              t2: 8n,
            },
            event: 'Changed',
            signature: '0x792991ed5ba9322deaef76cff5051ce4bedaaa4d097585970f9ad8f09f54e651',
            raw: {
              topics: [
                '0x792991ed5ba9322deaef76cff5051ce4bedaaa4d097585970f9ad8f09f54e651',
                '0x000000000000000000000000' + addressLowercase.replace('0x', ''),
                '0x0000000000000000000000000000000000000000000000000000000000000002',
              ],
              data: '0x00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000008',
            },
          },
        ],
      });
    });

    it('should sendTransaction and receive multiple confirmations', async () => {
      // eth_sendTransaction
      mockEthereumProvider.request.mockResolvedValueOnce(
        '0x1234000000000000000000000000000000000000000000000000000000056789',
      );

      // eth_blockNumber
      mockEthereumProvider.request.mockResolvedValueOnce('0xa');

      // eth_getTransactionReceipt
      mockEthereumProvider.request.mockResolvedValueOnce({
        from: address2Lowercase,
        cumulativeGasUsed: '0xa',
        transactionIndex: '0x3',
        transactionHash: '0x1234000000000000000000000000000000000000000000000000000000056789',
        blockNumber: '0xa',
        blockHash: '0x1234',
        gasUsed: '0x0',
        logs: [],
        status: '0x1',
      });

      // eth_blockNumber
      mockEthereumProvider.request.mockResolvedValueOnce('0xb');

      // eth_getTransactionReceipt
      mockEthereumProvider.request.mockResolvedValueOnce({
        from: address2Lowercase,
        cumulativeGasUsed: '0xa',
        transactionIndex: '0x3',
        transactionHash: '0x1234000000000000000000000000000000000000000000000000000000056789',
        blockNumber: '0xa',
        blockHash: '0x1234',
        gasUsed: '0x0',
        logs: [],
        status: '0x1',
      });

      const contract = new TestContract(eth, address);

      const receipt = await contract.methods
        .mySend(address, 10n)
        .send({ from: address2, maxFeePerGas: 21345678654321n })
        .getReceipt(true, 2, 0, 0);

      expect(receipt).toEqual({
        from: address2,
        cumulativeGasUsed: 10,
        transactionIndex: 3,
        transactionHash: TxHash.fromString('0x1234000000000000000000000000000000000000000000000000000000056789'),
        blockNumber: 10,
        blockHash: '0x1234',
        gasUsed: 0,
        logs: [],
        anonymousLogs: [],
        events: {},
        status: true,
      });
    });

    it('should sendTransaction to contract function', async () => {
      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_sendTransaction');
        expect(params).toEqual([
          {
            data:
              signature +
              '000000000000000000000000' +
              addressLowercase.replace('0x', '') +
              '0000000000000000000000000000000000000000000000000000000000000011',
            from: addressLowercase,
            to: addressLowercase,
            maxFeePerGas: '0x369d1f7fd2',
          },
        ]);
        return Promise.resolve();
      });

      const contract = new TestContract(eth, address);

      await contract.methods.mySend(address, 17n).send({ from: address, maxFeePerGas: 234564321234n }).getTxHash();
    });

    it('should throw error when trying to send ether to a non payable contract function', () => {
      const contract = new TestContract(eth, address);

      expect(() => contract.methods.myDisallowedSend(address, 17n).send({ from: address, value: 123n })).toThrowError(
        /non-payable/,
      );
    });

    it('should not throw error when trying to not send ether to a non payable contract function', async () => {
      // const signature = 'myDisallowedSend(address,uint256)';

      // mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
      //   expect(method).toBe('eth_sendTransaction');
      //   expect(params).toEqual([
      //     {
      //       data:
      //         sha3(signature).slice(0, 10) +
      //         '000000000000000000000000' +
      //         addressLowercase.replace('0x', '') +
      //         '0000000000000000000000000000000000000000000000000000000000000011',
      //       from: addressLowercase,
      //       to: addressLowercase,
      //       maxFeePerGas: '0x1555757ee6b1',
      //     },
      //   ]);
      //   return Promise.resolve();
      // });

      // mockEthereumProvider.request.mockResolvedValueOnce({
      //   contractAddress: null,
      //   cumulativeGasUsed: '0xa',
      //   transactionIndex: '0x3',
      //   transactionHash: '0x1234',
      //   blockNumber: '0xa',
      //   blockHash: '0x1234',
      //   gasUsed: '0x0',
      //   logs: [],
      // });

      const contract = new TestContract(eth, address);

      await contract.methods
        .myDisallowedSend(address, 17n)
        .send({ from: address, maxFeePerGas: 23456787654321n })
        .getTxHash();
    });

    it('should sendTransaction to contract function using the function name incl. parameters', async () => {
      // eth_sendTransaction
      mockEthereumProvider.request.mockResolvedValueOnce(
        '0x1234000000000000000000000000000000000000000000000000000000056789',
      );

      // eth_getTransactionReceipt
      mockEthereumProvider.request.mockResolvedValueOnce({
        blockHash: '0x1234',
      });

      const contract = new TestContract(eth, address);

      await contract.methods['mySend(address,uint256)'](address, 17)
        .send({
          from: address,
          maxFeePerGas: 23456787654321n,
        })
        .getTxHash();

      expect(mockEthereumProvider.request).toHaveBeenCalledWith({
        method: 'eth_sendTransaction',
        params: [
          {
            data:
              signature +
              '000000000000000000000000' +
              addressLowercase.replace('0x', '') +
              '0000000000000000000000000000000000000000000000000000000000000011',
            from: addressLowercase,
            to: addressLowercase,
            maxFeePerGas: '0x1555757ee6b1',
          },
        ],
      });
    });

    it('should sendTransaction to contract function using the signature', async () => {
      // eth_sendTransaction
      mockEthereumProvider.request.mockResolvedValueOnce(
        '0x1234000000000000000000000000000000000000000000000000000000056789',
      );

      // eth_getTransactionReceipt
      mockEthereumProvider.request.mockResolvedValueOnce({
        blockHash: '0x1234',
      });

      const contract = new TestContract(eth, address);

      await contract.methods[signature](address, 17).send({ from: address, maxFeePerGas: 1230000000n }).getTxHash();

      expect(mockEthereumProvider.request).toHaveBeenCalledWith({
        method: 'eth_sendTransaction',
        params: [
          {
            data:
              signature +
              '000000000000000000000000' +
              addressLowercase.replace('0x', '') +
              '0000000000000000000000000000000000000000000000000000000000000011',
            from: addressLowercase,
            to: addressLowercase,
            maxFeePerGas: '0x49504f80',
          },
        ],
      });
    });

    it('should throw when trying to create a tx object and wrong amount of params', () => {
      const contract = new Contract(eth, TestContractAbi, address);
      expect(() => contract.methods.mySend(address)).toThrowError('No matching method with 1 arguments for mySend.');
    });

    it('should make a call with optional params', async () => {
      const signature = 'balance(address)';
      let count = 0;

      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        count++;
        if (count > 1) {
          return Promise.resolve();
        }

        expect(method).toBe('eth_call');
        expect(params).toEqual([
          {
            data: sha3(signature).slice(0, 10) + '000000000000000000000000' + addressLowercase.replace('0x', ''),
            to: addressLowercase,
            from: addressLowercase,
            gas: '0xc350',
          },
          'latest',
        ]);
        return Promise.resolve('0x0000000000000000000000000000000000000000000000000000000000000032');
      });

      const contract = new TestContract(eth, address);

      const r = await contract.methods.balance(address).call({ from: address, gas: 50000 });
      expect(r).toBe(50n);
    });

    it('should explicitly make a call with optional params', async () => {
      const signature = 'balance(address)';

      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_call');
        expect(params).toEqual([
          {
            data: sha3(signature).slice(0, 10) + '000000000000000000000000' + addressLowercase.replace('0x', ''),
            to: addressLowercase,
            from: addressLowercase,
            gas: '0xc350',
          },
          'latest',
        ]);
        return Promise.resolve('0x0000000000000000000000000000000000000000000000000000000000000032');
      });

      const contract = new TestContract(eth, address);

      const r = await contract.methods.balance(address).call({ from: address, gas: 50000 });
      expect(r).toBe(50n);
    });

    it('should explicitly make a call with optional params and defaultBlock', async () => {
      const signature = 'balance(address)';

      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_call');
        expect(params).toEqual([
          {
            data: sha3(signature).slice(0, 10) + '000000000000000000000000' + addressLowercase.replace('0x', ''),
            to: addressLowercase,
            from: addressLowercase,
            gas: '0xc350',
          },
          '0xb',
        ]);
        return Promise.resolve('0x0000000000000000000000000000000000000000000000000000000000000032');
      });

      const contract = new TestContract(eth, address);

      const r = await contract.methods.balance(address).call({ from: address, gas: 50000 }, 11);
      expect(r).toBe(50n);
    });

    it('should sendTransaction with optional params', async () => {
      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_sendTransaction');
        expect(params).toEqual([
          {
            data:
              signature +
              '000000000000000000000000' +
              addressLowercase.replace('0x', '') +
              '0000000000000000000000000000000000000000000000000000000000000011',
            to: addressLowercase,
            from: addressLowercase,
            gas: '0xc350',
            maxFeePerGas: '0xbb8',
            value: '0x2710',
          },
        ]);
        return Promise.resolve();
      });

      // eth_getTransactionReceipt
      mockEthereumProvider.request.mockResolvedValueOnce({
        blockHash: '0x1234',
      });

      const contract = new TestContract(eth, address);

      await contract.methods
        .mySend(address, 17n)
        .send({ from: address, gas: 50000, maxFeePerGas: 3000n, value: 10000n })
        .getTxHash();
    });

    // it.only('should sendTransaction and fill in default gasPrice', async () => {
    //   mockEthereumProvider.request.mockImplementationOnce(async ({ method, params }) => {
    //     expect(method).toBe('eth_gasPrice');
    //     return '0x45656456456456';
    //   });

    //   mockEthereumProvider.request.mockImplementationOnce(async ({ method, params }) => {
    //     expect(method).toBe('eth_sendTransaction');
    //     expect(params).toEqual([
    //       {
    //         data:
    //           signature +
    //           '000000000000000000000000' +
    //           addressLowercase.replace('0x', '') +
    //           '0000000000000000000000000000000000000000000000000000000000000011',
    //         to: addressLowercase,
    //         from: addressLowercase,
    //         maxFeePerGas: '0x45656456456456',
    //       },
    //     ]);
    //   });

    //   // eth_getTransactionReceipt
    //   mockEthereumProvider.request.mockResolvedValueOnce({
    //     blockHash: '0x1234',
    //   });

    //   const contract = new TestContract(eth, address);

    //   await contract.methods.mySend(address, 17).send({ from: address });
    // });

    it('should explicitly sendTransaction with optional params', async () => {
      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_sendTransaction');
        expect(params).toEqual([
          {
            data:
              signature +
              '000000000000000000000000' +
              addressLowercase.replace('0x', '') +
              '0000000000000000000000000000000000000000000000000000000000000011',
            to: addressLowercase,
            from: addressLowercase,
            gas: '0xc350',
            maxFeePerGas: '0xbb8',
            value: '0x2710',
          },
        ]);
        return Promise.resolve();
      });

      // eth_getTransactionReceipt
      mockEthereumProvider.request.mockResolvedValueOnce({
        blockHash: '0x1234',
      });

      const contract = new TestContract(eth, address);

      await contract.methods
        .mySend(address, 17n)
        .send({ from: address, gas: 50000, maxFeePerGas: 3000n, value: 10000n })
        .getTxHash();
    });

    it('should explicitly estimateGas with optional params', async () => {
      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_estimateGas');
        expect(params).toEqual([
          {
            data:
              signature +
              '000000000000000000000000' +
              addressUnprefixedLowercase +
              '0000000000000000000000000000000000000000000000000000000000000011',
            to: addressLowercase,
            from: addressLowercase,
            gas: '0xc350',
            maxFeePerGas: '0xbb8',
            value: '0x2710',
          },
        ]);

        return Promise.resolve('0x10');
      });

      // eth_getTransactionReceipt
      mockEthereumProvider.request.mockResolvedValueOnce({
        blockHash: '0x1234',
      });

      const contract = new TestContract(eth, address);

      const gasUsed = await contract.methods
        .mySend(address, 17n)
        .estimateGas({ from: address, gas: 50000, maxFeePerGas: 3000n, value: 10000n });

      expect(gasUsed).toBe(16);
    });

    it('getPastEvents should get past events and format them correctly', async () => {
      const signature = 'Changed(address,uint256,uint256,uint256)';

      const topic1 = [
        sha3(signature),
        '0x000000000000000000000000' + address.toString().replace('0x', ''),
        '0x000000000000000000000000000000000000000000000000000000000000000a',
      ];
      const topic2 = [
        sha3(signature),
        '0x000000000000000000000000' + address.toString().replace('0x', ''),
        '0x0000000000000000000000000000000000000000000000000000000000000003',
      ];

      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_getLogs');
        expect(params).toEqual([
          {
            address: addressLowercase,
            topics: [
              '0x792991ed5ba9322deaef76cff5051ce4bedaaa4d097585970f9ad8f09f54e651',
              '0x000000000000000000000000' + address2.toString().replace('0x', ''),
              null,
            ],
          },
        ]);

        return Promise.resolve([
          {
            address: address.toString(),
            topics: topic1,
            blockNumber: '0x3',
            transactionHash: '0x1234555555555555555555555555555555555555555555555555555555555555',
            transactionIndex: '0x0',
            blockHash: '0x1345',
            logIndex: '0x4',
            data:
              '0x0000000000000000000000000000000000000000000000000000000000000002' +
              '0000000000000000000000000000000000000000000000000000000000000009',
          },
          {
            address: address.toString(),
            topics: topic2,
            blockNumber: '0x4',
            transactionHash: '0x1235555555555555555555555555555555555555555555555555555555555555',
            transactionIndex: '0x0',
            blockHash: '0x1346',
            logIndex: '0x1',
            data:
              '0x0000000000000000000000000000000000000000000000000000000000000004' +
              '0000000000000000000000000000000000000000000000000000000000000005',
          },
        ]);
      });

      const contract = new TestContract(eth, address);

      const result = await contract.getLogs('Changed', { filter: { from: address2 } });

      expect(result).toMatchObject([
        {
          event: 'Changed',
          signature: '0x792991ed5ba9322deaef76cff5051ce4bedaaa4d097585970f9ad8f09f54e651',
          address,
          blockNumber: 3,
          transactionHash: TxHash.fromString('0x1234555555555555555555555555555555555555555555555555555555555555'),
          blockHash: '0x1345',
          logIndex: 4,
          transactionIndex: 0,
          raw: {
            data:
              '0x0000000000000000000000000000000000000000000000000000000000000002' +
              '0000000000000000000000000000000000000000000000000000000000000009',
            topics: topic1,
          },
          args: {
            0: address,
            1: 10n,
            2: 2n,
            3: 9n,
            from: address,
            amount: 10n,
            t1: 2n,
            t2: 9n,
          },
        },
        {
          event: 'Changed',
          signature: '0x792991ed5ba9322deaef76cff5051ce4bedaaa4d097585970f9ad8f09f54e651',
          address,
          blockNumber: 4,
          transactionHash: TxHash.fromString('0x1235555555555555555555555555555555555555555555555555555555555555'),
          blockHash: '0x1346',
          logIndex: 1,
          transactionIndex: 0,
          raw: {
            data:
              '0x0000000000000000000000000000000000000000000000000000000000000004' +
              '0000000000000000000000000000000000000000000000000000000000000005',
            topics: topic2,
          },
          args: {
            0: address,
            1: 3n,
            2: 4n,
            3: 5n,
            from: address,
            amount: 3n,
            t1: 4n,
            t2: 5n,
          },
        },
      ]);
    });

    it('should call testArr method and properly parse result', async () => {
      const signature = 'testArr(int[])';

      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_call');
        expect(params).toEqual([
          {
            data:
              sha3(signature).slice(0, 10) +
              '0000000000000000000000000000000000000000000000000000000000000020' +
              '0000000000000000000000000000000000000000000000000000000000000001' +
              '0000000000000000000000000000000000000000000000000000000000000003',
            to: addressLowercase,
          },
          'latest',
        ]);
        return Promise.resolve('0x0000000000000000000000000000000000000000000000000000000000000005');
      });

      // eth_getTransactionReceipt
      mockEthereumProvider.request.mockResolvedValueOnce({
        blockHash: '0x1234',
      });

      const contract = new TestContract(eth, address);

      const result = await contract.methods.testArr([3n]).call();
      expect(result).toBe(5n);
    });

    it('should call owner method, properly', async () => {
      const signature = 'owner()';

      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_call');
        expect(params).toEqual([
          {
            data: sha3(signature).slice(0, 10),
            to: addressLowercase,
          },
          'latest',
        ]);
        return Promise.resolve('0x000000000000000000000000' + addressLowercase.replace('0x', ''));
      });

      // eth_getTransactionReceipt
      mockEthereumProvider.request.mockResolvedValueOnce({
        blockHash: '0x1234',
      });

      const contract = new TestContract(eth, address);

      const result = await contract.methods.owner().call();
      expect(result).toEqual(address);
    });

    it('should decode an struct correctly', async () => {
      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_call');
        expect(params).toEqual([
          {
            data: '0x2a4aedd5000000000000000000000000' + addressUnprefixedLowercase,
            to: addressLowercase,
          },
          'latest',
        ]);
        return Promise.resolve('0x0000000000000000000000000000000000000000000000000000000000000001');
      });

      const contract = new TestContract(eth, address);

      const result = await contract.methods.listOfNestedStructs(address).call();
      const expectedArray: any = [];
      expectedArray[0] = true;
      expectedArray.status = true;

      expect(result).toEqual(expectedArray);
    });

    it('should call an contract method with an struct as parameter', async () => {
      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_sendTransaction');
        expect(params).toEqual([
          {
            data: '0x814a4d160000000000000000000000000000000000000000000000000000000000000001',
            from: addressLowercase,
            gas: '0xc350',
            maxFeePerGas: '0xbb8',
            to: addressLowercase,
          },
        ]);
        return Promise.resolve();
      });

      // eth_getTransactionReceipt
      mockEthereumProvider.request.mockResolvedValueOnce({
        blockHash: '0x1234',
      });

      const contract = new TestContract(eth, address);

      await contract.methods
        .addStruct({ status: true })
        .send({
          from: address,
          gas: 50000,
          maxFeePerGas: 3000n,
        })
        .getTxHash();
    });
  });

  describe('encodeABI', () => {
    const abi = new ContractAbi([
      {
        constant: true,
        inputs: [
          {
            name: 'a',
            type: 'bytes32',
          },
          {
            name: 'b',
            type: 'bytes32',
          },
        ],
        name: 'takesTwoBytes32',
        outputs: [
          {
            name: '',
            type: 'bytes32',
          },
        ],
        payable: false,
        type: 'function',
        stateMutability: 'view',
      },
    ]);
    const contractAddress = EthAddress.fromString('0x11f4d0A3c12e86B4b5F39B213F7E19D048276DAe');
    const mockEthereumProvider = mock<EthereumProvider>();
    const eth = new EthereumRpc(mockEthereumProvider);

    it('should handle bytes32 arrays that only contain 1 byte', () => {
      const contract = new Contract(eth, abi, contractAddress);

      const result = contract.methods.takesTwoBytes32('0xaa', '0xbb').encodeABI();

      expect(bufferToHex(result)).toBe(
        [
          '0x1323517e',
          'aa00000000000000000000000000000000000000000000000000000000000000',
          'bb00000000000000000000000000000000000000000000000000000000000000',
        ].join(''),
      );
    });

    it('should handle bytes32 arrays that are short 1 byte', () => {
      const contract = new Contract(eth, abi, contractAddress);

      const result = contract.methods
        .takesTwoBytes32('0x'.concat('a'.repeat(62)), '0x'.concat('b'.repeat(62)))
        .encodeABI();

      expect(bufferToHex(result)).toBe(
        [
          '0x1323517e',
          'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa00',
          'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb00',
        ].join(''),
      );
    });

    it('should handle bytes32 arrays that are full', () => {
      const contract = new Contract(eth, abi, contractAddress);

      const result = contract.methods
        .takesTwoBytes32('0x'.concat('a'.repeat(64)), '0x'.concat('b'.repeat(64)))
        .encodeABI();

      expect(bufferToHex(result)).toBe(
        [
          '0x1323517e',
          'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
          'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
        ].join(''),
      );
    });

    it('should throw an exception on bytes32 arrays that are too long', () => {
      const contract = new Contract(eth, abi, contractAddress);

      const test = () =>
        contract.methods.takesTwoBytes32('0x'.concat('a'.repeat(66)), '0x'.concat('b'.repeat(66))).encodeABI();

      expect(test).toThrow();
    });
  });

  describe('deploy', () => {
    const txReceipt: RawTransactionReceipt = {
      transactionHash: '0x1234555555555555555555555555555555555555555555555555555555555555',
      to: null,
      from: addressLowercase,
      contractAddress: address2Lowercase,
      blockHash: '0xffdd',
      transactionIndex: '0x0',
      blockNumber: '0x1',
      cumulativeGasUsed: '0x1',
      gasUsed: '0x1',
      logs: [],
      status: '0x1',
    };

    it('should deploy a contract and use all promise steps', async () => {
      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_sendTransaction');
        expect(params).toEqual([
          {
            data:
              '0x01234567000000000000000000000000' +
              addressUnprefixedLowercase +
              '00000000000000000000000000000000000000000000000000000000000000c8',
            from: addressLowercase,
            gas: '0xc350',
            maxFeePerGas: '0xbb8',
          },
        ]);
        return Promise.resolve('0x5550000000000000000000000000000000000000000000000000000000000032');
      });

      mockEthereumProvider.request.mockImplementationOnce(({ method }) => {
        expect(method).toBe('eth_blockNumber');
        return Promise.resolve('0x0');
      });

      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_getTransactionReceipt');
        expect(params).toEqual(['0x5550000000000000000000000000000000000000000000000000000000000032']);
        return Promise.resolve(null);
      });

      mockEthereumProvider.request.mockImplementationOnce(({ method }) => {
        expect(method).toBe('eth_blockNumber');
        return Promise.resolve('0x1');
      });

      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_getTransactionReceipt');
        expect(params).toEqual(['0x5550000000000000000000000000000000000000000000000000000000000032']);
        return Promise.resolve(txReceipt);
      });

      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_getCode');
        expect(params).toEqual([address2Lowercase, 'latest']);
        return Promise.resolve('0x321');
      });

      const contract = new TestContract(eth);

      const sendTx = contract.deployBytecode('0x01234567', address, 200).send({
        from: address,
        gas: 50000,
        maxFeePerGas: 3000n,
      });

      const txHash = await sendTx.getTxHash();
      const receipt = await sendTx.getReceipt(true, 1, 0, 0);

      expect(txHash).toBe('0x5550000000000000000000000000000000000000000000000000000000000032');
      expect(receipt.contractAddress).toEqual(address2);
    });

    it('should deploy a contract with no ctor', async () => {
      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_sendTransaction');
        expect(params).toEqual([
          {
            data: '0x01234567',
            from: addressLowercase,
            gas: '0xc350',
            maxFeePerGas: '0xbb8',
          },
        ]);
        return Promise.resolve('0x5550000000000000000000000000000000000000000000000000000000000032');
      });

      // eth_blockNumber
      mockEthereumProvider.request.mockResolvedValueOnce('0x1');

      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_getTransactionReceipt');
        expect(params).toEqual(['0x5550000000000000000000000000000000000000000000000000000000000032']);
        return Promise.resolve(txReceipt);
      });

      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_getCode');
        expect(params).toEqual([address2Lowercase, 'latest']);
        return Promise.resolve('0x321100');
      });

      const contract = new TestNoCtorContract(eth);

      const sendTx = contract.deploy().send({
        from: address,
        gas: 50000,
        maxFeePerGas: 3000n,
      });

      const txHash = await sendTx.getTxHash();
      const receipt = await sendTx.getReceipt();

      expect(txHash.toString()).toBe('0x5550000000000000000000000000000000000000000000000000000000000032');
      expect(receipt.contractAddress).toEqual(address2);
    });
  });
});
