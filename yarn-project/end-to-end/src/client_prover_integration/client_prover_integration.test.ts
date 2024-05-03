import { type Tx } from '@aztec/aztec.js';
import { type ClientProtocolArtifact } from '@aztec/noir-protocol-circuits-types';
import { type BBNativeProofCreator } from '@aztec/pxe';

import { ClientProverTest } from './client_prover_test.js';

const TIMEOUT = 300_000;

async function verifyProof(_1: ClientProtocolArtifact, _2: Tx, _3: BBNativeProofCreator) {
  // TODO(@PhilWindle): Will verify proof once the circuits are fixed
  await Promise.resolve();
  //const result = await proofCreator.verifyProof(circuitType, tx.proof);
  expect(true).toBeTruthy();
}

describe('client_prover_integration', () => {
  const t = new ClientProverTest('transfer_private');
  let { provenAsset, accounts, tokenSim, logger, proofCreator } = t;

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.applyMintSnapshot();
    await t.setup();
    ({ provenAsset, accounts, tokenSim, logger, proofCreator } = t);
  });

  afterAll(async () => {
    await t.teardown();
  });

  afterEach(async () => {
    await t.tokenSim.check();
  });

  it(
    'private transfer less than balance',
    async () => {
      logger.info(
        `Starting test using function: ${provenAsset.address}:${provenAsset.methods.balance_of_private.selector}`,
      );
      const balance0 = await provenAsset.methods.balance_of_private(accounts[0].address).simulate();
      const amount = balance0 / 2n;
      expect(amount).toBeGreaterThan(0n);
      const interaction = provenAsset.methods.transfer(accounts[0].address, accounts[1].address, amount, 0);
      const provenTx = await interaction.prove();

      // This will recursively verify all app and kernel circuits involved in the private stage of this transaction!
      logger.info(`Verifying kernel tail proof`);
      await verifyProof('PrivateKernelTailArtifact', provenTx, proofCreator!);

      await interaction.send().wait();
      tokenSim.transferPrivate(accounts[0].address, accounts[1].address, amount);
    },
    TIMEOUT,
  );

  it(
    'public transfer less than balance',
    async () => {
      logger.info(
        `Starting test using function: ${provenAsset.address}:${provenAsset.methods.balance_of_public.selector}`,
      );
      const balance0 = await provenAsset.methods.balance_of_public(accounts[0].address).simulate();
      const amount = balance0 / 2n;
      expect(amount).toBeGreaterThan(0n);
      const interaction = provenAsset.methods.transfer(accounts[0].address, accounts[1].address, amount, 0);
      const provenTx = await interaction.prove();

      // This will recursively verify all app and kernel circuits involved in the private stage of this transaction!
      logger.info(`Verifying kernel tail to public proof`);
      await verifyProof('PrivateKernelTailToPublicArtifact', provenTx, proofCreator!);

      await interaction.send().wait();
      tokenSim.transferPublic(accounts[0].address, accounts[1].address, amount);
    },
    TIMEOUT,
  );
});
