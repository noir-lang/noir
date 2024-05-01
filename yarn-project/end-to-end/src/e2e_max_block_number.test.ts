import { Fr, type PXE, type Wallet } from '@aztec/aztec.js';
import { TestContract } from '@aztec/noir-contracts.js';

import { setup } from './fixtures/utils.js';

describe('e2e_max_block_number', () => {
  let wallet: Wallet;
  let pxe: PXE;
  let teardown: () => Promise<void>;

  let contract: TestContract;

  beforeAll(async () => {
    ({ teardown, wallet, pxe } = await setup());
    contract = await TestContract.deploy(wallet).send().deployed();
  });

  afterAll(() => teardown());

  describe('when requesting max block numbers higher than the mined one', () => {
    let maxBlockNumber: number;

    beforeEach(async () => {
      maxBlockNumber = (await pxe.getBlockNumber()) + 20;
    });

    describe('with no enqueued public calls', () => {
      const enqueuePublicCall = false;

      it('sets the max block number', async () => {
        const tx = await contract.methods.set_tx_max_block_number(maxBlockNumber, enqueuePublicCall).prove();
        expect(tx.data.forRollup!.rollupValidationRequests.maxBlockNumber.isSome).toEqual(true);
        expect(tx.data.forRollup!.rollupValidationRequests.maxBlockNumber.value).toEqual(new Fr(maxBlockNumber));
      });

      it('does not invalidate the transaction', async () => {
        await contract.methods.set_tx_max_block_number(maxBlockNumber, enqueuePublicCall).send().wait();
      });
    });

    describe('with an enqueued public call', () => {
      const enqueuePublicCall = true;

      it('sets the max block number', async () => {
        const tx = await contract.methods.set_tx_max_block_number(maxBlockNumber, enqueuePublicCall).prove();
        expect(tx.data.forPublic!.validationRequests.forRollup.maxBlockNumber.isSome).toEqual(true);
        expect(tx.data.forPublic!.validationRequests.forRollup.maxBlockNumber.value).toEqual(new Fr(maxBlockNumber));
      });

      it('does not invalidate the transaction', async () => {
        await contract.methods.set_tx_max_block_number(maxBlockNumber, enqueuePublicCall).send().wait();
      });
    });
  });

  describe('when requesting max block numbers lower than the mined one', () => {
    let maxBlockNumber: number;

    beforeEach(async () => {
      maxBlockNumber = await pxe.getBlockNumber();
    });

    describe('with no enqueued public calls', () => {
      const enqueuePublicCall = false;

      it('sets the max block number', async () => {
        const tx = await contract.methods.set_tx_max_block_number(maxBlockNumber, enqueuePublicCall).prove();
        expect(tx.data.forRollup!.rollupValidationRequests.maxBlockNumber.isSome).toEqual(true);
        expect(tx.data.forRollup!.rollupValidationRequests.maxBlockNumber.value).toEqual(new Fr(maxBlockNumber));
      });

      it('invalidates the transaction', async () => {
        await expect(
          contract.methods.set_tx_max_block_number(maxBlockNumber, enqueuePublicCall).send().wait(),
        ).rejects.toThrow('dropped');
      });
    });

    describe('with an enqueued public call', () => {
      const enqueuePublicCall = true;

      it('sets the max block number', async () => {
        const tx = await contract.methods.set_tx_max_block_number(maxBlockNumber, enqueuePublicCall).prove();
        expect(tx.data.forPublic!.validationRequests.forRollup.maxBlockNumber.isSome).toEqual(true);
        expect(tx.data.forPublic!.validationRequests.forRollup.maxBlockNumber.value).toEqual(new Fr(maxBlockNumber));
      });

      it('invalidates the transaction', async () => {
        await expect(
          contract.methods.set_tx_max_block_number(maxBlockNumber, enqueuePublicCall).send().wait(),
        ).rejects.toThrow('dropped');
      });
    });
  });
});
