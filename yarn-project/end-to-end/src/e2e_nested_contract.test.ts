import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer } from '@aztec/aztec-rpc';
import { AztecAddress, BatchCall, Fr, Wallet } from '@aztec/aztec.js';
import { toBigIntBE } from '@aztec/foundation/bigint-buffer';
import { DebugLogger } from '@aztec/foundation/log';
import { toBigInt } from '@aztec/foundation/serialize';
import { ChildContract, ImportTestContract, ParentContract, TestContract } from '@aztec/noir-contracts/types';
import { AztecRPC, L2BlockL2Logs } from '@aztec/types';

import { setup } from './fixtures/utils.js';

describe('e2e_nested_contract', () => {
  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServer: AztecRPC;
  let wallet: Wallet;
  let logger: DebugLogger;

  beforeEach(async () => {
    ({ aztecNode, aztecRpcServer, wallet, logger } = await setup());
  }, 100_000);

  afterEach(async () => {
    await aztecNode?.stop();
    if (aztecRpcServer instanceof AztecRPCServer) {
      await aztecRpcServer?.stop();
    }
  });

  describe('parent manually calls child', () => {
    let parentContract: ParentContract;
    let childContract: ChildContract;

    beforeEach(async () => {
      parentContract = await ParentContract.deploy(wallet).send().deployed();
      childContract = await ChildContract.deploy(wallet).send().deployed();
    }, 100_000);

    const getChildStoredValue = (child: { address: AztecAddress }) =>
      aztecRpcServer.getPublicStorageAt(child.address, new Fr(1)).then(x => toBigInt(x!));

    it('performs nested calls', async () => {
      await parentContract.methods
        .entryPoint(childContract.address, childContract.methods.value.selector.toField())
        .send()
        .wait();
    }, 100_000);

    it('fails simulation if calling a function not allowed to be called externally', async () => {
      await expect(
        parentContract.methods
          .entryPoint(childContract.address, childContract.methods.valueInternal.selector.toField())
          .simulate(),
      ).rejects.toThrowError('Assertion failed: Sender must be this contract');
    }, 100_000);

    it('performs public nested calls', async () => {
      await parentContract.methods
        .pubEntryPoint(childContract.address, childContract.methods.pubGetValue.selector.toField(), 42n)
        .send()
        .wait();
    }, 100_000);

    it('enqueues a single public call', async () => {
      await parentContract.methods
        .enqueueCallToChild(childContract.address, childContract.methods.pubIncValue.selector.toField(), 42n)
        .send()
        .wait();
      expect(await getChildStoredValue(childContract)).toEqual(42n);
    }, 100_000);

    it('fails simulation if calling a public function not allowed to be called externally', async () => {
      await expect(
        parentContract.methods
          .enqueueCallToChild(childContract.address, childContract.methods.pubIncValueInternal.selector.toField(), 42n)
          .simulate(),
      ).rejects.toThrowError('Assertion failed: Sender must be this contract');
    }, 100_000);

    it('enqueues multiple public calls', async () => {
      await parentContract.methods
        .enqueueCallToChildTwice(childContract.address, childContract.methods.pubIncValue.selector.value, 42n)
        .send()
        .wait();
      expect(await getChildStoredValue(childContract)).toEqual(85n);
    }, 100_000);

    it('enqueues a public call with nested public calls', async () => {
      await parentContract.methods
        .enqueueCallToPubEntryPoint(childContract.address, childContract.methods.pubIncValue.selector.toField(), 42n)
        .send()
        .wait();
      expect(await getChildStoredValue(childContract)).toEqual(42n);
    }, 100_000);

    it('enqueues multiple public calls with nested public calls', async () => {
      await parentContract.methods
        .enqueueCallsToPubEntryPoint(childContract.address, childContract.methods.pubIncValue.selector.toField(), 42n)
        .send()
        .wait();
      expect(await getChildStoredValue(childContract)).toEqual(85n);
    }, 100_000);

    // Regression for https://github.com/AztecProtocol/aztec-packages/issues/640
    it('reads fresh value after write within the same tx', async () => {
      await parentContract.methods
        .pubEntryPointTwice(childContract.address, childContract.methods.pubIncValue.selector.value, 42n)
        .send()
        .wait();
      expect(await getChildStoredValue(childContract)).toEqual(84n);
    }, 100_000);

    // Regression for https://github.com/AztecProtocol/aztec-packages/issues/1645
    // Executes a public call first and then a private call (which enqueues another public call)
    // through the account contract, if the account entrypoint behaves properly, it will honor
    // this order and not run the private call first which results in the public calls being inverted.
    it('executes public calls in expected order', async () => {
      const pubSetValueSelector = childContract.methods.pubSetValue.selector.toField();
      const actions = [
        childContract.methods.pubSetValue(20n).request(),
        parentContract.methods.enqueueCallToChild(childContract.address, pubSetValueSelector, 40n).request(),
      ];

      const tx = await new BatchCall(wallet, actions).send().wait();
      const logs = L2BlockL2Logs.unrollLogs(await wallet.getUnencryptedLogs(tx.blockNumber!, 1)).map(toBigIntBE);
      expect(logs).toEqual([20n, 40n]);
      expect(await getChildStoredValue(childContract)).toEqual(40n);
    });
  });

  describe('importer uses autogenerated test contract interface', () => {
    let importerContract: ImportTestContract;
    let testContract: TestContract;

    beforeEach(async () => {
      logger(`Deploying importer test contract`);
      importerContract = await ImportTestContract.deploy(wallet).send().deployed();
      logger(`Deploying test contract`);
      testContract = await TestContract.deploy(wallet).send().deployed();
    }, 30_000);

    it('calls a method with multiple arguments', async () => {
      logger(`Calling main on importer contract`);
      await importerContract.methods.main(testContract.address).send().wait();
    }, 30_000);

    it('calls a method no arguments', async () => {
      logger(`Calling noargs on importer contract`);
      await importerContract.methods.callNoArgs(testContract.address).send().wait();
    }, 30_000);

    it('calls an open function', async () => {
      logger(`Calling openfn on importer contract`);
      await importerContract.methods.callOpenFn(testContract.address).send().wait();
    }, 30_000);

    it('calls an open function from an open function', async () => {
      logger(`Calling pub openfn on importer contract`);
      await importerContract.methods.pubCallOpenFn(testContract.address).send().wait();
    }, 30_000);
  });
});
