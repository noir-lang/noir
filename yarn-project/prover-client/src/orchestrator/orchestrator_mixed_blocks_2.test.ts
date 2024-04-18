import { MerkleTreeId, PROVING_STATUS } from '@aztec/circuit-types';
import { NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP } from '@aztec/circuits.js';
import { fr } from '@aztec/circuits.js/testing';
import { range } from '@aztec/foundation/array';
import { times } from '@aztec/foundation/collection';
import { createDebugLogger } from '@aztec/foundation/log';
import { openTmpStore } from '@aztec/kv-store/utils';
import { type MerkleTreeOperations, MerkleTrees } from '@aztec/world-state';

import { type MemDown, default as memdown } from 'memdown';

import { makeBloatedProcessedTx, makeEmptyProcessedTestTx, updateExpectedTreesFromTxs } from '../mocks/fixtures.js';
import { TestContext } from '../mocks/test_context.js';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;

const logger = createDebugLogger('aztec:orchestrator-mixed-blocks-2');

describe('prover/orchestrator/mixed-blocks', () => {
  let context: TestContext;
  let expectsDb: MerkleTreeOperations;

  beforeEach(async () => {
    context = await TestContext.new(logger);
    expectsDb = await MerkleTrees.new(openTmpStore()).then(t => t.asLatest());
  }, 20_000);

  afterEach(async () => {
    await context.cleanup();
  });

  describe('blocks', () => {
    it.each([
      [0, 2],
      [1, 2],
      [4, 4],
      [5, 8],
    ] as const)(
      'builds an L2 block with %i bloated txs and %i txs total',
      async (bloatedCount: number, totalCount: number) => {
        const noteHashTreeBefore = await context.actualDb.getTreeInfo(MerkleTreeId.NOTE_HASH_TREE);
        const txs = [
          ...(await Promise.all(times(bloatedCount, (i: number) => makeBloatedProcessedTx(context.actualDb, i)))),
          ...(await Promise.all(times(totalCount - bloatedCount, _ => makeEmptyProcessedTestTx(context.actualDb)))),
        ];

        const l1ToL2Messages = range(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, 1 + 0x400).map(fr);

        const blockTicket = await context.orchestrator.startNewBlock(
          txs.length,
          context.globalVariables,
          l1ToL2Messages,
          await makeEmptyProcessedTestTx(context.actualDb),
        );

        for (const tx of txs) {
          await context.orchestrator.addNewTx(tx);
        }

        const result = await blockTicket.provingPromise;
        expect(result.status).toBe(PROVING_STATUS.SUCCESS);

        const finalisedBlock = await context.orchestrator.finaliseBlock();

        expect(finalisedBlock.block.number).toEqual(context.blockNumber);

        await updateExpectedTreesFromTxs(expectsDb, txs);
        const noteHashTreeAfter = await context.actualDb.getTreeInfo(MerkleTreeId.NOTE_HASH_TREE);

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
