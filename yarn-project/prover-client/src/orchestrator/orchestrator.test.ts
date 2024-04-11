import {
  MerkleTreeId,
  PROVING_STATUS,
  type ProcessedTx,
  type ProvingFailure,
  makeEmptyProcessedTx as makeEmptyProcessedTxFromHistoricalTreeRoots,
  makeProcessedTx,
  mockTx,
} from '@aztec/circuit-types';
import {
  AztecAddress,
  type BaseOrMergeRollupPublicInputs,
  EthAddress,
  Fr,
  GasFees,
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
} from '@aztec/circuits.js';
import {
  fr,
  makeBaseOrMergeRollupPublicInputs,
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

  const makeGlobals = (blockNumber: number) => {
    return new GlobalVariables(chainId, version, new Fr(blockNumber), Fr.ZERO, coinbase, feeRecipient, GasFees.empty());
  };

  beforeEach(async () => {
    blockNumber = 3;
    globalVariables = makeGlobals(blockNumber);

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
          tx.data.end.newNoteHashes.filter(x => !x.isZero()),
          Fr.zero(),
          MAX_NEW_NOTE_HASHES_PER_TX,
        ),
      ),
    );
    await expectsDb.batchInsert(
      MerkleTreeId.NULLIFIER_TREE,
      txs.flatMap(tx =>
        padArrayEnd(
          tx.data.end.newNullifiers.filter(x => !x.isZero()),
          Fr.zero(),
          MAX_NEW_NULLIFIERS_PER_TX,
        ).map(x => x.toBuffer()),
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

      processedTx.data.end.newNoteHashes = makeTuple(MAX_NEW_NOTE_HASHES_PER_TX, fr, seed + 0x100);
      processedTx.data.end.newNullifiers = makeTuple(MAX_NEW_NULLIFIERS_PER_TX, fr, seed + 0x100000);

      processedTx.data.end.newNullifiers[tx.data.forPublic!.end.newNullifiers.length - 1] = Fr.zero();

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

        const finalisedBlock = await builder.finaliseBlock();

        expect(finalisedBlock.block.number).toEqual(blockNumber);

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
      const finalisedBlock = await builder.finaliseBlock();

      expect(finalisedBlock.block.number).toEqual(blockNumber);
    }, 30_000);

    it('builds a block with 1 transaction', async () => {
      const txs = await Promise.all([makeEmptyProcessedTx()]);

      // This will need to be a 2 tx block
      const blockTicket = await builder.startNewBlock(2, globalVariables, [], await makeEmptyProcessedTx());

      for (const tx of txs) {
        await builder.addNewTx(tx);
      }

      //  we need to complete the block as we have not added a full set of txs
      await builder.setBlockCompleted();

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);
      const finalisedBlock = await builder.finaliseBlock();

      expect(finalisedBlock.block.number).toEqual(blockNumber);
    }, 30_000);

    it('builds multiple blocks in sequence', async () => {
      const numBlocks = 5;
      let header = await builderDb.buildInitialHeader();

      for (let i = 0; i < numBlocks; i++) {
        const tx = await makeBloatedProcessedTx(i + 1);
        const emptyTx = await makeEmptyProcessedTx();
        tx.data.constants.historicalHeader = header;
        emptyTx.data.constants.historicalHeader = header;

        const blockNum = i + 1000;

        const globals = makeGlobals(blockNum);

        // This will need to be a 2 tx block
        const blockTicket = await builder.startNewBlock(2, globals, [], emptyTx);

        await builder.addNewTx(tx);

        //  we need to complete the block as we have not added a full set of txs
        await builder.setBlockCompleted();

        const result = await blockTicket.provingPromise;
        expect(result.status).toBe(PROVING_STATUS.SUCCESS);
        const finalisedBlock = await builder.finaliseBlock();

        expect(finalisedBlock.block.number).toEqual(blockNum);
        header = finalisedBlock.block.header;

        await builderDb.commit();
      }
    }, 60_000);

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
      const finalisedBlock = await builder.finaliseBlock();

      expect(finalisedBlock.block.number).toEqual(blockNumber);
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
      const finalisedBlock = await builder.finaliseBlock();

      expect(finalisedBlock.block.number).toEqual(blockNumber);
    }, 200_000);

    it('cancels current block and switches to new ones', async () => {
      const txs1 = await Promise.all([makeBloatedProcessedTx(1), makeBloatedProcessedTx(2)]);

      const txs2 = await Promise.all([makeBloatedProcessedTx(3), makeBloatedProcessedTx(4)]);

      const globals1: GlobalVariables = makeGlobals(100);
      const globals2: GlobalVariables = makeGlobals(101);

      const l1ToL2Messages = range(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, 1 + 0x400).map(fr);

      const blockTicket1 = await builder.startNewBlock(2, globals1, l1ToL2Messages, await makeEmptyProcessedTx());

      await builder.addNewTx(txs1[0]);
      await builder.addNewTx(txs1[1]);

      // Now we cancel the block. The first block will come to a stop as and when current proofs complete
      builder.cancelBlock();

      const result1 = await blockTicket1.provingPromise;

      // in all likelihood, the block will have a failure code as we cancelled it
      // however it may have actually completed proving before we cancelled in which case it could be a succes code
      if (result1.status === PROVING_STATUS.FAILURE) {
        expect((result1 as ProvingFailure).reason).toBe('Proving cancelled');
      }

      await builderDb.rollback();

      const blockTicket2 = await builder.startNewBlock(2, globals2, l1ToL2Messages, await makeEmptyProcessedTx());

      await builder.addNewTx(txs2[0]);
      await builder.addNewTx(txs2[1]);

      const result2 = await blockTicket2.provingPromise;
      expect(result2.status).toBe(PROVING_STATUS.SUCCESS);
      const finalisedBlock = await builder.finaliseBlock();

      expect(finalisedBlock.block.number).toEqual(101);
    }, 10000);

    it('automatically cancels an incomplete block when starting a new one', async () => {
      const txs1 = await Promise.all([makeBloatedProcessedTx(1), makeBloatedProcessedTx(2)]);

      const txs2 = await Promise.all([makeBloatedProcessedTx(3), makeBloatedProcessedTx(4)]);

      const globals1: GlobalVariables = makeGlobals(100);
      const globals2: GlobalVariables = makeGlobals(101);

      const l1ToL2Messages = range(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, 1 + 0x400).map(fr);

      const blockTicket1 = await builder.startNewBlock(2, globals1, l1ToL2Messages, await makeEmptyProcessedTx());

      await builder.addNewTx(txs1[0]);

      await builderDb.rollback();

      const blockTicket2 = await builder.startNewBlock(2, globals2, l1ToL2Messages, await makeEmptyProcessedTx());

      await builder.addNewTx(txs2[0]);
      await builder.addNewTx(txs2[1]);

      const result1 = await blockTicket1.provingPromise;
      expect(result1.status).toBe(PROVING_STATUS.FAILURE);
      expect((result1 as ProvingFailure).reason).toBe('Proving cancelled');

      const result2 = await blockTicket2.provingPromise;
      expect(result2.status).toBe(PROVING_STATUS.SUCCESS);
      const finalisedBlock = await builder.finaliseBlock();

      expect(finalisedBlock.block.number).toEqual(101);
    }, 10000);

    it('builds an unbalanced L2 block', async () => {
      const txs = await Promise.all([makeBloatedProcessedTx(1), makeBloatedProcessedTx(2), makeBloatedProcessedTx(3)]);

      const l1ToL2Messages = range(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, 1 + 0x400).map(fr);

      // this needs to be a 4 tx block that will need to be completed
      const blockTicket = await builder.startNewBlock(4, globalVariables, l1ToL2Messages, await makeEmptyProcessedTx());

      for (const tx of txs) {
        await builder.addNewTx(tx);
      }

      await builder.setBlockCompleted();

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);
      const finalisedBlock = await builder.finaliseBlock();

      expect(finalisedBlock.block.number).toEqual(blockNumber);
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
        'Rollup not accepting further transactions',
      );

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);
      const finalisedBlock = await builder.finaliseBlock();

      expect(finalisedBlock.block.number).toEqual(blockNumber);
    }, 30_000);

    it('throws if adding a transaction before start', async () => {
      await expect(async () => await builder.addNewTx(await makeEmptyProcessedTx())).rejects.toThrow(
        `Invalid proving state, call startNewBlock before adding transactions`,
      );
    }, 1000);

    it('throws if completing a block before start', async () => {
      await expect(async () => await builder.setBlockCompleted()).rejects.toThrow(
        'Invalid proving state, call startNewBlock before adding transactions or completing the block',
      );
    }, 1000);

    it('throws if finalising an incompletre block', async () => {
      await expect(async () => await builder.finaliseBlock()).rejects.toThrow(
        'Invalid proving state, a block must be proven before it can be finalised',
      );
    }, 1000);

    it('throws if finalising an already finalised block', async () => {
      const txs = await Promise.all([makeEmptyProcessedTx(), makeEmptyProcessedTx()]);

      const blockTicket = await builder.startNewBlock(txs.length, globalVariables, [], await makeEmptyProcessedTx());

      for (const tx of txs) {
        await builder.addNewTx(tx);
      }

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);
      const finalisedBlock = await builder.finaliseBlock();
      expect(finalisedBlock.block.number).toEqual(blockNumber);
      await expect(async () => await builder.finaliseBlock()).rejects.toThrow('Block already finalised');
    }, 20000);

    it('throws if adding to a cancelled block', async () => {
      await builder.startNewBlock(2, globalVariables, [], await makeEmptyProcessedTx());

      builder.cancelBlock();

      await expect(async () => await builder.addNewTx(await makeEmptyProcessedTx())).rejects.toThrow(
        'Rollup not accepting further transactions',
      );
    }, 10000);

    it.each([[-4], [0], [1], [3], [8.1], [7]] as const)(
      'fails to start a block with %i transaxctions',
      async (blockSize: number) => {
        await expect(
          async () => await builder.startNewBlock(blockSize, globalVariables, [], await makeEmptyProcessedTx()),
        ).rejects.toThrow(`Length of txs for the block should be a power of two and at least two (got ${blockSize})`);
      },
      10000,
    );

    it('rejects if too many l1 to l2 messages are provided', async () => {
      // Assemble a fake transaction
      const l1ToL2Messages = new Array(100).fill(new Fr(0n));
      await expect(
        async () => await builder.startNewBlock(2, globalVariables, l1ToL2Messages, await makeEmptyProcessedTx()),
      ).rejects.toThrow('Too many L1 to L2 messages');
    });
  });
});
