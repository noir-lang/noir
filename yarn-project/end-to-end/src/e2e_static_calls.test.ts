import { Wallet } from '@aztec/aztec.js';
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
      await parentContract.methods
        .privateStaticCall(childContract.address, childContract.methods.privateGetValue.selector, [
          42n,
          wallet.getCompleteAddress().address,
        ])
        .send()
        .wait();
    }, 100_000);

    it('performs legal public to public static calls', async () => {
      await parentContract.methods
        .enqueueStaticCallToPubFunction(childContract.address, childContract.methods.pubGetValue.selector, [42n])
        .send()
        .wait();
    }, 100_000);

    it('performs legal enqueued public static calls', async () => {
      await parentContract.methods
        .publicStaticCall(childContract.address, childContract.methods.pubGetValue.selector, [42n])
        .send()
        .wait();
    }, 100_000);

    it('fails when performing illegal private to private static calls', async () => {
      await expect(
        parentContract.methods
          .privateStaticCall(childContract.address, childContract.methods.privateSetValue.selector, [
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
          .publicStaticCall(childContract.address, childContract.methods.pubSetValue.selector, [42n])
          .send()
          .wait(),
      ).rejects.toThrow('Static call cannot update the state, emit L2->L1 messages or generate logs');
    }, 100_000);

    it('fails when performing illegal enqueued public static calls', async () => {
      await expect(
        parentContract.methods
          .enqueueStaticCallToPubFunction(childContract.address, childContract.methods.pubSetValue.selector, [42n])
          .send()
          .wait(),
      ).rejects.toThrow('Static call cannot update the state, emit L2->L1 messages or generate logs');
    }, 100_000);
  });
});
