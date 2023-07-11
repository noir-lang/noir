import { AztecNodeService } from '@aztec/aztec-node';
import { AztecAddress, AztecRPCServer, Contract, ContractDeployer, Fr, TxStatus, Wallet } from '@aztec/aztec.js';
import { DebugLogger } from '@aztec/foundation/log';
import { PublicTokenContractAbi } from '@aztec/noir-contracts/examples';

import { L2BlockL2Logs, LogType } from '@aztec/types';
import times from 'lodash.times';
import { expectAztecStorageSlot, pointToPublicKey, setup } from './utils.js';

describe('e2e_public_token_contract', () => {
  let aztecNode: AztecNodeService;
  let aztecRpcServer: AztecRPCServer;
  let wallet: Wallet;
  let accounts: AztecAddress[];
  let logger: DebugLogger;

  let contract: Contract;
  const balanceSlot = 1n;

  const deployContract = async () => {
    logger(`Deploying L2 public contract...`);
    const deployer = new ContractDeployer(PublicTokenContractAbi, aztecRpcServer);
    const tx = deployer.deploy().send();

    logger(`Tx sent with hash ${await tx.getTxHash()}`);
    const receipt = await tx.getReceipt();
    contract = new Contract(receipt.contractAddress!, PublicTokenContractAbi, wallet);
    await tx.isMined(0, 0.1);
    const txReceipt = await tx.getReceipt();
    expect(txReceipt.status).toEqual(TxStatus.MINED);
    logger(`L2 contract deployed at ${receipt.contractAddress}`);
    return { contract, tx, txReceipt };
  };

  const expectLogsFromLastBlockToBe = async (logMessages: string[]) => {
    const l2BlockNum = await aztecNode.getBlockHeight();
    const unencryptedLogs = await aztecNode.getLogs(l2BlockNum, 1, LogType.UNENCRYPTED);
    const unrolledLogs = L2BlockL2Logs.unrollLogs(unencryptedLogs);
    const asciiLogs = unrolledLogs.map(log => log.toString('ascii'));

    expect(asciiLogs).toStrictEqual(logMessages);
  };

  beforeEach(async () => {
    ({ aztecNode, aztecRpcServer, accounts, wallet, logger } = await setup());
  }, 100_000);

  afterEach(async () => {
    await aztecNode.stop();
    await aztecRpcServer.stop();
  });

  it('should deploy a public token contract', async () => {
    const { txReceipt } = await deployContract();
    expect(txReceipt.status).toEqual(TxStatus.MINED);
  }, 30_000);

  it('should deploy a public token contract and mint tokens to a recipient', async () => {
    const mintAmount = 359n;

    const recipientIdx = 0;

    const recipient = accounts[recipientIdx];
    const { contract: deployedContract } = await deployContract();

    const PK = await aztecRpcServer.getAccountPublicKey(recipient);

    const tx = deployedContract.methods.mint(mintAmount, pointToPublicKey(PK)).send({ from: recipient });

    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);
    await expectAztecStorageSlot(logger, aztecNode, contract, balanceSlot, Fr.fromBuffer(PK.x.toBuffer()), mintAmount);
    await expectLogsFromLastBlockToBe(['Coins minted']);
  }, 45_000);

  // Regression for https://github.com/AztecProtocol/aztec-packages/issues/640
  it('should mint tokens thrice to a recipient within the same block', async () => {
    const mintAmount = 42n;
    const recipientIdx = 0;
    const recipient = accounts[recipientIdx];

    const PK = await aztecRpcServer.getAccountPublicKey(recipient);
    const { contract: deployedContract } = await deployContract();

    // Assemble two mint txs sequentially (no parallel calls to circuits!) and send them simultaneously
    const methods = times(3, () => deployedContract.methods.mint(mintAmount, pointToPublicKey(PK)));
    for (const method of methods) await method.simulate({ from: recipient });
    const txs = await Promise.all(methods.map(method => method.send()));

    // Check that all txs got mined in the same block
    await Promise.all(txs.map(tx => tx.isMined()));
    const receipts = await Promise.all(txs.map(tx => tx.getReceipt()));
    expect(receipts.map(r => r.status)).toEqual(times(3, () => TxStatus.MINED));
    expect(receipts.map(r => r.blockNumber)).toEqual(times(3, () => receipts[0].blockNumber));

    await expectAztecStorageSlot(
      logger,
      aztecNode,
      contract,
      balanceSlot,
      Fr.fromBuffer(PK.x.toBuffer()),
      mintAmount * 3n,
    );
    await expectLogsFromLastBlockToBe(['Coins minted', 'Coins minted', 'Coins minted']);
  }, 60_000);
});
