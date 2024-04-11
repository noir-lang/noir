import { type AccountWallet, AztecAddress, Fr, FunctionSelector, TxStatus } from '@aztec/aztec.js';
import {
  AvmAcvmInteropTestContract,
  AvmInitializerTestContract,
  AvmNestedCallsTestContract,
  AvmTestContract,
} from '@aztec/noir-contracts.js';

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
  });

  describe('ACVM interoperability', () => {
    let avmContract: AvmAcvmInteropTestContract;

    beforeEach(async () => {
      avmContract = await AvmAcvmInteropTestContract.deploy(wallet).send().deployed();
    }, 50_000);

    it('Can execute ACVM function among AVM functions', async () => {
      expect(await avmContract.methods.constant_field_acvm().simulate()).toEqual([123456n]);
    });

    it('Can call AVM function from ACVM', async () => {
      expect(await avmContract.methods.call_avm_from_acvm().simulate()).toEqual([123456n]);
    });

    it('Can call ACVM function from AVM', async () => {
      expect(await avmContract.methods.call_acvm_from_avm().simulate()).toEqual([123456n]);
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
    describe('Authwit', () => {
      it('Works if authwit provided', async () => {
        const recipient = AztecAddress.random();
        const action = avmContract.methods.test_authwit_send_money(
          /*from=*/ wallet.getCompleteAddress(),
          recipient,
          100,
        );
        let tx = await wallet
          .setPublicAuthWit({ caller: wallet.getCompleteAddress().address, action }, /*authorized=*/ true)
          .send()
          .wait();
        expect(tx.status).toEqual(TxStatus.MINED);

        tx = await avmContract.methods
          .test_authwit_send_money(/*from=*/ wallet.getCompleteAddress(), recipient, 100)
          .send()
          .wait();
        expect(tx.status).toEqual(TxStatus.MINED);
      });

      it('Fails if authwit not provided', async () => {
        await expect(
          async () =>
            await avmContract.methods
              .test_authwit_send_money(/*from=*/ wallet.getCompleteAddress(), /*to=*/ AztecAddress.random(), 100)
              .send()
              .wait(),
        ).rejects.toThrow(/Message not authorized by account/);
      });
    });

    describe('AvmInitializerTestContract', () => {
      let avmContract: AvmInitializerTestContract;

      beforeEach(async () => {
        avmContract = await AvmInitializerTestContract.deploy(wallet).send().deployed();
      }, 50_000);

      describe('Storage', () => {
        it('Read immutable (initialized) storage (Field)', async () => {
          expect(await avmContract.methods.read_storage_immutable().simulate()).toEqual([42n]);
        });
      });
    });
  });

  describe('AvmNestedCallsTestContract', () => {
    let avmContract: AvmNestedCallsTestContract;
    let secondAvmContract: AvmNestedCallsTestContract;

    beforeEach(async () => {
      avmContract = await AvmNestedCallsTestContract.deploy(wallet).send().deployed();
      secondAvmContract = await AvmNestedCallsTestContract.deploy(wallet).send().deployed();
    }, 50_000);

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
      expect(tx.status).toEqual(TxStatus.MINED);
    });
    // TODO(4293): this should work! Fails in public kernel because both nullifiers are incorrectly being siloed by same address
    it.skip('Should be able to emit the same unsiloed nullifier from two different contracts', async () => {
      const nullifier = new Fr(1);
      const tx = await avmContract.methods
        .create_same_nullifier_in_nested_call(secondAvmContract.address, nullifier)
        .send()
        .wait();
      expect(tx.status).toEqual(TxStatus.MINED);
    });
    it('Should be able to emit different unsiloed nullifiers from two different contracts', async () => {
      const nullifier = new Fr(1);
      const tx = await avmContract.methods
        .create_different_nullifier_in_nested_call(secondAvmContract.address, nullifier)
        .send()
        .wait();
      expect(tx.status).toEqual(TxStatus.MINED);
    });
  });
});
