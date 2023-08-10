import { AztecAddress, CircuitsWasm, EthAddress, Fr } from '@aztec/circuits.js';
import { randomBytes } from '@aztec/foundation/crypto';
import { Pedersen } from '@aztec/merkle-tree';
import {
  ContractData,
  ContractDataAndBytecode,
  L1ToL2Message,
  L2Block,
  L2BlockL2Logs,
  LogType,
  MerkleTreeId,
  SiblingPath,
  TxHash,
  mockTx,
} from '@aztec/types';

import { jest } from '@jest/globals';

import { HttpNode } from './http-node.js';

const TEST_URL = 'http://aztec-node-url.com/';

const setFetchMock = (response: any, status = 200): void => {
  global.fetch = jest
    .fn<typeof global.fetch>()
    .mockImplementation((_input: RequestInfo | URL, _init?: RequestInit | undefined) => {
      return Promise.resolve({
        status,
        ok: true,
        json: () => response,
        arrayBuffer: () => response,
      } as Response);
    });
};

describe('HttpNode', () => {
  let httpNode: HttpNode;
  let pedersen: Pedersen;

  beforeAll(async () => {
    httpNode = new HttpNode(TEST_URL);
    pedersen = new Pedersen(await CircuitsWasm.get());
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('isReady', () => {
    it.each([true, false])('should return %s when the node is ready', async () => {
      const response = { isReady: true };
      setFetchMock(response);

      const result = await httpNode.isReady();

      expect(fetch).toHaveBeenCalledWith(TEST_URL);
      expect(result).toBe(true);
    });
  });

  describe('getBlocks', () => {
    it('should fetch and parse blocks', async () => {
      const block1 = L2Block.random(1);
      const block2 = L2Block.random(2);
      const response = {
        blocks: [block1.encode(), block2.encode()],
      };
      setFetchMock(response);

      const result = await httpNode.getBlocks(0, 3);

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}get-blocks?from=0&limit=3`);
      expect(result).toEqual([block1, block2]);
    });

    it('should return an empty array if blocks are not available', async () => {
      const response = { blocks: [] };
      setFetchMock(response);

      const result = await httpNode.getBlocks(0, 2);

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}get-blocks?from=0&limit=2`);
      expect(result).toEqual([]);
    });
  });

  describe('getBlock', () => {
    it('should fetch and parse a block', async () => {
      const block1 = L2Block.random(1);
      const response = {
        block: block1.encode(),
      };
      setFetchMock(response);

      const result = await httpNode.getBlock(1);

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}get-block?number=1`);
      expect(result).toEqual(block1);
    });

    it('should return undefined if the block is not available', async () => {
      const response = { block: undefined };
      setFetchMock(response);

      const result = await httpNode.getBlock(2);

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}get-block?number=2`);
      expect(result).toEqual(undefined);
    });
  });

  describe('getBlockHeight', () => {
    it('should fetch and return the block height', async () => {
      const response = { blockHeight: 100 };
      setFetchMock(response);

      const result = await httpNode.getBlockHeight();

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}get-block-height`);
      expect(result).toBe(100);
    });
  });

  describe('getVersion', () => {
    it('should fetch and return the version', async () => {
      const response = { version: 5 };
      setFetchMock(response);

      const result = await httpNode.getVersion();

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}get-version`);
      expect(result).toBe(5);
    });
  });

  describe('getChainId', () => {
    it('should fetch and return the chain ID', async () => {
      const response = { chainId: 1234 };
      setFetchMock(response);

      const result = await httpNode.getChainId();

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}get-chain-id`);
      expect(result).toBe(1234);
    });
  });

  describe('getRollupAddress', () => {
    it('should fetch and return the rollup address', async () => {
      const addr = EthAddress.random();
      const response = { rollupAddress: addr.toString() };
      setFetchMock(response);

      const result = await httpNode.getRollupAddress();

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}get-rollup-address`);
      expect(result).toEqual(addr);
    });
  });

  describe('getContractDataAndBytecode', () => {
    it('should fetch and return contract public data', async () => {
      const contractDataAndBytecode = ContractDataAndBytecode.random();
      const response = {
        contractData: contractDataAndBytecode.toBuffer(),
      };

      setFetchMock(response);

      const result = await httpNode.getContractDataAndBytecode(contractDataAndBytecode.contractData.contractAddress);

      expect(fetch).toHaveBeenCalledWith(
        `${TEST_URL}contract-data-and-bytecode?address=${contractDataAndBytecode.contractData.contractAddress.toString()}`,
      );
      expect(result).toEqual(contractDataAndBytecode);
    });

    it('should return undefined if contract data is not available', async () => {
      const response = {
        contractData: undefined,
      };
      setFetchMock(response);

      const randomAddress = AztecAddress.random();

      const result = await httpNode.getContractDataAndBytecode(randomAddress);

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}contract-data-and-bytecode?address=${randomAddress.toString()}`);
      expect(result).toEqual(undefined);
    });
  });

  describe('getLogs', () => {
    it.each(['encrypted', 'unencrypted'])('should fetch and return %s logs', async logType => {
      const processedLogType = logType === 'encrypted' ? LogType.ENCRYPTED : LogType.UNENCRYPTED;

      const from = 0;
      const limit = 3;
      const log1 = L2BlockL2Logs.random(2, 3, 4);
      const log2 = L2BlockL2Logs.random(1, 5, 2);
      const response = {
        logs: [log1.toBuffer(), log2.toBuffer()],
      };
      setFetchMock(response);

      const result = await httpNode.getLogs(from, limit, processedLogType);

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}get-logs?from=${from}&limit=${limit}&logType=${processedLogType}`);
      expect(result).toEqual([log1, log2]);
    });

    it.each(['encrypted', 'unencrypted'])(
      'should return an empty array if %s logs are not available',
      async logType => {
        const processedLogType = logType === 'encrypted' ? LogType.ENCRYPTED : LogType.UNENCRYPTED;

        const from = 0;
        const limit = 2;
        const response = {};
        setFetchMock(response);

        const result = await httpNode.getLogs(from, limit, processedLogType);

        expect(fetch).toHaveBeenCalledWith(
          `${TEST_URL}get-logs?from=${from}&limit=${limit}&logType=${processedLogType}`,
        );
        expect(result).toEqual([]);
      },
    );
  });

  describe('getContractData', () => {
    it('should fetch and return contract data', async () => {
      const contractData = ContractData.random();
      const response = {
        contractData: contractData.toBuffer(),
      };
      setFetchMock(response);

      const result = await httpNode.getContractData(contractData.contractAddress);

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}contract-data?address=${contractData.contractAddress.toString()}`);
      expect(result).toEqual(contractData);
    });

    it('should return undefined if contract data is not available', async () => {
      const response = {
        contractData: undefined,
      };
      setFetchMock(response);

      const randomAddress = AztecAddress.random();

      const result = await httpNode.getContractData(randomAddress);

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}contract-data?address=${randomAddress.toString()}`);
      expect(result).toBeUndefined();
    });
  });

  describe('sendTx', () => {
    it('should submit a transaction to the p2p pool', async () => {
      const tx = mockTx();
      const irrelevantResponse = {};
      setFetchMock(irrelevantResponse);

      await httpNode.sendTx(tx);

      const init: RequestInit = {
        method: 'POST',
        body: tx.toBuffer(),
      };
      const call = (fetch as jest.Mock).mock.calls[0] as any[];
      expect(call[0].href).toBe(`${TEST_URL}tx`);
      expect(call[1]).toStrictEqual(init);
    });
  });

  describe('getPendingTxByHash', () => {
    it('should fetch and return a pending tx', async () => {
      const txHash = new TxHash(randomBytes(TxHash.SIZE));
      const tx = mockTx();
      const response = tx.toBuffer();
      setFetchMock(response);

      const result = await httpNode.getPendingTxByHash(txHash);

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}get-pending-tx?hash=${txHash}`);
      expect(result).toEqual(tx);
    });

    it('should return undefined if the pending tx does not exist', async () => {
      const txHash = new TxHash(randomBytes(TxHash.SIZE));
      const response = undefined;
      setFetchMock(response, 404);

      const result = await httpNode.getPendingTxByHash(txHash);

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}get-pending-tx?hash=${txHash}`);
      expect(result).toBeUndefined();
    });
  });

  describe('findContractIndex', () => {
    it('should fetch and return the index of the given contract', async () => {
      const leafValue = Buffer.from('abc123', 'hex');
      const index = '123456';
      const response = { index };
      setFetchMock(response);

      const result = await httpNode.findContractIndex(leafValue);

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}contract-index?leaf=${leafValue.toString('hex')}`);
      expect(result).toBe(BigInt(index));
    });

    it('should return undefined if the contract index is not found', async () => {
      const leafValue = Buffer.from('def456', 'hex');
      const response = {};
      setFetchMock(response);

      const result = await httpNode.findContractIndex(leafValue);

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}contract-index?leaf=${leafValue.toString('hex')}`);
      expect(result).toBeUndefined();
    });
  });

  describe('getContractPath', () => {
    it('should fetch and return the sibling path for the given leaf index', async () => {
      const leafIndex = BigInt(123456);
      const siblingPath = SiblingPath.ZERO(3, Buffer.alloc(32), pedersen);
      const response = { path: siblingPath.toBuffer().toString('hex') };
      setFetchMock(response);

      const result = await httpNode.getContractPath(leafIndex);

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}contract-path?leaf=${leafIndex.toString()}`);
      expect(result).toEqual(siblingPath);
    });
  });

  describe('findCommitmentIndex', () => {
    it('should fetch and return the index of the given leaf', async () => {
      const leafValue = Buffer.from('0123456789', 'hex');
      const expectedIndex = BigInt(123);
      const response = { index: expectedIndex.toString() };
      setFetchMock(response);

      const result = await httpNode.findCommitmentIndex(leafValue);

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}commitment-index?leaf=${leafValue.toString('hex')}`);
      expect(result).toBe(expectedIndex);
    });

    it('should return undefined if the commitment index is not found', async () => {
      const leafValue = Buffer.from('def456', 'hex');
      const response = {};
      setFetchMock(response);

      const result = await httpNode.findCommitmentIndex(leafValue);

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}commitment-index?leaf=${leafValue.toString('hex')}`);
      expect(result).toBeUndefined();
    });
  });

  describe('getDataTreePath', () => {
    it('should fetch and return the sibling path for the given leaf index', async () => {
      const leafIndex = BigInt(123456);
      const siblingPath = SiblingPath.ZERO(3, Buffer.alloc(32), pedersen);
      const response = { path: siblingPath.toBuffer().toString('hex') };
      setFetchMock(response);

      const result = await httpNode.getDataTreePath(leafIndex);

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}data-path?leaf=${leafIndex.toString()}`);
      expect(result).toEqual(siblingPath);
    });
  });

  describe('getL1ToL2MessageAndIndex', () => {
    it('should fetch and return the L1 to L2 message and index for the given message key', async () => {
      const messageKey = new Fr(789);
      const expectedMessage = L1ToL2Message.random();
      const expectedIndex = BigInt(123);
      const response = { message: expectedMessage.toBuffer().toString('hex'), index: expectedIndex.toString() };
      setFetchMock(response);

      const result = await httpNode.getL1ToL2MessageAndIndex(messageKey);

      expect(fetch).toHaveBeenCalledWith(`${TEST_URL}l1-l2-message?messageKey=${messageKey.toString()}`);
      expect(result).toEqual({
        message: expectedMessage,
        index: expectedIndex,
      });
    });
  });

  describe('getL1ToL2MessagesTreePath', () => {
    it('should fetch and return the sibling path for the given leaf index', async () => {
      const leafIndex = BigInt(123456);
      const siblingPath = SiblingPath.ZERO(3, Buffer.alloc(32), pedersen);
      const response = { path: siblingPath.toBuffer().toString('hex') };
      setFetchMock(response);

      const result = await httpNode.getL1ToL2MessagesTreePath(leafIndex);

      const url = `${TEST_URL}l1-l2-path?leaf=${leafIndex.toString()}`;
      expect(fetch).toHaveBeenCalledWith(url);
      expect(result).toEqual(siblingPath);
    });
  });

  describe('getPublicStorageAt', () => {
    it('should fetch and return the storage value at the given contract slot', async () => {
      const contractAddress = AztecAddress.random();
      const slot = BigInt(789);
      const storageValue = Buffer.from('0123456789', 'hex');
      const response = { value: storageValue.toString('hex') };
      setFetchMock(response);

      const result = await httpNode.getPublicStorageAt(contractAddress, slot);

      const url = `${TEST_URL}public-storage-at?address=${contractAddress}&slot=${slot.toString()}`;
      expect(fetch).toHaveBeenCalledWith(url);
      expect(result).toEqual(storageValue);
    });

    it('should return undefined if the storage value is not found', async () => {
      const contractAddress = AztecAddress.random();
      const slot = BigInt(987);
      const response = {};
      setFetchMock(response);

      const result = await httpNode.getPublicStorageAt(contractAddress, slot);

      const url = `${TEST_URL}public-storage-at?address=${contractAddress}&slot=${slot.toString()}`;
      expect(fetch).toHaveBeenCalledWith(url);
      expect(result).toBeUndefined();
    });
  });

  describe('getTreeRoots', () => {
    it('should fetch and return the current committed roots for the data trees', async () => {
      const roots: Record<MerkleTreeId, Fr> = {
        [MerkleTreeId.CONTRACT_TREE]: Fr.random(),
        [MerkleTreeId.PRIVATE_DATA_TREE]: Fr.random(),
        [MerkleTreeId.NULLIFIER_TREE]: Fr.random(),
        [MerkleTreeId.PUBLIC_DATA_TREE]: Fr.random(),
        [MerkleTreeId.L1_TO_L2_MESSAGES_TREE]: Fr.random(),
        [MerkleTreeId.BLOCKS_TREE]: Fr.random(),
      };

      const rootsInResponse: Record<MerkleTreeId, string> = Object.fromEntries(
        Object.entries(roots).map(([key, value]) => [key, value.toString()]),
      ) as Record<MerkleTreeId, string>;
      const response = { roots: rootsInResponse };

      setFetchMock(response);

      const result = await httpNode.getTreeRoots();

      const url = `${TEST_URL}tree-roots`;
      expect(fetch).toHaveBeenCalledWith(url);
      expect(result).toEqual(roots);
    });
  });
});
