import { AztecNodeService } from '@aztec/aztec-node';
import { AztecAddress, AztecRPCServer, Contract, ContractDeployer, TxStatus } from '@aztec/aztec.js';
import { ContractAbi } from '@aztec/foundation/abi';
import { DebugLogger } from '@aztec/foundation/log';
import { ChildAbi } from '@aztec/noir-contracts/examples';

import { EcdsaSignature } from '@aztec/circuits.js';
import { toBigInt } from '@aztec/foundation/serialize';
import { KeyStore } from '@aztec/key-store';
import { setup } from './utils.js';

describe('e2e_account_contract', () => {
  let aztecNode: AztecNodeService;
  let aztecRpcServer: AztecRPCServer;
  let keyStore: KeyStore;
  let logger: DebugLogger;

  let account: AztecAddress;
  let child: Contract;

  beforeEach(async () => {
    ({ aztecNode, aztecRpcServer, keyStore, logger } = await setup(1));

    account = (await aztecRpcServer.getAccounts())[0];
    child = await deployContract(ChildAbi);
  }, 60_000);

  afterEach(async () => {
    await aztecNode.stop();
    await aztecRpcServer.stop();
  });

  const deployContract = async (abi: ContractAbi) => {
    logger(`Deploying L2 contract ${abi.name}...`);
    const deployer = new ContractDeployer(abi, aztecRpcServer);
    const tx = deployer.deploy().send();

    await tx.isMined(0, 0.1);

    const receipt = await tx.getReceipt();
    const contract = new Contract(receipt.contractAddress!, abi, aztecRpcServer);
    logger(`L2 contract ${abi.name} deployed at ${contract.address}`);
    return contract;
  };

  it('calls a private function', async () => {
    const tx = child.methods.value(42).send({ from: account });

    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);
  }, 30_000);

  it('calls a public function', async () => {
    const tx = child.methods.pubStoreValue(42).send({ from: account });

    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);
    expect(toBigInt((await aztecNode.getStorageAt(child.address, 1n))!)).toEqual(42n);
  }, 30_000);

  it('rejects ecdsa signature from a different key', async () => {
    keyStore.ecdsaSign = () => Promise.resolve(EcdsaSignature.random());
    await expect(child.methods.value(42).create({ from: account })).rejects.toMatch(
      /could not satisfy all constraints/,
    );
  }, 30_000);
});
