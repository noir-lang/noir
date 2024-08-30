import { TestCircuitVerifier } from '@aztec/bb-prover';
import {
  type AztecNode,
  type L1ToL2MessageSource,
  type L2BlockSource,
  type L2LogsSource,
  MerkleTreeId,
  type MerkleTreeOperations,
  mockTxForRollup,
} from '@aztec/circuit-types';
import { AztecAddress, EthAddress, Fr, GasFees, GlobalVariables, MaxBlockNumber } from '@aztec/circuits.js';
import { type AztecLmdbStore } from '@aztec/kv-store/lmdb';
import { type P2P } from '@aztec/p2p';
import { type GlobalVariableBuilder } from '@aztec/sequencer-client';
import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';
import { type ContractDataSource } from '@aztec/types/contracts';
import { type WorldStateSynchronizer } from '@aztec/world-state';

import { type MockProxy, mock, mockFn } from 'jest-mock-extended';

import { type AztecNodeConfig, getConfigEnvVars } from './config.js';
import { AztecNodeService } from './server.js';

describe('aztec node', () => {
  let p2p: MockProxy<P2P>;
  let globalVariablesBuilder: MockProxy<GlobalVariableBuilder>;
  let merkleTreeOps: MockProxy<MerkleTreeOperations>;

  let lastBlockNumber: number;

  let node: AztecNode;

  const chainId = new Fr(12345);
  const version = Fr.ZERO;
  const coinbase = EthAddress.random();
  const feeRecipient = AztecAddress.random();
  const gasFees = GasFees.empty();

  beforeEach(() => {
    lastBlockNumber = 0;

    p2p = mock<P2P>();

    globalVariablesBuilder = mock<GlobalVariableBuilder>();
    merkleTreeOps = mock<MerkleTreeOperations>();

    const worldState = mock<WorldStateSynchronizer>({
      getLatest: () => merkleTreeOps,
    });

    const l2BlockSource = mock<L2BlockSource>({
      getBlockNumber: mockFn().mockResolvedValue(lastBlockNumber),
    });

    const l2LogsSource = mock<L2LogsSource>();

    const l1ToL2MessageSource = mock<L1ToL2MessageSource>();

    // all txs use the same allowed FPC class
    const contractSource = mock<ContractDataSource>();

    const store = mock<AztecLmdbStore>();

    const aztecNodeConfig: AztecNodeConfig = getConfigEnvVars();

    node = new AztecNodeService(
      {
        ...aztecNodeConfig,
        l1Contracts: {
          ...aztecNodeConfig.l1Contracts,
          rollupAddress: EthAddress.ZERO,
          registryAddress: EthAddress.ZERO,
          inboxAddress: EthAddress.ZERO,
          outboxAddress: EthAddress.ZERO,
          availabilityOracleAddress: EthAddress.ZERO,
        },
      },
      p2p,
      l2BlockSource,
      l2LogsSource,
      l2LogsSource,
      contractSource,
      l1ToL2MessageSource,
      worldState,
      undefined,
      31337,
      1,
      globalVariablesBuilder,
      store,
      new TestCircuitVerifier(),
      new NoopTelemetryClient(),
    );
  });

  describe('tx validation', () => {
    it('tests that the node correctly validates double spends', async () => {
      const txs = [mockTxForRollup(0x10000), mockTxForRollup(0x20000)];
      txs.forEach(tx => {
        tx.data.constants.txContext.chainId = chainId;
      });
      const doubleSpendTx = txs[0];
      const doubleSpendWithExistingTx = txs[1];

      const mockedGlobalVariables = new GlobalVariables(
        chainId,
        version,
        new Fr(lastBlockNumber + 1),
        new Fr(1),
        Fr.ZERO,
        coinbase,
        feeRecipient,
        gasFees,
      );

      globalVariablesBuilder.buildGlobalVariables
        .mockResolvedValueOnce(mockedGlobalVariables)
        .mockResolvedValueOnce(mockedGlobalVariables);

      expect(await node.isValidTx(doubleSpendTx)).toBe(true);

      // We push a duplicate nullifier that was created in the same transaction
      doubleSpendTx.data.forRollup!.end.nullifiers.push(doubleSpendTx.data.forRollup!.end.nullifiers[0]);

      expect(await node.isValidTx(doubleSpendTx)).toBe(false);

      globalVariablesBuilder.buildGlobalVariables
        .mockResolvedValueOnce(mockedGlobalVariables)
        .mockResolvedValueOnce(mockedGlobalVariables);

      expect(await node.isValidTx(doubleSpendWithExistingTx)).toBe(true);

      // We make a nullifier from `doubleSpendWithExistingTx` a part of the nullifier tree, so it gets rejected as double spend
      const doubleSpendNullifier = doubleSpendWithExistingTx.data.forRollup!.end.nullifiers[0].toBuffer();
      merkleTreeOps.findLeafIndex.mockImplementation((treeId: MerkleTreeId, value: any) => {
        return Promise.resolve(
          treeId === MerkleTreeId.NULLIFIER_TREE && value.equals(doubleSpendNullifier) ? 1n : undefined,
        );
      });

      expect(await node.isValidTx(doubleSpendWithExistingTx)).toBe(false);
    });

    it('tests that the node correctly validates chain id', async () => {
      const tx = mockTxForRollup(0x10000);
      tx.data.constants.txContext.chainId = chainId;

      const mockedGlobalVariables = new GlobalVariables(
        chainId,
        version,
        new Fr(lastBlockNumber + 1),
        new Fr(1),
        Fr.ZERO,
        coinbase,
        feeRecipient,
        gasFees,
      );

      globalVariablesBuilder.buildGlobalVariables
        .mockResolvedValueOnce(mockedGlobalVariables)
        .mockResolvedValueOnce(mockedGlobalVariables);

      expect(await node.isValidTx(tx)).toBe(true);

      // We make the chain id on the tx not equal to the configured chain id
      tx.data.constants.txContext.chainId = new Fr(1n + chainId.value);

      expect(await node.isValidTx(tx)).toBe(false);
    });

    it('tests that the node correctly validates max block numbers', async () => {
      const txs = [mockTxForRollup(0x10000), mockTxForRollup(0x20000), mockTxForRollup(0x30000)];
      txs.forEach(tx => {
        tx.data.constants.txContext.chainId = chainId;
      });

      const noMaxBlockNumberMetadata = txs[0];
      const invalidMaxBlockNumberMetadata = txs[1];
      const validMaxBlockNumberMetadata = txs[2];

      invalidMaxBlockNumberMetadata.data.forRollup!.rollupValidationRequests = {
        maxBlockNumber: new MaxBlockNumber(true, new Fr(1)),
        getSize: () => 1,
        toBuffer: () => Fr.ZERO.toBuffer(),
      };

      validMaxBlockNumberMetadata.data.forRollup!.rollupValidationRequests = {
        maxBlockNumber: new MaxBlockNumber(true, new Fr(5)),
        getSize: () => 1,
        toBuffer: () => Fr.ZERO.toBuffer(),
      };

      const mockedGlobalVariables = new GlobalVariables(
        chainId,
        version,
        new Fr(lastBlockNumber + 5),
        new Fr(1),
        Fr.ZERO,
        coinbase,
        feeRecipient,
        gasFees,
      );

      globalVariablesBuilder.buildGlobalVariables
        .mockResolvedValueOnce(mockedGlobalVariables)
        .mockResolvedValueOnce(mockedGlobalVariables)
        .mockResolvedValueOnce(mockedGlobalVariables);

      // Default tx with no max block number should be valid
      expect(await node.isValidTx(noMaxBlockNumberMetadata)).toBe(true);
      // Tx with max block number < current block number should be invalid
      expect(await node.isValidTx(invalidMaxBlockNumberMetadata)).toBe(false);
      // Tx with max block number >= current block number should be valid
      expect(await node.isValidTx(validMaxBlockNumberMetadata)).toBe(true);
    });
  });
});
