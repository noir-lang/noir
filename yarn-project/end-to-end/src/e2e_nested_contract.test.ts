import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer } from '@aztec/aztec-rpc';
import { AztecAddress, Contract, ContractDeployer, Fr, Wallet } from '@aztec/aztec.js';
import { ContractAbi } from '@aztec/foundation/abi';
import { DebugLogger } from '@aztec/foundation/log';
import { toBigInt } from '@aztec/foundation/serialize';
import { ChildContractAbi, ParentContractAbi } from '@aztec/noir-contracts/artifacts';
import { ChildContract, ImportTestContract, ParentContract, TestContract } from '@aztec/noir-contracts/types';
import { AztecRPC, CompleteAddress, TxStatus } from '@aztec/types';

import { setup } from './fixtures/utils.js';

describe('e2e_nested_contract', () => {
  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServer: AztecRPC;
  let wallet: Wallet;
  let sender: AztecAddress;
  let logger: DebugLogger;

  beforeEach(async () => {
    let accounts: CompleteAddress[];
    ({ aztecNode, aztecRpcServer, accounts, wallet, logger } = await setup());
    sender = accounts[0].address;
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
      parentContract = (await deployContract(ParentContractAbi)) as ParentContract;
      childContract = (await deployContract(ChildContractAbi)) as ChildContract;
    }, 100_000);

    const deployContract = async (abi: ContractAbi) => {
      logger(`Deploying L2 contract ${abi.name}...`);
      const deployer = new ContractDeployer(abi, aztecRpcServer);
      const tx = deployer.deploy().send();

      await tx.isMined({ interval: 0.1 });

      const receipt = await tx.getReceipt();
      const contract = await Contract.at(receipt.contractAddress!, abi, wallet);
      logger(`L2 contract ${abi.name} deployed at ${contract.address}`);
      return contract;
    };

    const addressToField = (address: AztecAddress): bigint => Fr.fromBuffer(address.toBuffer()).value;

    const getChildStoredValue = (child: { address: AztecAddress }) =>
      aztecRpcServer.getPublicStorageAt(child.address, new Fr(1)).then(x => toBigInt(x!));

    /**
     * Milestone 3.
     */
    it('performs nested calls', async () => {
      const tx = parentContract.methods
        .entryPoint(childContract.address, Fr.fromBuffer(childContract.methods.value.selector))
        .send({ origin: sender });

      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();

      expect(receipt.status).toBe(TxStatus.MINED);
    }, 100_000);

    it('performs public nested calls', async () => {
      const tx = parentContract.methods
        .pubEntryPoint(childContract.address, Fr.fromBuffer(childContract.methods.pubValue.selector), 42n)
        .send({ origin: sender });

      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();

      expect(receipt.status).toBe(TxStatus.MINED);
    }, 100_000);

    it('enqueues a single public call', async () => {
      const tx = parentContract.methods
        .enqueueCallToChild(childContract.address, Fr.fromBuffer(childContract.methods.pubStoreValue.selector), 42n)
        .send({ origin: sender });

      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();
      expect(receipt.status).toBe(TxStatus.MINED);

      expect(await getChildStoredValue(childContract)).toEqual(42n);
    }, 100_000);

    // Fails with "solver opcode resolution error: cannot solve opcode: expression has too many unknowns %EXPR [ 0 ]%"
    // See https://github.com/noir-lang/noir/issues/1347
    it.skip('enqueues multiple public calls', async () => {
      const tx = parentContract.methods
        .enqueueCallToChildTwice(
          addressToField(childContract.address),
          Fr.fromBuffer(childContract.methods.pubStoreValue.selector).value,
          42n,
        )
        .send({ origin: sender });

      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();
      expect(receipt.status).toBe(TxStatus.MINED);

      expect(await getChildStoredValue(childContract)).toEqual(85n);
    }, 100_000);

    it('enqueues a public call with nested public calls', async () => {
      const tx = parentContract.methods
        .enqueueCallToPubEntryPoint(
          childContract.address,
          Fr.fromBuffer(childContract.methods.pubStoreValue.selector),
          42n,
        )
        .send({ origin: sender });

      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();
      expect(receipt.status).toBe(TxStatus.MINED);

      expect(await getChildStoredValue(childContract)).toEqual(42n);
    }, 100_000);

    // Fails with "solver opcode resolution error: cannot solve opcode: expression has too many unknowns %EXPR [ 0 ]%"
    // See https://github.com/noir-lang/noir/issues/1347
    it.skip('enqueues multiple public calls with nested public calls', async () => {
      const tx = parentContract.methods
        .enqueueCallsToPubEntryPoint(
          childContract.address,
          Fr.fromBuffer(childContract.methods.pubStoreValue.selector),
          42n,
        )
        .send({ origin: sender });

      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();
      expect(receipt.status).toBe(TxStatus.MINED);

      expect(await getChildStoredValue(childContract)).toEqual(84n);
    }, 100_000);

    // Regression for https://github.com/AztecProtocol/aztec-packages/issues/640
    // Fails with "solver opcode resolution error: cannot solve opcode: expression has too many unknowns %EXPR [ 0 ]%"
    // See https://github.com/noir-lang/noir/issues/1347
    it.skip('reads fresh value after write within the same tx', async () => {
      const tx = parentContract.methods
        .pubEntryPointTwice(
          addressToField(childContract.address),
          Fr.fromBuffer(childContract.methods.pubStoreValue.selector).value,
          42n,
        )
        .send({ origin: sender });

      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();

      expect(receipt.status).toBe(TxStatus.MINED);
      expect(await getChildStoredValue(childContract)).toEqual(85n);
    }, 100_000);
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
  });
});
