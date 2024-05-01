import { PROVING_STATUS, makeEmptyProcessedTx, mockTx } from '@aztec/circuit-types';
import { Fr, Header, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP } from '@aztec/circuits.js';
import { makeTuple } from '@aztec/foundation/array';
import { times } from '@aztec/foundation/collection';
import { createDebugLogger } from '@aztec/foundation/log';

import { TestContext } from '../mocks/test_context.js';
import { BBNativeRollupProver } from './bb_prover.js';

const logger = createDebugLogger('aztec:bb-prover-full-rollup');

describe('prover/bb_prover/full-rollup', () => {
  let context: TestContext;

  beforeAll(async () => {
    context = await TestContext.new(logger, 1, BBNativeRollupProver.new);
  });

  afterAll(async () => {
    await context.cleanup();
  });

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

    const l1ToL2Messages = makeTuple<Fr, typeof NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP>(
      NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
      Fr.random,
    );

    const provingTicket = await context.orchestrator.startNewBlock(
      numTransactions,
      context.globalVariables,
      l1ToL2Messages,
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
  });
});
