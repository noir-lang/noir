import { PROVING_STATUS, makeEmptyProcessedTx, mockTx } from '@aztec/circuit-types';
import { Fr, Header } from '@aztec/circuits.js';
import { times } from '@aztec/foundation/collection';
import { createDebugLogger } from '@aztec/foundation/log';

import { type MemDown, default as memdown } from 'memdown';

import { TestContext } from '../mocks/test_context.js';
import { BBNativeRollupProver } from './bb_prover.js';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;

const logger = createDebugLogger('aztec:bb-prover-full-rollup');

describe('prover/bb_prover/full-rollup', () => {
  let context: TestContext;

  beforeAll(async () => {
    context = await TestContext.new(logger, BBNativeRollupProver.new);
  }, 60_000);

  afterAll(async () => {
    await context.cleanup();
  }, 5000);

  it('proves all circuits', async () => {
    const numTransactions = 4;
    const txs = times(numTransactions, (i: number) =>
      mockTx(1000 * (i + 1), {
        numberOfNonRevertiblePublicCallRequests: 2,
        numberOfRevertiblePublicCallRequests: 1,
      }),
    );
    for (const tx of txs) {
      tx.data.constants.historicalHeader = await context.actualDb.buildInitialHeader();
    }

    const provingTicket = await context.orchestrator.startNewBlock(
      numTransactions,
      context.globalVariables,
      [],
      makeEmptyProcessedTx(Header.empty(), new Fr(1234), new Fr(1)),
    );

    const [processed, failed] = await context.processPublicFunctions(txs, numTransactions, context.orchestrator);

    expect(processed.length).toBe(numTransactions);
    expect(failed.length).toBe(0);

    await context.orchestrator.setBlockCompleted();

    const provingResult = await provingTicket.provingPromise;

    expect(provingResult.status).toBe(PROVING_STATUS.SUCCESS);

    const blockResult = await context.orchestrator.finaliseBlock();

    await expect(context.prover.verifyProof('RootRollupArtifact', blockResult.proof)).resolves.not.toThrow();

    await context.orchestrator.stop();
  }, 600_000);
});
