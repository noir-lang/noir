import { FullProverTest } from './e2e_prover_test.js';

const TIMEOUT = 1_800_000;

describe('full_prover_with_padding_tx', () => {
  const t = new FullProverTest('full_prover_with_padding_tx', 1);
  let { provenAssets, accounts, tokenSim, logger } = t;

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.applyMintSnapshot();
    await t.setup();
    await t.deployVerifier();
    ({ provenAssets, accounts, tokenSim, logger } = t);
  });

  afterAll(async () => {
    await t.teardown();
  });

  afterEach(async () => {
    await t.tokenSim.check();
  });

  it(
    'makes a private transfers and pads the block with a padding tx',
    async () => {
      logger.info(
        `Starting test using function: ${provenAssets[0].address}:${provenAssets[0].methods.balance_of_private.selector}`,
      );
      const privateBalance = await provenAssets[0].methods.balance_of_private(accounts[0].address).simulate();
      const privateSendAmount = privateBalance / 2n;
      expect(privateSendAmount).toBeGreaterThan(0n);
      const privateInteraction = provenAssets[0].methods.transfer(accounts[1].address, privateSendAmount);

      const privateTx = await privateInteraction.prove();

      // This will recursively verify all app and kernel circuits involved in the private stage of this transaction!
      logger.info(`Verifying private kernel tail proof`);
      await expect(t.circuitProofVerifier?.verifyProof(privateTx)).resolves.not.toThrow();

      const sentPrivateTx = privateInteraction.send();
      await sentPrivateTx.wait({ timeout: 1200, interval: 10 });
      tokenSim.transferPrivate(accounts[0].address, accounts[1].address, privateSendAmount);
    },
    TIMEOUT,
  );
});
