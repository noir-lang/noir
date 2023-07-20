import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer, EthAddress } from '@aztec/aztec-rpc';
import { AztecAddress, Wallet } from '@aztec/aztec.js';
import { DebugLogger } from '@aztec/foundation/log';
import { retryUntil } from '@aztec/foundation/retry';
import { ZkTokenContract } from '@aztec/noir-contracts/types';
import { L2BlockL2Logs, LogType, TxStatus } from '@aztec/types';

import { setup, setupAztecRPCServer } from './utils.js';

describe('e2e_2_rpc_servers', () => {
  let aztecNode: AztecNodeService;
  let aztecRpcServerA: AztecRPCServer;
  let aztecRpcServerB: AztecRPCServer;
  let walletA: Wallet;
  let walletB: Wallet;
  let userA: AztecAddress;
  let userB: AztecAddress;
  let logger: DebugLogger;

  let contractWithWalletA: ZkTokenContract;
  let contractWithWalletB: ZkTokenContract;

  const initialBalance = 987n;
  const transferAmount1 = 654n;
  const transferAmount2 = 323n;

  beforeEach(async () => {
    let accounts: AztecAddress[] = [];
    ({ aztecNode, aztecRpcServer: aztecRpcServerA, accounts, wallet: walletA, logger } = await setup(1));
    [userA] = accounts;

    ({
      aztecRpcServer: aztecRpcServerB,
      accounts: accounts,
      wallet: walletB,
    } = await setupAztecRPCServer(1, aztecNode));
    [userB] = accounts;

    logger(`Deploying L2 contract...`);
    const tx = ZkTokenContract.deploy(aztecRpcServerA, initialBalance, userA).send();
    const receipt = await tx.getReceipt();
    contractWithWalletA = new ZkTokenContract(receipt.contractAddress!, walletA);
    contractWithWalletB = new ZkTokenContract(receipt.contractAddress!, walletB);

    await tx.isMined(0, 0.1);
    const minedReceipt = await tx.getReceipt();
    expect(minedReceipt.status).toEqual(TxStatus.MINED);
    logger('L2 contract deployed');
  }, 100_000);

  afterEach(async () => {
    await aztecNode?.stop();
    await aztecRpcServerA?.stop();
    await aztecRpcServerB?.stop();
  });

  const expectBalance = async (wallet: Wallet, owner: AztecAddress, expectedBalance: bigint) => {
    // First wait until the corresponding RPC server has synchronised the account
    const isUserSynchronised = async () => {
      return await wallet.isAccountSynchronised(owner);
    };
    await retryUntil(isUserSynchronised, owner.toString(), 5);

    // Then check the balance
    const contractWithWallet = new ZkTokenContract(contractWithWalletA.address, wallet);
    const [balance] = await contractWithWallet.methods.getBalance(owner).view({ from: owner });
    logger(`Account ${owner} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  const expectsNumOfEncryptedLogsInTheLastBlockToBe = async (numEncryptedLogs: number) => {
    const l2BlockNum = await aztecNode.getBlockHeight();
    const encryptedLogs = await aztecNode.getLogs(l2BlockNum, 1, LogType.ENCRYPTED);
    const unrolledLogs = L2BlockL2Logs.unrollLogs(encryptedLogs);
    expect(unrolledLogs.length).toBe(numEncryptedLogs);
  };

  const expectUnencryptedLogsFromLastBlockToBe = async (logMessages: string[]) => {
    const l2BlockNum = await aztecNode.getBlockHeight();
    const unencryptedLogs = await aztecNode.getLogs(l2BlockNum, 1, LogType.UNENCRYPTED);
    const unrolledLogs = L2BlockL2Logs.unrollLogs(unencryptedLogs);
    const asciiLogs = unrolledLogs.map(log => log.toString('ascii'));

    expect(asciiLogs).toStrictEqual(logMessages);
  };

  it('transfers fund from user A to B via RPC Server A followed by transfer from B to A via RPC Server B', async () => {
    // Add account B pub key and partial address to wallet A
    const [accountBPubKey, accountBPartialAddress] = await aztecRpcServerB.getPublicKeyAndPartialAddress(userB);
    await aztecRpcServerA.addPublicKeyAndPartialAddress(userB, accountBPubKey, accountBPartialAddress);
    // Add account A pub key and partial address to wallet B
    const [accountAPubKey, accountAPartialAddress] = await aztecRpcServerA.getPublicKeyAndPartialAddress(userA);
    await aztecRpcServerB.addPublicKeyAndPartialAddress(userA, accountAPubKey, accountAPartialAddress);

    // Add zkToken to rpc server B
    await aztecRpcServerB.addContracts([
      {
        abi: ZkTokenContract.abi,
        address: contractWithWalletA.address,
        portalContract: EthAddress.ZERO,
      },
    ]);

    // Check initial balances and logs are as expected
    await expectBalance(walletA, userA, initialBalance);
    await expectBalance(walletB, userB, 0n);
    await expectsNumOfEncryptedLogsInTheLastBlockToBe(1);
    await expectUnencryptedLogsFromLastBlockToBe(['Balance set in constructor']);

    // Transfer funds from A to B via rpc server A
    const txAToB = contractWithWalletA.methods.transfer(transferAmount1, userA, userB).send({ origin: userA });

    await txAToB.isMined(0, 0.1);
    const receiptAToB = await txAToB.getReceipt();

    expect(receiptAToB.status).toBe(TxStatus.MINED);

    // Check balances and logs are as expected
    await expectBalance(walletA, userA, initialBalance - transferAmount1);
    await expectBalance(walletB, userB, transferAmount1);
    await expectsNumOfEncryptedLogsInTheLastBlockToBe(2);
    await expectUnencryptedLogsFromLastBlockToBe(['Coins transferred']);

    // Transfer funds from B to A via rpc server B
    const txBToA = contractWithWalletB.methods.transfer(transferAmount2, userB, userA).send({ origin: userB });

    await txBToA.isMined(0, 0.1);
    const receiptBToA = await txBToA.getReceipt();

    expect(receiptBToA.status).toBe(TxStatus.MINED);

    // Check balances and logs are as expected
    await expectBalance(walletA, userA, initialBalance - transferAmount1 + transferAmount2);
    await expectBalance(walletB, userB, transferAmount1 - transferAmount2);
    await expectsNumOfEncryptedLogsInTheLastBlockToBe(2);
    await expectUnencryptedLogsFromLastBlockToBe(['Coins transferred']);
  }, 120_000);
});
