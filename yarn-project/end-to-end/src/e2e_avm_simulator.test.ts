import { AztecAddress, Fr, TxStatus, type Wallet } from '@aztec/aztec.js';
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
