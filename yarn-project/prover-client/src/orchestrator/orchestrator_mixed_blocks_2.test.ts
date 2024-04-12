import { MerkleTreeId, PROVING_STATUS } from '@aztec/circuit-types';
import { type GlobalVariables, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP } from '@aztec/circuits.js';
import { fr } from '@aztec/circuits.js/testing';
import { range } from '@aztec/foundation/array';
import { times } from '@aztec/foundation/collection';
import { createDebugLogger } from '@aztec/foundation/log';
import { openTmpStore } from '@aztec/kv-store/utils';
import { type MerkleTreeOperations, MerkleTrees } from '@aztec/world-state';

import { type MemDown, default as memdown } from 'memdown';

import {
  getConfig,
  getSimulationProvider,
  makeBloatedProcessedTx,
  makeEmptyProcessedTestTx,
  makeGlobals,
  updateExpectedTreesFromTxs,
} from '../mocks/fixtures.js';
import { TestCircuitProver } from '../prover/test_circuit_prover.js';
import { ProvingOrchestrator } from './orchestrator.js';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;

const logger = createDebugLogger('aztec:orchestrator-test');

describe('prover/orchestrator', () => {
  let builder: ProvingOrchestrator;
  let builderDb: MerkleTreeOperations;
  let expectsDb: MerkleTreeOperations;

  let prover: TestCircuitProver;

  let blockNumber: number;

  let globalVariables: GlobalVariables;

  beforeEach(async () => {
    blockNumber = 3;
    globalVariables = makeGlobals(blockNumber);

    const acvmConfig = await getConfig(logger);
    const simulationProvider = await getSimulationProvider({
      acvmWorkingDirectory: acvmConfig?.acvmWorkingDirectory,
      acvmBinaryPath: acvmConfig?.expectedAcvmPath,
    });
    prover = new TestCircuitProver(simulationProvider);

    builderDb = await MerkleTrees.new(openTmpStore()).then(t => t.asLatest());
    expectsDb = await MerkleTrees.new(openTmpStore()).then(t => t.asLatest());
    builder = new ProvingOrchestrator(builderDb, prover, 1);
  }, 20_000);

  describe('blocks', () => {
    beforeEach(async () => {
      builder = await ProvingOrchestrator.new(builderDb, prover);
    });

    afterEach(async () => {
      await builder.stop();
    });

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
          ...(await Promise.all(times(bloatedCount, (i: number) => makeBloatedProcessedTx(builderDb, i)))),
          ...(await Promise.all(times(totalCount - bloatedCount, _ => makeEmptyProcessedTestTx(builderDb)))),
        ];

        const l1ToL2Messages = range(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, 1 + 0x400).map(fr);

        const blockTicket = await builder.startNewBlock(
          txs.length,
          globalVariables,
          l1ToL2Messages,
          await makeEmptyProcessedTestTx(builderDb),
        );

        for (const tx of txs) {
          await builder.addNewTx(tx);
        }

        const result = await blockTicket.provingPromise;
        expect(result.status).toBe(PROVING_STATUS.SUCCESS);

        const finalisedBlock = await builder.finaliseBlock();

        expect(finalisedBlock.block.number).toEqual(blockNumber);

        await updateExpectedTreesFromTxs(expectsDb, txs);
        const noteHashTreeAfter = await builderDb.getTreeInfo(MerkleTreeId.NOTE_HASH_TREE);

        if (bloatedCount > 0) {
          expect(noteHashTreeAfter.root).not.toEqual(noteHashTreeBefore.root);
        }

        const expectedNoteHashTreeAfter = await expectsDb.getTreeInfo(MerkleTreeId.NOTE_HASH_TREE).then(t => t.root);
        expect(noteHashTreeAfter.root).toEqual(expectedNoteHashTreeAfter);
      },
      60000,
    );
  });
});
