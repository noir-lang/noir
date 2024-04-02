import { type Wallet } from '@aztec/aztec.js';
import { ChildContract, ParentContract } from '@aztec/noir-contracts.js';

import { setup } from './fixtures/utils.js';

describe('e2e_static_calls', () => {
  let wallet: Wallet;
  let parentContract: ParentContract;
  let childContract: ChildContract;
  let teardown: () => Promise<void>;

  beforeEach(async () => {
    ({ teardown, wallet } = await setup());
  }, 100_000);

  afterEach(() => teardown());

  beforeEach(async () => {
    parentContract = await ParentContract.deploy(wallet).send().deployed();
    childContract = await ChildContract.deploy(wallet).send().deployed();
  }, 100_000);

  describe('parent calls child', () => {
    it('performs legal private to private static calls', async () => {
      // We create a note in the set, so...
      await childContract.methods.private_set_value(42n, wallet.getCompleteAddress().address).send().wait();
      // ...this call doesn't fail due to get_notes returning 0 notes
      await parentContract.methods
        .private_static_call(childContract.address, childContract.methods.private_get_value.selector, [
          42n,
          wallet.getCompleteAddress().address,
        ])
        .send()
        .wait();
    }, 100_000);

    it('performs legal (nested) private to private static calls', async () => {
      // We create a note in the set, so...
      await childContract.methods.private_set_value(42n, wallet.getCompleteAddress().address).send().wait();
      // ...this call doesn't fail due to get_notes returning 0 notes
      await parentContract.methods
        .private_nested_static_call(childContract.address, childContract.methods.private_get_value.selector, [
          42n,
          wallet.getCompleteAddress().address,
        ])
        .send()
        .wait();
    }, 100_000);

    it('performs legal public to public static calls', async () => {
      await parentContract.methods
        .public_static_call(childContract.address, childContract.methods.pub_get_value.selector, [42n])
        .send()
        .wait();
    }, 100_000);

    it('performs legal (nested) public to public static calls', async () => {
      await parentContract.methods
        .public_nested_static_call(childContract.address, childContract.methods.pub_get_value.selector, [42n])
        .send()
        .wait();
    }, 100_000);

    it('performs legal enqueued public static calls', async () => {
      await parentContract.methods
        .enqueue_static_call_to_pub_function(childContract.address, childContract.methods.pub_get_value.selector, [42n])
        .send()
        .wait();
    }, 100_000);

    it('performs legal (nested) enqueued public static calls', async () => {
      await parentContract.methods
        .enqueue_static_nested_call_to_pub_function(
          childContract.address,
          childContract.methods.pub_get_value.selector,
          [42n],
        )
        .send()
        .wait();
    }, 100_000);

    it('fails when performing illegal private to private static calls', async () => {
      await expect(
        parentContract.methods
          .private_static_call(childContract.address, childContract.methods.private_set_value.selector, [
            42n,
            wallet.getCompleteAddress().address,
          ])
          .send()
          .wait(),
      ).rejects.toThrow('Static call cannot create new notes, emit L2->L1 messages or generate logs');
    }, 100_000);

    it('fails when performing illegal (nested) private to private static calls', async () => {
      await expect(
        parentContract.methods
          .private_nested_static_call(childContract.address, childContract.methods.private_set_value.selector, [
            42n,
            wallet.getCompleteAddress().address,
          ])
          .send()
          .wait(),
      ).rejects.toThrow('Static call cannot create new notes, emit L2->L1 messages or generate logs');
    }, 100_000);

    it('fails when performing illegal public to public static calls', async () => {
      await expect(
        parentContract.methods
          .public_static_call(childContract.address, childContract.methods.pub_set_value.selector, [42n])
          .send()
          .wait(),
      ).rejects.toThrow('Static call cannot update the state, emit L2->L1 messages or generate logs');
    }, 100_000);

    it('fails when performing illegal (nested) public to public static calls', async () => {
      await expect(
        parentContract.methods
          .public_nested_static_call(childContract.address, childContract.methods.pub_set_value.selector, [42n])
          .send()
          .wait(),
      ).rejects.toThrow('Static call cannot update the state, emit L2->L1 messages or generate logs');
    }, 100_000);

    it('fails when performing illegal enqueued public static calls', async () => {
      await expect(
        parentContract.methods
          .enqueue_static_call_to_pub_function(childContract.address, childContract.methods.pub_set_value.selector, [
            42n,
          ])
          .send()
          .wait(),
      ).rejects.toThrow('Static call cannot update the state, emit L2->L1 messages or generate logs');
    }, 100_000);

    it('fails when performing illegal (nested) enqueued public static calls', async () => {
      await expect(
        parentContract.methods
          .enqueue_static_nested_call_to_pub_function(
            childContract.address,
            childContract.methods.pub_set_value.selector,
            [42n],
          )
          .send()
          .wait(),
      ).rejects.toThrow('Static call cannot update the state, emit L2->L1 messages or generate logs');
    }, 100_000);
  });
});
