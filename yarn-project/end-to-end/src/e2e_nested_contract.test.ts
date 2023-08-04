import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer } from '@aztec/aztec-rpc';
import { AztecAddress, Contract, ContractDeployer, Fr, Wallet } from '@aztec/aztec.js';
import { ContractAbi } from '@aztec/foundation/abi';
import { DebugLogger } from '@aztec/foundation/log';
import { toBigInt } from '@aztec/foundation/serialize';
import { ChildContractAbi, ParentContractAbi } from '@aztec/noir-contracts/artifacts';
import { ChildContract, ParentContract } from '@aztec/noir-contracts/types';
import { AztecRPC, TxStatus } from '@aztec/types';

import { setup } from './fixtures/utils.js';

describe('e2e_nested_contract', () => {
  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServer: AztecRPC;
  let wallet: Wallet;
  let accounts: AztecAddress[];
  let logger: DebugLogger;

  let parentContract: ParentContract;
  let childContract: ChildContract;

  beforeEach(async () => {
    ({ aztecNode, aztecRpcServer, accounts, wallet, logger } = await setup());

    parentContract = (await deployContract(ParentContractAbi)) as ParentContract;
    childContract = (await deployContract(ChildContractAbi)) as ChildContract;
  }, 100_000);

  afterEach(async () => {
    await aztecNode?.stop();
    if (aztecRpcServer instanceof AztecRPCServer) {
      await aztecRpcServer?.stop();
    }
  });

  const deployContract = async (abi: ContractAbi) => {
    logger(`Deploying L2 contract ${abi.name}...`);
    const deployer = new ContractDeployer(abi, aztecRpcServer);
    const tx = deployer.deploy().send();

    await tx.isMined({ interval: 0.1 });

    const receipt = await tx.getReceipt();
    const contract = await Contract.create(receipt.contractAddress!, abi, wallet);
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
      .send({ origin: accounts[0] });

    await tx.isMined({ interval: 0.1 });
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);
  }, 100_000);

  it('performs public nested calls', async () => {
    const tx = parentContract.methods
      .pubEntryPoint(childContract.address, Fr.fromBuffer(childContract.methods.pubValue.selector), 42n)
      .send({ origin: accounts[0] });

    await tx.isMined({ interval: 0.1 });
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);
  }, 100_000);

  it('enqueues a single public call', async () => {
    const tx = parentContract.methods
      .enqueueCallToChild(childContract.address, Fr.fromBuffer(childContract.methods.pubStoreValue.selector), 42n)
      .send({ origin: accounts[0] });

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
      .send({ origin: accounts[0] });

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
      .send({ origin: accounts[0] });

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
      .send({ origin: accounts[0] });

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
      .send({ origin: accounts[0] });

    await tx.isMined({ interval: 0.1 });
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);
    expect(await getChildStoredValue(childContract)).toEqual(85n);
  }, 100_000);
});
