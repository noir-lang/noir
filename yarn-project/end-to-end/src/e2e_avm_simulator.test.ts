import { AztecAddress, Fr, FunctionSelector, TxStatus, type Wallet } from '@aztec/aztec.js';
import { AvmInitializerTestContract, AvmTestContract } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import { setup } from './fixtures/utils.js';

const TIMEOUT = 100_000;

describe('e2e_avm_simulator', () => {
  jest.setTimeout(TIMEOUT);

  let wallet: Wallet;
  let teardown: () => Promise<void>;

  beforeAll(async () => {
    ({ teardown, wallet } = await setup());
  }, 100_000);

  afterAll(() => teardown());

  describe('AvmTestContract', () => {
    let avmContract: AvmTestContract;

    beforeEach(async () => {
      avmContract = await AvmTestContract.deploy(wallet).send().deployed();
    }, 50_000);

    describe('Storage', () => {
      it('Modifies storage (Field)', async () => {
        await avmContract.methods.set_storage_single(20n).send().wait();
        expect(await avmContract.methods.view_storage_single().simulate()).toEqual(20n);
      });

      it('Modifies storage (Map)', async () => {
        const address = AztecAddress.fromBigInt(9090n);
        await avmContract.methods.set_storage_map(address, 100).send().wait();
        await avmContract.methods.add_storage_map(address, 100).send().wait();
        expect(await avmContract.methods.view_storage_map(address).simulate()).toEqual(200n);
      });
    });

    describe('Contract instance', () => {
      it('Works', async () => {
        const tx = await avmContract.methods.test_get_contract_instance().send().wait();
        expect(tx.status).toEqual(TxStatus.MINED);
      });
    });

    describe('Nullifiers', () => {
      // Nullifier will not yet be siloed by the kernel.
      it('Emit and check in the same tx', async () => {
        const tx = await avmContract.methods.emit_nullifier_and_check(123456).send().wait();
        expect(tx.status).toEqual(TxStatus.MINED);
      });

      // Nullifier will have been siloed by the kernel, but we check against the unsiloed one.
      it('Emit and check in separate tx', async () => {
        const nullifier = new Fr(123456);
        let tx = await avmContract.methods.new_nullifier(nullifier).send().wait();
        expect(tx.status).toEqual(TxStatus.MINED);

        tx = await avmContract.methods.assert_nullifier_exists(nullifier).send().wait();
        expect(tx.status).toEqual(TxStatus.MINED);
      });
    });

    describe('ACVM interoperability', () => {
      it('Can execute ACVM function among AVM functions', async () => {
        expect(await avmContract.methods.constant_field_acvm().simulate()).toEqual([123456n, 0n, 0n, 0n]);
      });

      it('Can call AVM function from ACVM', async () => {
        expect(await avmContract.methods.call_avm_from_acvm().simulate()).toEqual([123456n, 0n, 0n, 0n]);
      });

      it('Can call ACVM function from AVM', async () => {
        expect(await avmContract.methods.call_acvm_from_avm().simulate()).toEqual([123456n, 0n, 0n, 0n]);
      });

      it('AVM sees settled nullifiers by ACVM', async () => {
        const nullifier = new Fr(123456);
        await avmContract.methods.new_nullifier(nullifier).send().wait();
        await avmContract.methods.assert_unsiloed_nullifier_acvm(nullifier).send().wait();
      });

      it('AVM nested call to ACVM sees settled nullifiers', async () => {
        const nullifier = new Fr(123456);
        await avmContract.methods.new_nullifier(nullifier).send().wait();
        await avmContract.methods
          .avm_to_acvm_call(FunctionSelector.fromSignature('assert_unsiloed_nullifier_acvm(Field)'), nullifier)
          .send()
          .wait();
      });
    });
  });

  describe('AvmInitializerTestContract', () => {
    let avmContract: AvmInitializerTestContract;

    beforeEach(async () => {
      avmContract = await AvmInitializerTestContract.deploy(wallet).send().deployed();
    }, 50_000);

    describe('Storage', () => {
      it('Read immutable (initialized) storage (Field)', async () => {
        expect(await avmContract.methods.view_storage_immutable().simulate()).toEqual(42n);
      });
    });
  });
});
