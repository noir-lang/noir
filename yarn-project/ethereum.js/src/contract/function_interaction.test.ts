import { EthAddress } from '@aztec/foundation/eth-address';
import { mock } from 'jest-mock-extended';
import { EthereumProvider } from '../provider/ethereum_provider.js';
import { sha3 } from '../crypto/index.js';
import { ContractAbi } from './abi/contract_abi.js';
import { FunctionInteraction } from './function_interaction.js';
import { TxHash, EthereumRpc } from '../eth_rpc/index.js';

describe('contract', () => {
  describe('function_interaction', () => {
    const contractAddress = EthAddress.fromString('0x11f4d0A3c12e86B4b5F39B213F7E19D048276DAe');
    const contractAddressLowercase = contractAddress.toString().toLowerCase();
    const contractAddressUnprefixedLowercase = contractAddressLowercase.slice(2);
    const from = EthAddress.fromString('0x5555567890123456789012345678901234567891');
    const fromAddressLowercase = from.toString().toLowerCase();
    let mockEthereumProvider: ReturnType<typeof mock<EthereumProvider>>;

    beforeEach(() => {
      mockEthereumProvider = mock<EthereumProvider>();
    });

    it('should emit correct transaction hash and receipt on send', async () => {
      const signature = sha3('mySend(address,uint256)').slice(0, 10);

      const contractAbi = new ContractAbi([
        {
          name: 'mySend',
          type: 'function',
          inputs: [
            {
              name: 'to',
              type: 'address',
            },
            {
              name: 'value',
              type: 'uint256',
            },
          ],
          outputs: [],
        },
      ]);
      const methodAbi = contractAbi.functions[0];

      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_sendTransaction');
        expect(params).toEqual([
          {
            data:
              signature +
              '000000000000000000000000' +
              contractAddressUnprefixedLowercase +
              '000000000000000000000000000000000000000000000000000000000000000a',
            from: fromAddressLowercase,
            to: contractAddressLowercase,
            // gasPrice: '0x5af3107a4000',
          },
        ]);
        return Promise.resolve('0x1234000000000000000000000000000000000000000000000000000000056789');
      });

      mockEthereumProvider.request.mockImplementationOnce(({ method }) => {
        expect(method).toBe('eth_blockNumber');
        return Promise.resolve('0xa');
      });

      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_getTransactionReceipt');
        expect(params).toEqual(['0x1234000000000000000000000000000000000000000000000000000000056789']);
        return Promise.resolve({
          from: fromAddressLowercase,
          contractAddress: contractAddressLowercase,
          cumulativeGasUsed: '0xa',
          transactionIndex: '0x3',
          transactionHash: '0x1234000000000000000000000000000000000000000000000000000000056789',
          blockNumber: '0xa',
          blockHash: '0xbf1234',
          gasUsed: '0x0',
          status: '0x1',
          logs: [],
        });
      });

      const args = [contractAddress, 10];
      const tx = new FunctionInteraction(
        new EthereumRpc(mockEthereumProvider),
        methodAbi,
        contractAbi,
        contractAddress,
        args,
      );

      const txSend = tx.send({ from });

      expect(await txSend.getTxHash()).toBe('0x1234000000000000000000000000000000000000000000000000000000056789');
      expect(await txSend.getReceipt()).toEqual({
        from,
        contractAddress,
        cumulativeGasUsed: 10,
        transactionIndex: 3,
        transactionHash: TxHash.fromString('0x1234000000000000000000000000000000000000000000000000000000056789'),
        blockNumber: 10,
        blockHash: '0xbf1234',
        gasUsed: 0,
        logs: [],
        anonymousLogs: [],
        events: {},
        status: true,
      });
    });

    it('should return correct result on call', async () => {
      const signature = sha3('balance(address)').slice(0, 10);

      const contractAbi = new ContractAbi([
        {
          name: 'balance',
          type: 'function',
          inputs: [
            {
              name: 'who',
              type: 'address',
            },
          ],
          constant: true,
          outputs: [
            {
              name: 'value',
              type: 'uint256',
            },
          ],
        },
      ]);
      const methodAbi = contractAbi.functions[0];

      mockEthereumProvider.request.mockImplementationOnce(({ method, params }) => {
        expect(method).toBe('eth_call');
        expect(params).toEqual([
          {
            data: signature + '000000000000000000000000' + contractAddressUnprefixedLowercase,
            from: from.toString().toLowerCase(),
            to: contractAddressLowercase,
          },
          'latest',
        ]);
        return Promise.resolve('0x000000000000000000000000000000000000000000000000000000000000000a');
      });

      const args = [contractAddress];
      const tx = new FunctionInteraction(
        new EthereumRpc(mockEthereumProvider),
        methodAbi,
        contractAbi,
        contractAddress,
        args,
      );

      const result = await tx.call({ from });
      expect(result).toBe(10n);
    });
  });
});
