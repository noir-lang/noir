import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer, Contract, ContractDeployer, TxStatus } from '@aztec/aztec.js';
import { ContractAbi } from '@aztec/foundation/abi';
import { DebugLogger } from '@aztec/foundation/log';
import { AccountContractAbi, ChildAbi } from '@aztec/noir-contracts/examples';

import { toBigInt } from '@aztec/foundation/serialize';
import { setup } from './utils.js';

describe('e2e_account_contract', () => {
  let aztecNode: AztecNodeService;
  let aztecRpcServer: AztecRPCServer;
  let logger: DebugLogger;

  let account: Contract;
  let child: Contract;

  const privKey = Buffer.from('ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80', 'hex');

  beforeEach(async () => {
    ({ aztecNode, aztecRpcServer, logger } = await setup());

    account = await deployContract(AccountContractAbi);
    await aztecRpcServer.addSmartAccount(privKey, account.address);

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
    const tx = child.methods.value(42).send({ from: account.address });

    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);
  }, 30_000);

  it('calls a public function', async () => {
    const tx = child.methods.pubStoreValue(42).send({ from: account.address });

    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);
    expect(toBigInt((await aztecNode.getStorageAt(child.address, 1n))!)).toEqual(42n);
  }, 30_000);

  // TODO: Reenable this test by hijacking the keystore to generate a different signature!
  // it('rejects ecdsa signature from a different key', async () => {
  //   const payload = buildPayload([callChildValue(42)], []);
  //   const call = buildCall(payload, { privKey: '2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6' });
  //   await expect(call.create({ from: accounts[0] })).rejects.toMatch(/could not satisfy all constraints/);
  // }, 30_000);
});
