import { DelegateCallsTest } from './delegate_calls_test.js';

describe('e2e_delegate_calls', () => {
  const t = new DelegateCallsTest('delegate_calls');
  let { delegatorContract, delegatedOnContract, wallet } = t;

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.setup();
    // Have to destructure again to ensure we have latest refs.
    ({ delegatorContract, delegatedOnContract, wallet } = t);
  });

  afterAll(async () => {
    await t.teardown();
  });

  describe('delegates on another contract', () => {
    it("runs another contract's private function on delegator's storage", async () => {
      const sentValue = 42n;
      await delegatorContract.methods
        .private_delegate_set_value(delegatedOnContract.address, sentValue, wallet.getCompleteAddress().address)
        .send()
        .wait();

      const delegatorValue = await delegatorContract.methods
        .view_private_value(sentValue, wallet.getCompleteAddress().address)
        .simulate();

      const delegatedOnValue = await delegatedOnContract.methods
        .view_private_value(sentValue, wallet.getCompleteAddress().address)
        .simulate();

      expect(delegatedOnValue).toEqual(0n);
      expect(delegatorValue).toEqual(sentValue);
    }, 100_000);

    it("runs another contract's enqueued public function on delegator's storage", async () => {
      const sentValue = 42n;
      await delegatorContract.methods.enqueued_delegate_set_value(delegatedOnContract.address, sentValue).send().wait();

      const delegatorValue = await delegatorContract.methods.view_public_value().simulate();
      const delegatedOnValue = await delegatedOnContract.methods.view_public_value().simulate();

      expect(delegatedOnValue).toEqual(0n);
      expect(delegatorValue).toEqual(sentValue);
    }, 100_000);

    it("runs another contract's public function on delegator's storage", async () => {
      const sentValue = 42n;
      await delegatorContract.methods.public_delegate_set_value(delegatedOnContract.address, sentValue).send().wait();

      const delegatorValue = await delegatorContract.methods.view_public_value().simulate();
      const delegatedOnValue = await delegatedOnContract.methods.view_public_value().simulate();

      expect(delegatedOnValue).toEqual(0n);
      expect(delegatorValue).toEqual(sentValue);
    }, 100_000);
  });
});
