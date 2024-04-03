import { type Wallet } from '@aztec/aztec.js';
import { DocsExampleContract } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import { setup } from './fixtures/utils.js';

const TIMEOUT = 100_000;

describe('e2e_state_vars', () => {
  jest.setTimeout(TIMEOUT);

  let wallet: Wallet;

  let teardown: () => Promise<void>;
  let contract: DocsExampleContract;

  const POINTS = 1n;
  const RANDOMNESS = 2n;

  beforeAll(async () => {
    ({ teardown, wallet } = await setup());
    contract = await DocsExampleContract.deploy(wallet).send().deployed();
  }, 30_000);

  afterAll(() => teardown());

  describe('SharedImmutable', () => {
    it('private read of uninitialized SharedImmutable', async () => {
      const s = await contract.methods.get_shared_immutable().simulate();

      // Send the transaction and wait for it to be mined (wait function throws if the tx is not mined)
      await contract.methods.match_shared_immutable(s.account, s.points).send().wait();
    });

    it('private read of SharedImmutable', async () => {
      // Initializes the shared immutable and then reads the value using an unconstrained function
      // checking the return values with:
      // 1. A constrained private function that reads it directly
      // 2. A constrained private function that calls another private function that reads.

      await contract.methods.initialize_shared_immutable(1).send().wait();

      const a = await contract.methods.get_shared_immutable_constrained_private().simulate();
      const b = await contract.methods.get_shared_immutable_constrained_private_indirect().simulate();
      const c = await contract.methods.get_shared_immutable().simulate();

      expect((a as any)[0]).toEqual((c as any)['account'].toBigInt());
      expect((a as any)[1]).toEqual((c as any)['points']);
      expect((b as any)[0]).toEqual((c as any)['account'].toBigInt());
      expect((b as any)[1]).toEqual((c as any)['points']);

      expect(a).toEqual(b);
      await contract.methods.match_shared_immutable(c.account, c.points).send().wait();
    });

    it('public read of SharedImmutable', async () => {
      // Reads the value using an unconstrained function checking the return values with:
      // 1. A constrained public function that reads it directly
      // 2. A constrained public function that calls another public function that reads.

      const a = await contract.methods.get_shared_immutable_constrained_public().simulate();
      const b = await contract.methods.get_shared_immutable_constrained_public_indirect().simulate();
      const c = await contract.methods.get_shared_immutable().simulate();

      expect((a as any)[0]).toEqual((c as any)['account'].toBigInt());
      expect((a as any)[1]).toEqual((c as any)['points']);
      expect((b as any)[0]).toEqual((c as any)['account'].toBigInt());
      expect((b as any)[1]).toEqual((c as any)['points']);

      expect(a).toEqual(b);
      await contract.methods.match_shared_immutable(c.account, c.points).send().wait();
    });

    it('initializing SharedImmutable the second time should fail', async () => {
      // Jest executes the tests sequentially and the first call to initialize_shared_immutable was executed
      // in the previous test, so the call bellow should fail.
      await expect(contract.methods.initialize_shared_immutable(1).send().wait()).rejects.toThrow(
        "Assertion failed: SharedImmutable already initialized 'fields_read[0] == 0'",
      );
    }, 100_000);
  });

  describe('PublicImmutable', () => {
    it('initialize and read public immutable', async () => {
      const numPoints = 1n;

      await contract.methods.initialize_public_immutable(numPoints).send().wait();
      const p = await contract.methods.get_public_immutable().simulate();

      expect(p.account).toEqual(wallet.getCompleteAddress().address);
      expect(p.points).toEqual(numPoints);
    }, 200_000);

    it('initializing PublicImmutable the second time should fail', async () => {
      // Jest executes the tests sequentially and the first call to initialize_public_immutable was executed
      // in the previous test, so the call bellow should fail.
      await expect(contract.methods.initialize_public_immutable(1).send().wait()).rejects.toThrow(
        "Assertion failed: PublicImmutable already initialized 'fields_read[0] == 0'",
      );
    }, 100_000);
  });

  describe('PrivateMutable', () => {
    it('fail to read uninitialized PrivateMutable', async () => {
      expect(await contract.methods.is_legendary_initialized().simulate()).toEqual(false);
      await expect(contract.methods.get_legendary_card().simulate()).rejects.toThrow();
    });

    it('initialize PrivateMutable', async () => {
      expect(await contract.methods.is_legendary_initialized().simulate()).toEqual(false);
      // Send the transaction and wait for it to be mined (wait function throws if the tx is not mined)
      const { debugInfo } = await contract.methods.initialize_private(RANDOMNESS, POINTS).send().wait({ debug: true });

      // 1 for the tx, another for the initializer
      expect(debugInfo!.nullifiers.length).toEqual(2);
      expect(await contract.methods.is_legendary_initialized().simulate()).toEqual(true);
    });

    it('fail to reinitialize', async () => {
      expect(await contract.methods.is_legendary_initialized().simulate()).toEqual(true);
      await expect(contract.methods.initialize_private(RANDOMNESS, POINTS).send().wait()).rejects.toThrow();
      expect(await contract.methods.is_legendary_initialized().simulate()).toEqual(true);
    });

    it('read initialized PrivateMutable', async () => {
      expect(await contract.methods.is_legendary_initialized().simulate()).toEqual(true);
      const { points, randomness } = await contract.methods.get_legendary_card().simulate();
      expect(points).toEqual(POINTS);
      expect(randomness).toEqual(RANDOMNESS);
    });

    it('replace with same value', async () => {
      expect(await contract.methods.is_legendary_initialized().simulate()).toEqual(true);
      const noteBefore = await contract.methods.get_legendary_card().simulate();
      const { debugInfo } = await contract.methods
        .update_legendary_card(RANDOMNESS, POINTS)
        .send()
        .wait({ debug: true });

      expect(debugInfo!.noteHashes.length).toEqual(1);
      // 1 for the tx, another for the nullifier of the previous note
      expect(debugInfo!.nullifiers.length).toEqual(2);

      const noteAfter = await contract.methods.get_legendary_card().simulate();

      expect(noteBefore.owner).toEqual(noteAfter.owner);
      expect(noteBefore.points).toEqual(noteAfter.points);
      expect(noteBefore.randomness).toEqual(noteAfter.randomness);
      expect(noteBefore.header.contract_address).toEqual(noteAfter.header.contract_address);
      expect(noteBefore.header.storage_slot).toEqual(noteAfter.header.storage_slot);
      expect(noteBefore.header.is_transient).toEqual(noteAfter.header.is_transient);
      // !!! Nonce must be different
      expect(noteBefore.header.nonce).not.toEqual(noteAfter.header.nonce);
    });

    it('replace PrivateMutable with other values', async () => {
      expect(await contract.methods.is_legendary_initialized().simulate()).toEqual(true);
      const { debugInfo } = await contract.methods
        .update_legendary_card(RANDOMNESS + 2n, POINTS + 1n)
        .send()
        .wait({ debug: true });

      expect(debugInfo!.noteHashes.length).toEqual(1);
      // 1 for the tx, another for the nullifier of the previous note
      expect(debugInfo!.nullifiers.length).toEqual(2);

      const { points, randomness } = await contract.methods.get_legendary_card().simulate();
      expect(points).toEqual(POINTS + 1n);
      expect(randomness).toEqual(RANDOMNESS + 2n);
    });

    it('replace PrivateMutable dependent on prior value', async () => {
      expect(await contract.methods.is_legendary_initialized().simulate()).toEqual(true);
      const noteBefore = await contract.methods.get_legendary_card().simulate();
      const { debugInfo } = await contract.methods.increase_legendary_points().send().wait({ debug: true });

      expect(debugInfo!.noteHashes.length).toEqual(1);
      // 1 for the tx, another for the nullifier of the previous note
      expect(debugInfo!.nullifiers.length).toEqual(2);

      const { points, randomness } = await contract.methods.get_legendary_card().simulate();
      expect(points).toEqual(noteBefore.points + 1n);
      expect(randomness).toEqual(noteBefore.randomness);
    });
  });

  describe('PrivateImmutable', () => {
    it('fail to read uninitialized PrivateImmutable', async () => {
      expect(await contract.methods.is_priv_imm_initialized().simulate()).toEqual(false);
      await expect(contract.methods.view_imm_card().simulate()).rejects.toThrow();
    });

    it('initialize PrivateImmutable', async () => {
      expect(await contract.methods.is_priv_imm_initialized().simulate()).toEqual(false);
      const { debugInfo } = await contract.methods
        .initialize_private_immutable(RANDOMNESS, POINTS)
        .send()
        .wait({ debug: true });

      expect(debugInfo!.noteHashes.length).toEqual(1);
      // 1 for the tx, another for the initializer
      expect(debugInfo!.nullifiers.length).toEqual(2);
      expect(await contract.methods.is_priv_imm_initialized().simulate()).toEqual(true);
    });

    it('fail to reinitialize', async () => {
      expect(await contract.methods.is_priv_imm_initialized().simulate()).toEqual(true);
      await expect(contract.methods.initialize_private_immutable(RANDOMNESS, POINTS).send().wait()).rejects.toThrow();
      expect(await contract.methods.is_priv_imm_initialized().simulate()).toEqual(true);
    });

    it('read initialized PrivateImmutable', async () => {
      expect(await contract.methods.is_priv_imm_initialized().simulate()).toEqual(true);
      const { points, randomness } = await contract.methods.view_imm_card().simulate();
      expect(points).toEqual(POINTS);
      expect(randomness).toEqual(RANDOMNESS);
    });
  });
});
