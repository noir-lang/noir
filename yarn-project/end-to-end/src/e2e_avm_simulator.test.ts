import { AztecAddress, Fr, TxStatus, type Wallet } from '@aztec/aztec.js';
import { AvmTestContract } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import { setup } from './fixtures/utils.js';

const TIMEOUT = 100_000;

describe('e2e_avm_simulator', () => {
  jest.setTimeout(TIMEOUT);

  let wallet: Wallet;
  let avmContact: AvmTestContract;
  let teardown: () => Promise<void>;

  beforeAll(async () => {
    ({ teardown, wallet } = await setup());
  }, 100_000);

  afterAll(() => teardown());

  beforeEach(async () => {
    avmContact = await AvmTestContract.deploy(wallet).send().deployed();
  }, 50_000);

  describe('Storage', () => {
    it('Modifies storage (Field)', async () => {
      await avmContact.methods.set_storage_single(20n).send().wait();
      expect(await avmContact.methods.view_storage_single().simulate()).toEqual(20n);
    });

    it('Modifies storage (Map)', async () => {
      const address = AztecAddress.fromBigInt(9090n);
      await avmContact.methods.set_storage_map(address, 100).send().wait();
      await avmContact.methods.add_storage_map(address, 100).send().wait();
      expect(await avmContact.methods.view_storage_map(address).simulate()).toEqual(200n);
    });
  });

  describe('Contract instance', () => {
    it('Works', async () => {
      const tx = await avmContact.methods.test_get_contract_instance().send().wait();
      expect(tx.status).toEqual(TxStatus.MINED);
    });
  });

  describe('Nullifiers', () => {
    // Nullifier will not yet be siloed by the kernel.
    it('Emit and check in the same tx', async () => {
      const tx = await avmContact.methods.emit_nullifier_and_check(123456).send().wait();
      expect(tx.status).toEqual(TxStatus.MINED);
    });

    // Nullifier will have been siloed by the kernel, but we check against the unsiloed one.
    it('Emit and check in separate tx', async () => {
      const nullifier = new Fr(123456);
      let tx = await avmContact.methods.new_nullifier(nullifier).send().wait();
      expect(tx.status).toEqual(TxStatus.MINED);

      tx = await avmContact.methods.assert_nullifier_exists(nullifier).send().wait();
      expect(tx.status).toEqual(TxStatus.MINED);
    });
  });
});
