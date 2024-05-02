import { type AztecAddress, Fr } from '@aztec/aztec.js';
import { ChildContract, ParentContract } from '@aztec/noir-contracts.js';

import { NestedContractTest } from './nested_contract_test.js';

describe('e2e_nested_contract manual_enqueue', () => {
  const t = new NestedContractTest('manual_enqueue');
  let { wallets, pxe, parentContract, childContract } = t;

  const getChildStoredValue = (child: { address: AztecAddress }) => pxe.getPublicStorageAt(child.address, new Fr(1));

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    // We don't have the manual snapshot because every test requires a fresh setup and teardown
    await t.setup();
    ({ wallets, pxe } = t);
  });

  beforeEach(async () => {
    parentContract = await ParentContract.deploy(wallets[0]).send().deployed();
    childContract = await ChildContract.deploy(wallets[0]).send().deployed();
  });

  afterAll(async () => {
    await t.teardown();
  });

  it('enqueues a single public call', async () => {
    await parentContract.methods
      .enqueue_call_to_child(childContract.address, childContract.methods.pub_inc_value.selector, 42n)
      .send()
      .wait();
    expect(await getChildStoredValue(childContract)).toEqual(new Fr(42n));
  });

  it('fails simulation if calling a public function not allowed to be called externally', async () => {
    await expect(
      parentContract.methods
        .enqueue_call_to_child(
          childContract.address,
          (childContract.methods as any).pub_inc_value_internal.selector,
          42n,
        )
        .prove(),
    ).rejects.toThrow(/Assertion failed: Function pub_inc_value_internal can only be called internally/);
  });

  it('enqueues multiple public calls', async () => {
    await parentContract.methods
      .enqueue_call_to_child_twice(childContract.address, childContract.methods.pub_inc_value.selector, 42n)
      .send()
      .wait();
    expect(await getChildStoredValue(childContract)).toEqual(new Fr(85n));
  });

  it('enqueues a public call with nested public calls', async () => {
    await parentContract.methods
      .enqueue_call_to_pub_entry_point(childContract.address, childContract.methods.pub_inc_value.selector, 42n)
      .send()
      .wait();
    expect(await getChildStoredValue(childContract)).toEqual(new Fr(42n));
  });

  it('enqueues multiple public calls with nested public calls', async () => {
    await parentContract.methods
      .enqueue_calls_to_pub_entry_point(childContract.address, childContract.methods.pub_inc_value.selector, 42n)
      .send()
      .wait();
    expect(await getChildStoredValue(childContract)).toEqual(new Fr(85n));
  });
});
