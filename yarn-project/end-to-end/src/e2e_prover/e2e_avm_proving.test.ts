import { FullProverTestAvm } from './e2e_prover_test_avm.js';

const TIMEOUT = 1_800_000;

// This makes AVM proving throw if there's a failure.
process.env.AVM_PROVING_STRICT = '1';

describe('full_prover/avm', () => {
  const t = new FullProverTestAvm('full_prover/avm');
  let { provenAsset: avmContract, logger } = t;

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.setup();
    await t.deployVerifier();
    ({ provenAsset: avmContract, logger } = t);
  });

  afterAll(async () => {
    await t.teardown();
  });

  it(
    'proves simple contract call',
    async () => {
      logger.info(
        `Starting test using function: ${avmContract.address}:${avmContract.methods.add_args_return.selector}`,
      );
      await avmContract.methods.add_args_return(3, 5).send().wait();
    },
    TIMEOUT,
  );
});
