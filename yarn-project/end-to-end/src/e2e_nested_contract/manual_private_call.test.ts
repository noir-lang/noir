import { NestedContractTest } from './nested_contract_test.js';

describe('e2e_nested_contract manual', () => {
  const t = new NestedContractTest('manual');
  let { parentContract, childContract } = t;

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.applyManualSnapshots();
    await t.setup();
    ({ parentContract, childContract } = t);
  });

  afterAll(async () => {
    await t.teardown();
  });

  it('performs nested calls', async () => {
    await parentContract.methods.entry_point(childContract.address, childContract.methods.value.selector).send().wait();
  });

  it('fails simulation if calling a function not allowed to be called externally', async () => {
    await expect(
      parentContract.methods
        .entry_point(childContract.address, (childContract.methods as any).value_internal.selector)
        .prove(),
    ).rejects.toThrow(/Assertion failed: Function value_internal can only be called internally/);
  });
});
