import { type AccountWallet, AztecAddress, BatchCall, Fr, TxStatus } from '@aztec/aztec.js';
import { GasSettings } from '@aztec/circuits.js';
import { AvmInitializerTestContract, AvmTestContract } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import { publicDeployAccounts, setup } from './fixtures/utils.js';

const TIMEOUT = 100_000;

describe('e2e_avm_simulator', () => {
  jest.setTimeout(TIMEOUT);

  let wallet: AccountWallet;
  let teardown: () => Promise<void>;

  beforeAll(async () => {
    ({ teardown, wallet } = await setup());
    await publicDeployAccounts(wallet, [wallet]);
  });

  afterAll(() => teardown());

  describe('AvmTestContract', () => {
    let avmContract: AvmTestContract;
    let secondAvmContract: AvmTestContract;

    beforeEach(async () => {
      avmContract = await AvmTestContract.deploy(wallet).send().deployed();
      secondAvmContract = await AvmTestContract.deploy(wallet).send().deployed();
    });

    describe('Assertions', () => {
      it('PXE processes failed assertions and fills in the error message with the expression', async () => {
        await expect(avmContract.methods.assertion_failure().simulate()).rejects.toThrow(
          "Assertion failed: This assertion should fail! 'not_true == true'",
        );
      });
      it('PXE processes failed assertions and fills in the error message with the expression (even complex ones)', async () => {
        await expect(avmContract.methods.assert_nullifier_exists(123).simulate()).rejects.toThrow(
          "Assertion failed: Nullifier doesn't exist! 'context.nullifier_exists(nullifier, context.storage_address())'",
        );
      });
    });

    describe('From private', () => {
      it('Should enqueue a public function correctly', async () => {
        await avmContract.methods.enqueue_public_from_private().simulate();
      });
    });

    describe('Gas metering', () => {
      it('Tracks L2 gas usage on simulation', async () => {
        const request = await avmContract.methods.add_args_return(20n, 30n).create();
        const simulation = await wallet.simulateTx(request, true);
        // Subtract the teardown gas allocation from the gas used to figure out the gas used by the contract logic.
        const l2TeardownAllocation = GasSettings.simulation().getTeardownLimits().l2Gas;
        const l2GasUsed = simulation.publicOutput!.end.gasUsed.l2Gas! - l2TeardownAllocation;
        // L2 gas used will vary a lot depending on codegen and other factors,
        // so we just set a wide range for it, and check it's not a suspiciously round number.
        expect(l2GasUsed).toBeGreaterThan(150);
        expect(l2GasUsed).toBeLessThan(1e6);
        expect(l2GasUsed! % 1000).not.toEqual(0);
      });
    });

    describe('Storage', () => {
      it('Modifies storage (Field)', async () => {
        await avmContract.methods.set_storage_single(20n).send().wait();
        expect(await avmContract.methods.read_storage_single().simulate()).toEqual(20n);
      });

      it('Modifies storage (Map)', async () => {
        const address = AztecAddress.fromBigInt(9090n);
        await avmContract.methods.set_storage_map(address, 100).send().wait();
        await avmContract.methods.add_storage_map(address, 100).send().wait();
        expect(await avmContract.methods.read_storage_map(address).simulate()).toEqual(200n);
      });

      it('Preserves storage across enqueued public calls', async () => {
        const address = AztecAddress.fromBigInt(9090n);
        // This will create 1 tx with 2 public calls in it.
        await new BatchCall(wallet, [
          avmContract.methods.set_storage_map(address, 100).request(),
          avmContract.methods.add_storage_map(address, 100).request(),
        ])
          .send()
          .wait();
        // On a separate tx, we check the result.
        expect(await avmContract.methods.read_storage_map(address).simulate()).toEqual(200n);
      });
    });

    describe('Contract instance', () => {
      it('Works', async () => {
        const tx = await avmContract.methods.test_get_contract_instance().send().wait();
        expect(tx.status).toEqual(TxStatus.SUCCESS);
      });
    });

    describe('Nullifiers', () => {
      // Nullifier will not yet be siloed by the kernel.
      it('Emit and check in the same tx', async () => {
        const tx = await avmContract.methods.emit_nullifier_and_check(123456).send().wait();
        expect(tx.status).toEqual(TxStatus.SUCCESS);
      });

      // Nullifier will have been siloed by the kernel, but we check against the unsiloed one.
      it('Emit and check in separate tx', async () => {
        const nullifier = new Fr(123456);
        let tx = await avmContract.methods.new_nullifier(nullifier).send().wait();
        expect(tx.status).toEqual(TxStatus.SUCCESS);

        tx = await avmContract.methods.assert_nullifier_exists(nullifier).send().wait();
        expect(tx.status).toEqual(TxStatus.SUCCESS);
      });

      it('Emit and check in separate enqueued calls but same tx', async () => {
        const nullifier = new Fr(123456);

        // This will create 1 tx with 2 public calls in it.
        await new BatchCall(wallet, [
          avmContract.methods.new_nullifier(nullifier).request(),
          avmContract.methods.assert_nullifier_exists(nullifier).request(),
        ])
          .send()
          .wait();
      });
    });

    describe('Nested calls', () => {
      it('Should NOT be able to emit the same unsiloed nullifier from the same contract', async () => {
        const nullifier = new Fr(1);
        await expect(
          avmContract.methods.create_same_nullifier_in_nested_call(avmContract.address, nullifier).send().wait(),
        ).rejects.toThrow();
      });

      it('Should be able to emit different unsiloed nullifiers from the same contract', async () => {
        const nullifier = new Fr(1);
        const tx = await avmContract.methods
          .create_different_nullifier_in_nested_call(avmContract.address, nullifier)
          .send()
          .wait();
        expect(tx.status).toEqual(TxStatus.SUCCESS);
      });

      it('Should be able to emit the same unsiloed nullifier from two different contracts', async () => {
        const nullifier = new Fr(1);
        const tx = await avmContract.methods
          .create_same_nullifier_in_nested_call(secondAvmContract.address, nullifier)
          .send()
          .wait();
        expect(tx.status).toEqual(TxStatus.SUCCESS);
      });

      it('Should be able to emit different unsiloed nullifiers from two different contracts', async () => {
        const nullifier = new Fr(1);
        const tx = await avmContract.methods
          .create_different_nullifier_in_nested_call(secondAvmContract.address, nullifier)
          .send()
          .wait();
        expect(tx.status).toEqual(TxStatus.SUCCESS);
      });
    });
  });

  describe('AvmInitializerTestContract', () => {
    let avmContract: AvmInitializerTestContract;

    beforeEach(async () => {
      avmContract = await AvmInitializerTestContract.deploy(wallet).send().deployed();
    });

    describe('Storage', () => {
      it('Read immutable (initialized) storage (Field)', async () => {
        expect(await avmContract.methods.read_storage_immutable().simulate()).toEqual(42n);
      });
    });
  });
});
