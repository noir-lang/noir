import {
  MerkleTreeId,
  PROVING_STATUS,
  type ProcessedTx,
  type ProvingSuccess,
  makeEmptyProcessedTx as makeEmptyProcessedTxFromHistoricalTreeRoots,
  makeProcessedTx,
  mockTx,
} from '@aztec/circuit-types';
import {
  AztecAddress,
  type BaseOrMergeRollupPublicInputs,
  EthAddress,
  Fr,
  GlobalVariables,
  KernelCircuitPublicInputs,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  NULLIFIER_SUBTREE_HEIGHT,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  PUBLIC_DATA_SUBTREE_HEIGHT,
  Proof,
  PublicDataTreeLeaf,
  PublicDataUpdateRequest,
  type RootRollupPublicInputs,
  SideEffect,
  SideEffectLinkedToNoteHash,
  sideEffectCmp,
} from '@aztec/circuits.js';
import {
  fr,
  makeBaseOrMergeRollupPublicInputs,
  makeNewSideEffect,
  makeNewSideEffectLinkedToNoteHash,
  makeParityPublicInputs,
  makeProof,
  makeRootRollupPublicInputs,
} from '@aztec/circuits.js/testing';
import { makeTuple, range } from '@aztec/foundation/array';
import { padArrayEnd, times } from '@aztec/foundation/collection';
import { sleep } from '@aztec/foundation/sleep';
import { openTmpStore } from '@aztec/kv-store/utils';
import { WASMSimulator } from '@aztec/simulator';
import { type MerkleTreeOperations, MerkleTrees } from '@aztec/world-state';

import { type MockProxy, mock } from 'jest-mock-extended';
import { type MemDown, default as memdown } from 'memdown';

import { getVerificationKeys } from '../mocks/verification_keys.js';
import { type RollupProver } from '../prover/index.js';
import { type RollupSimulator } from '../simulator/rollup.js';
import { ProvingOrchestrator } from './orchestrator.js';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;

describe('prover/tx-prover', () => {
  let builder: ProvingOrchestrator;
  let builderDb: MerkleTreeOperations;
  let expectsDb: MerkleTreeOperations;

  let simulator: MockProxy<RollupSimulator>;
  let prover: MockProxy<RollupProver>;

  let blockNumber: number;
  let baseRollupOutputLeft: BaseOrMergeRollupPublicInputs;
  let baseRollupOutputRight: BaseOrMergeRollupPublicInputs;
  let rootRollupOutput: RootRollupPublicInputs;
  let mockL1ToL2Messages: Fr[];

  let globalVariables: GlobalVariables;

  const emptyProof = new Proof(Buffer.alloc(32, 0));

  const chainId = Fr.ZERO;
  const version = Fr.ZERO;
  const coinbase = EthAddress.ZERO;
  const feeRecipient = AztecAddress.ZERO;

  beforeEach(async () => {
    blockNumber = 3;
    globalVariables = new GlobalVariables(chainId, version, new Fr(blockNumber), Fr.ZERO, coinbase, feeRecipient);

    builderDb = await MerkleTrees.new(openTmpStore()).then(t => t.asLatest());
    expectsDb = await MerkleTrees.new(openTmpStore()).then(t => t.asLatest());
    simulator = mock<RollupSimulator>();
    prover = mock<RollupProver>();
    builder = new ProvingOrchestrator(builderDb, new WASMSimulator(), getVerificationKeys(), prover);

    // Create mock l1 to L2 messages
    mockL1ToL2Messages = new Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n));

    // Create mock outputs for simulator
    baseRollupOutputLeft = makeBaseOrMergeRollupPublicInputs(0, globalVariables);
    baseRollupOutputRight = makeBaseOrMergeRollupPublicInputs(0, globalVariables);
    rootRollupOutput = makeRootRollupPublicInputs(0);
    rootRollupOutput.header.globalVariables = globalVariables;

    // Set up mocks
    prover.getBaseParityProof.mockResolvedValue(emptyProof);
    prover.getRootParityProof.mockResolvedValue(emptyProof);
    prover.getBaseRollupProof.mockResolvedValue(emptyProof);
    prover.getMergeRollupProof.mockResolvedValue(emptyProof);
    prover.getRootRollupProof.mockResolvedValue(emptyProof);
    simulator.baseParityCircuit
      .mockResolvedValueOnce(makeParityPublicInputs(1))
      .mockResolvedValue(makeParityPublicInputs(2))
      .mockResolvedValue(makeParityPublicInputs(3))
      .mockResolvedValueOnce(makeParityPublicInputs(4));
    simulator.rootParityCircuit.mockResolvedValueOnce(makeParityPublicInputs(5));
    simulator.baseRollupCircuit
      .mockResolvedValueOnce(baseRollupOutputLeft)
      .mockResolvedValueOnce(baseRollupOutputRight);
    simulator.rootRollupCircuit.mockResolvedValue(rootRollupOutput);
  }, 20_000);

  const makeEmptyProcessedTx = async () => {
    const header = await builderDb.buildInitialHeader();
    return makeEmptyProcessedTxFromHistoricalTreeRoots(header, chainId, version);
  };

  // Updates the expectedDb trees based on the new note hashes, contracts, and nullifiers from these txs
  const updateExpectedTreesFromTxs = async (txs: ProcessedTx[]) => {
    await expectsDb.appendLeaves(
      MerkleTreeId.NOTE_HASH_TREE,
      txs.flatMap(tx =>
        padArrayEnd(
          tx.data.end.newNoteHashes.filter(x => !x.isEmpty()).sort(sideEffectCmp),
          SideEffect.empty(),
          MAX_NEW_NOTE_HASHES_PER_TX,
        ).map(l => l.value),
      ),
    );
    await expectsDb.batchInsert(
      MerkleTreeId.NULLIFIER_TREE,
      txs.flatMap(tx =>
        padArrayEnd(
          tx.data.end.newNullifiers.filter(x => !x.isEmpty()).sort(sideEffectCmp),
          SideEffectLinkedToNoteHash.empty(),
          MAX_NEW_NULLIFIERS_PER_TX,
        ).map(x => x.value.toBuffer()),
      ),
      NULLIFIER_SUBTREE_HEIGHT,
    );
    for (const tx of txs) {
      await expectsDb.batchInsert(
        MerkleTreeId.PUBLIC_DATA_TREE,
        tx.data.end.publicDataUpdateRequests.map(write => {
          return new PublicDataTreeLeaf(write.leafSlot, write.newValue).toBuffer();
        }),
        PUBLIC_DATA_SUBTREE_HEIGHT,
      );
    }
  };

  // const updateL1ToL2MessageTree = async (l1ToL2Messages: Fr[]) => {
  //   const asBuffer = l1ToL2Messages.map(m => m.toBuffer());
  //   await expectsDb.appendLeaves(MerkleTreeId.L1_TO_L2_MESSAGE_TREE, asBuffer);
  // };

  // const updateArchive = async () => {
  //   const blockHash = rootRollupOutput.header.hash();
  //   await expectsDb.appendLeaves(MerkleTreeId.ARCHIVE, [blockHash.toBuffer()]);
  // };

  // const getTreeSnapshot = async (tree: MerkleTreeId) => {
  //   const treeInfo = await expectsDb.getTreeInfo(tree);
  //   return new AppendOnlyTreeSnapshot(Fr.fromBuffer(treeInfo.root), Number(treeInfo.size));
  // };

  // const getPartialStateReference = async () => {
  //   return new PartialStateReference(
  //     await getTreeSnapshot(MerkleTreeId.NOTE_HASH_TREE),
  //     await getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE),
  //     await getTreeSnapshot(MerkleTreeId.PUBLIC_DATA_TREE),
  //   );
  // };

  // const getStateReference = async () => {
  //   return new StateReference(
  //     await getTreeSnapshot(MerkleTreeId.L1_TO_L2_MESSAGE_TREE),
  //     await getPartialStateReference(),
  //   );
  // };

  // const buildMockSimulatorInputs = async () => {
  //   const kernelOutput = makePrivateKernelTailCircuitPublicInputs();
  //   kernelOutput.constants.historicalHeader = await expectsDb.buildInitialHeader();
  //   kernelOutput.needsAppLogic = false;
  //   kernelOutput.needsSetup = false;
  //   kernelOutput.needsTeardown = false;

  //   const tx = makeProcessedTx(
  //     new Tx(
  //       kernelOutput,
  //       emptyProof,
  //       makeEmptyLogs(),
  //       makeEmptyLogs(),
  //       times(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, makePublicCallRequest),
  //     ),
  //   );

  //   const txs = [tx, await makeEmptyProcessedTx()];

  //   // Calculate what would be the tree roots after the first tx and update mock circuit output
  //   await updateExpectedTreesFromTxs([txs[0]]);
  //   baseRollupOutputLeft.end = await getPartialStateReference();
  //   baseRollupOutputLeft.txsEffectsHash = to2Fields(toTxEffect(tx).hash());

  //   // Same for the tx on the right
  //   await updateExpectedTreesFromTxs([txs[1]]);
  //   baseRollupOutputRight.end = await getPartialStateReference();
  //   baseRollupOutputRight.txsEffectsHash = to2Fields(toTxEffect(tx).hash());

  //   // Update l1 to l2 message tree
  //   await updateL1ToL2MessageTree(mockL1ToL2Messages);

  //   // Collect all new nullifiers, commitments, and contracts from all txs in this block
  //   const txEffects: TxEffect[] = txs.map(tx => toTxEffect(tx));

  //   const body = new Body(padArrayEnd(mockL1ToL2Messages, Fr.ZERO, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP), txEffects);
  //   // We are constructing the block here just to get body hash/calldata hash so we can pass in an empty archive and header
  //   const l2Block = L2Block.fromFields({
  //     archive: AppendOnlyTreeSnapshot.zero(),
  //     header: Header.empty(),
  //     // Only the values below go to body hash/calldata hash
  //     body,
  //   });

  //   // Now we update can make the final header, compute the block hash and update archive
  //   rootRollupOutput.header.globalVariables = globalVariables;
  //   rootRollupOutput.header.contentCommitment.txsEffectsHash = l2Block.body.getTxsEffectsHash();
  //   rootRollupOutput.header.state = await getStateReference();

  //   await updateArchive();
  //   rootRollupOutput.archive = await getTreeSnapshot(MerkleTreeId.ARCHIVE);

  //   return txs;
  // };

  describe('error handling', () => {
    beforeEach(async () => {
      builder = await ProvingOrchestrator.new(builderDb, new WASMSimulator(), prover);
    });

    it.each([
      [
        'Base Rollup Failed',
        () => {
          prover.getBaseRollupProof.mockRejectedValue('Base Rollup Failed');
        },
      ],
      [
        'Merge Rollup Failed',
        () => {
          prover.getMergeRollupProof.mockRejectedValue('Merge Rollup Failed');
        },
      ],
      [
        'Root Rollup Failed',
        () => {
          prover.getRootRollupProof.mockRejectedValue('Root Rollup Failed');
        },
      ],
      [
        'Base Parity Failed',
        () => {
          prover.getBaseParityProof.mockRejectedValue('Base Parity Failed');
        },
      ],
      [
        'Root Parity Failed',
        () => {
          prover.getRootParityProof.mockRejectedValue('Root Parity Failed');
        },
      ],
    ] as const)(
      'handles a %s error',
      async (message: string, fn: () => void) => {
        fn();
        const txs = await Promise.all([
          makeEmptyProcessedTx(),
          makeEmptyProcessedTx(),
          makeEmptyProcessedTx(),
          makeEmptyProcessedTx(),
        ]);

        const blockTicket = await builder.startNewBlock(txs.length, globalVariables, [], await makeEmptyProcessedTx());

        for (const tx of txs) {
          await builder.addNewTx(tx);
        }
        await expect(blockTicket.provingPromise).resolves.toEqual({ status: PROVING_STATUS.FAILURE, reason: message });
      },
      60000,
    );

    afterEach(async () => {
      await builder.stop();
    });
  });

  describe('circuits simulator', () => {
    beforeEach(async () => {
      builder = await ProvingOrchestrator.new(builderDb, new WASMSimulator(), prover);
    });

    afterEach(async () => {
      await builder.stop();
    });

    const makeBloatedProcessedTx = async (seed = 0x1) => {
      seed *= MAX_NEW_NULLIFIERS_PER_TX; // Ensure no clashing given incremental seeds
      const tx = mockTx(seed);
      const kernelOutput = KernelCircuitPublicInputs.empty();
      kernelOutput.constants.historicalHeader = await builderDb.buildInitialHeader();
      kernelOutput.end.publicDataUpdateRequests = makeTuple(
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
        i => new PublicDataUpdateRequest(fr(i), fr(i + 10)),
        seed + 0x500,
      );
      kernelOutput.end.publicDataUpdateRequests = makeTuple(
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
        i => new PublicDataUpdateRequest(fr(i), fr(i + 10)),
        seed + 0x600,
      );

      const processedTx = makeProcessedTx(tx, kernelOutput, makeProof());

      processedTx.data.end.newNoteHashes = makeTuple(MAX_NEW_NOTE_HASHES_PER_TX, makeNewSideEffect, seed + 0x100);
      processedTx.data.end.newNullifiers = makeTuple(
        MAX_NEW_NULLIFIERS_PER_TX,
        makeNewSideEffectLinkedToNoteHash,
        seed + 0x100000,
      );

      processedTx.data.end.newNullifiers[tx.data.forPublic!.end.newNullifiers.length - 1] =
        SideEffectLinkedToNoteHash.empty();

      processedTx.data.end.newL2ToL1Msgs = makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_TX, fr, seed + 0x300);
      processedTx.data.end.encryptedLogsHash = Fr.fromBuffer(processedTx.encryptedLogs.hash());
      processedTx.data.end.unencryptedLogsHash = Fr.fromBuffer(processedTx.unencryptedLogs.hash());

      return processedTx;
    };

    it.each([
      [0, 2],
      [1, 2],
      [4, 4],
      [5, 8],
      [9, 16],
    ] as const)(
      'builds an L2 block with %i bloated txs and %i txs total',
      async (bloatedCount: number, totalCount: number) => {
        const noteHashTreeBefore = await builderDb.getTreeInfo(MerkleTreeId.NOTE_HASH_TREE);
        const txs = [
          ...(await Promise.all(times(bloatedCount, makeBloatedProcessedTx))),
          ...(await Promise.all(times(totalCount - bloatedCount, makeEmptyProcessedTx))),
        ];

        const blockTicket = await builder.startNewBlock(
          txs.length,
          globalVariables,
          mockL1ToL2Messages,
          await makeEmptyProcessedTx(),
        );

        for (const tx of txs) {
          await builder.addNewTx(tx);
        }

        const result = await blockTicket.provingPromise;
        expect(result.status).toBe(PROVING_STATUS.SUCCESS);

        expect((result as ProvingSuccess).block.number).toEqual(blockNumber);

        await updateExpectedTreesFromTxs(txs);
        const noteHashTreeAfter = await builderDb.getTreeInfo(MerkleTreeId.NOTE_HASH_TREE);

        if (bloatedCount > 0) {
          expect(noteHashTreeAfter.root).not.toEqual(noteHashTreeBefore.root);
        }

        const expectedNoteHashTreeAfter = await expectsDb.getTreeInfo(MerkleTreeId.NOTE_HASH_TREE).then(t => t.root);
        expect(noteHashTreeAfter.root).toEqual(expectedNoteHashTreeAfter);
      },
      60000,
    );

    it('builds an empty L2 block', async () => {
      const txs = await Promise.all([makeEmptyProcessedTx(), makeEmptyProcessedTx()]);

      const blockTicket = await builder.startNewBlock(txs.length, globalVariables, [], await makeEmptyProcessedTx());

      for (const tx of txs) {
        await builder.addNewTx(tx);
      }

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);
      expect((result as ProvingSuccess).block.number).toEqual(blockNumber);
    }, 30_000);

    it('builds a block with 1 transaction', async () => {
      const txs = await Promise.all([makeEmptyProcessedTx()]);

      const blockTicket = await builder.startNewBlock(txs.length, globalVariables, [], await makeEmptyProcessedTx());

      for (const tx of txs) {
        await builder.addNewTx(tx);
      }

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);
      expect((result as ProvingSuccess).block.number).toEqual(blockNumber);
    }, 30_000);

    it('builds a mixed L2 block', async () => {
      const txs = await Promise.all([
        makeBloatedProcessedTx(1),
        makeBloatedProcessedTx(2),
        makeBloatedProcessedTx(3),
        makeBloatedProcessedTx(4),
      ]);

      const l1ToL2Messages = range(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, 1 + 0x400).map(fr);

      const blockTicket = await builder.startNewBlock(
        txs.length,
        globalVariables,
        l1ToL2Messages,
        await makeEmptyProcessedTx(),
      );

      for (const tx of txs) {
        await builder.addNewTx(tx);
      }

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);
      expect((result as ProvingSuccess).block.number).toEqual(blockNumber);
    }, 200_000);

    it('builds a block concurrently with transactions', async () => {
      const txs = await Promise.all([
        makeBloatedProcessedTx(1),
        makeBloatedProcessedTx(2),
        makeBloatedProcessedTx(3),
        makeBloatedProcessedTx(4),
      ]);

      const l1ToL2Messages = range(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, 1 + 0x400).map(fr);

      const blockTicket = await builder.startNewBlock(
        txs.length,
        globalVariables,
        l1ToL2Messages,
        await makeEmptyProcessedTx(),
      );

      for (const tx of txs) {
        await builder.addNewTx(tx);
        await sleep(1000);
      }

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);
      expect((result as ProvingSuccess).block.number).toEqual(blockNumber);
    }, 200_000);

    // it('cancels current blocks and switches to new ones', async () => {
    //   const txs = await Promise.all([
    //     makeBloatedProcessedTx(1),
    //     makeBloatedProcessedTx(2),
    //     makeBloatedProcessedTx(3),
    //     makeBloatedProcessedTx(4),
    //   ]);

    //   const l1ToL2Messages = range(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, 1 + 0x400).map(fr);

    //   const blockPromise1 = await builder.startNewBlock(
    //     txs.length,
    //     globalVariables,
    //     l1ToL2Messages,
    //     await makeEmptyProcessedTx(),
    //   );

    //   builder.addNewTx(txs[0]);

    //   const blockPromise2 = await builder.startNewBlock(
    //     txs.length,
    //     globalVariables,
    //     l1ToL2Messages,
    //     await makeEmptyProcessedTx(),
    //   );

    //   builder.addNewTx(txs[0]);

    //   await expect(blockPromise1).rejects.toEqual('Block cancelled');

    //   const result = await blockPromise2;
    //   expect(result.block.number).toEqual(blockNumber);
    // }, 200_000);

    it('builds an unbalanced L2 block', async () => {
      const txs = await Promise.all([makeBloatedProcessedTx(1), makeBloatedProcessedTx(2), makeBloatedProcessedTx(3)]);

      const l1ToL2Messages = range(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, 1 + 0x400).map(fr);

      const blockTicket = await builder.startNewBlock(
        txs.length,
        globalVariables,
        l1ToL2Messages,
        await makeEmptyProcessedTx(),
      );

      for (const tx of txs) {
        await builder.addNewTx(tx);
      }

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);
      expect((result as ProvingSuccess).block.number).toEqual(blockNumber);
    }, 200_000);

    it('throws if adding too many transactions', async () => {
      const txs = await Promise.all([
        makeBloatedProcessedTx(1),
        makeBloatedProcessedTx(2),
        makeBloatedProcessedTx(3),
        makeBloatedProcessedTx(4),
      ]);

      const blockTicket = await builder.startNewBlock(txs.length, globalVariables, [], await makeEmptyProcessedTx());

      for (const tx of txs) {
        await builder.addNewTx(tx);
      }

      await expect(async () => await builder.addNewTx(await makeEmptyProcessedTx())).rejects.toThrow(
        `Rollup already contains 4 transactions`,
      );

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);
      expect((result as ProvingSuccess).block.number).toEqual(blockNumber);
    }, 30_000);

    it('throws if adding a transaction before start', async () => {
      await expect(async () => await builder.addNewTx(await makeEmptyProcessedTx())).rejects.toThrow(
        `Invalid proving state, call startNewBlock before adding transactions`,
      );
    }, 30_000);

    it('rejects if too many l1 to l2 messages are provided', async () => {
      // Assemble a fake transaction
      const l1ToL2Messages = new Array(100).fill(new Fr(0n));
      await expect(
        async () => await builder.startNewBlock(1, globalVariables, l1ToL2Messages, await makeEmptyProcessedTx()),
      ).rejects.toThrow('Too many L1 to L2 messages');
    });
  });
});
