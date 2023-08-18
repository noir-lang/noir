// Test suite for testing proper ordering of side effects
import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer } from '@aztec/aztec-rpc';
import { Wallet } from '@aztec/aztec.js';
import { Fr } from '@aztec/circuits.js';
import { toBigInt } from '@aztec/foundation/serialize';
import { ChildContract, ParentContract } from '@aztec/noir-contracts/types';
import { AztecRPC, L2BlockL2Logs } from '@aztec/types';

import { setup } from './fixtures/utils.js';

// See https://github.com/AztecProtocol/aztec-packages/issues/1601
describe('e2e_ordering', () => {
  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServer: AztecRPC;
  let wallet: Wallet;

  beforeEach(async () => {
    ({ aztecNode, aztecRpcServer, wallet } = await setup());
  }, 100_000);

  afterEach(async () => {
    await aztecNode?.stop();
    if (aztecRpcServer instanceof AztecRPCServer) {
      await aztecRpcServer?.stop();
    }
  });

  describe('with parent and child contract', () => {
    let parent: ParentContract;
    let child: ChildContract;
    let pubSetValueSelector: Buffer;

    beforeEach(async () => {
      parent = await ParentContract.deploy(wallet).send().deployed();
      child = await ChildContract.deploy(wallet).send().deployed();
      pubSetValueSelector = child.methods.pubSetValue.selector;
    });

    describe('enqueued public calls ordering', () => {
      const nestedValue = 10n;
      const directValue = 20n;

      const expectedOrders = {
        enqueueCallsToChildWithNestedFirst: [nestedValue, directValue],
        enqueueCallsToChildWithNestedLast: [directValue, nestedValue],
      } as const;

      it.each(['enqueueCallsToChildWithNestedFirst', 'enqueueCallsToChildWithNestedLast'] as const)(
        'orders public function execution in %s',
        async method => {
          const expectedOrder = expectedOrders[method];
          const action = parent.methods[method](child.address, pubSetValueSelector);
          const tx = await action.simulate();
          await action.send().wait();

          // There are two enqueued calls
          const enqueuedPublicCalls = tx.enqueuedPublicFunctionCalls;
          expect(enqueuedPublicCalls.length).toEqual(2);

          // The call stack hashes in the output of the kernel proof match the tx enqueuedPublicFunctionCalls
          const hashes = await Promise.all(enqueuedPublicCalls.map(c => c.toPublicCallStackItem().then(i => i.hash())));
          expect(tx.data.end.publicCallStack.slice(0, 2)).toEqual(hashes);

          // The enqueued public calls are in the expected order based on the argument they set (stack is reversed!)
          expect(enqueuedPublicCalls.map(c => c.args[0].toBigInt())).toEqual([...expectedOrder].reverse());

          // Logs are emitted in the expected order
          const logs = await aztecRpcServer.getUnencryptedLogs(1, 10).then(L2BlockL2Logs.unrollLogs);
          const expectedLogs = expectedOrder.map(x => Buffer.from([Number(x)]));
          expect(logs).toEqual(expectedLogs);

          // The final value of the child is the last one set
          const value = await aztecRpcServer.getPublicStorageAt(child.address, new Fr(1)).then(x => toBigInt(x!));
          expect(value).toEqual(expectedOrder[1]);
        },
      );
    });
  });
});
