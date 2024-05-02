import { type AztecAddress, BatchCall, Fr, type Wallet, toBigIntBE } from '@aztec/aztec.js';

import { NestedContractTest } from './nested_contract_test.js';

describe('e2e_nested_contract manual', () => {
  const t = new NestedContractTest('manual');
  let wallet: Wallet;
  let { wallets, pxe, parentContract, childContract } = t;

  const getChildStoredValue = (child: { address: AztecAddress }) => pxe.getPublicStorageAt(child.address, new Fr(1));

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.applyManualSnapshots();
    await t.setup();
    ({ wallets, pxe, parentContract, childContract } = t);
    wallet = wallets[0];
  });

  afterAll(async () => {
    await t.teardown();
  });

  it('performs public nested calls', async () => {
    await parentContract.methods
      .pub_entry_point(childContract.address, childContract.methods.pub_get_value.selector, 42n)
      .send()
      .wait();
  });

  // Regression for https://github.com/AztecProtocol/aztec-packages/issues/640
  it('reads fresh value after write within the same tx', async () => {
    await parentContract.methods
      .pub_entry_point_twice(childContract.address, childContract.methods.pub_inc_value.selector, 42n)
      .send()
      .wait();
    expect(await getChildStoredValue(childContract)).toEqual(new Fr(84n));
  });

  // Regression for https://github.com/AztecProtocol/aztec-packages/issues/1645
  // Executes a public call first and then a private call (which enqueues another public call)
  // through the account contract, if the account entrypoint behaves properly, it will honor
  // this order and not run the private call first which results in the public calls being inverted.
  it('executes public calls in expected order', async () => {
    const pubSetValueSelector = childContract.methods.pub_set_value.selector;
    const actions = [
      childContract.methods.pub_set_value(20n).request(),
      parentContract.methods.enqueue_call_to_child(childContract.address, pubSetValueSelector, 40n).request(),
    ];

    const tx = await new BatchCall(wallet, actions).send().wait();
    const extendedLogs = (
      await wallet.getUnencryptedLogs({
        fromBlock: tx.blockNumber!,
      })
    ).logs;
    const processedLogs = extendedLogs.map(extendedLog => toBigIntBE(extendedLog.log.data));
    expect(processedLogs).toEqual([20n, 40n]);
    expect(await getChildStoredValue(childContract)).toEqual(new Fr(40n));
  });
});
