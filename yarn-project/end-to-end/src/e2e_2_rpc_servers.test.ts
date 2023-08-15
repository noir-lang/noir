import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer, EthAddress, Fr } from '@aztec/aztec-rpc';
import { AztecAddress, Wallet } from '@aztec/aztec.js';
import { DebugLogger } from '@aztec/foundation/log';
import { retryUntil } from '@aztec/foundation/retry';
import { toBigInt } from '@aztec/foundation/serialize';
import { ChildContract, PrivateTokenContract } from '@aztec/noir-contracts/types';
import { AztecRPC, CompleteAddress, TxStatus } from '@aztec/types';

import {
  expectUnencryptedLogsFromLastBlockToBe,
  expectsNumOfEncryptedLogsInTheLastBlockToBe,
  setup,
  setupAztecRPCServer,
} from './fixtures/utils.js';

const { SANDBOX_URL = '' } = process.env;

describe('e2e_2_rpc_servers', () => {
  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServerA: AztecRPC;
  let aztecRpcServerB: AztecRPC;
  let walletA: Wallet;
  let walletB: Wallet;
  let userA: CompleteAddress;
  let userB: CompleteAddress;
  let logger: DebugLogger;

  beforeEach(async () => {
    // this test can't be run against the sandbox as it requires 2 RPC servers
    if (SANDBOX_URL) {
      throw new Error(`Test can't be run against the sandbox as 2 RPC servers are required`);
    }
    let accounts: CompleteAddress[] = [];
    ({ aztecNode, aztecRpcServer: aztecRpcServerA, accounts, wallet: walletA, logger } = await setup(1));
    [userA] = accounts;

    ({
      aztecRpcServer: aztecRpcServerB,
      accounts: accounts,
      wallet: walletB,
    } = await setupAztecRPCServer(1, aztecNode!, null, undefined, true));
    [userB] = accounts;
  }, 100_000);

  afterEach(async () => {
    await aztecNode?.stop();
    if (aztecRpcServerA instanceof AztecRPCServer) {
      await aztecRpcServerA?.stop();
    }
    if (aztecRpcServerB instanceof AztecRPCServer) {
      await aztecRpcServerB?.stop();
    }
  });

  const awaitUserSynchronised = async (wallet: Wallet, owner: AztecAddress) => {
    const isUserSynchronised = async () => {
      return await wallet.isAccountStateSynchronised(owner);
    };
    await retryUntil(isUserSynchronised, `synch of user ${owner.toString()}`, 10);
  };

  const expectTokenBalance = async (
    wallet: Wallet,
    tokenAddress: AztecAddress,
    owner: AztecAddress,
    expectedBalance: bigint,
  ) => {
    // First wait until the corresponding RPC server has synchronised the account
    await awaitUserSynchronised(wallet, owner);

    // Then check the balance
    const contractWithWallet = await PrivateTokenContract.create(tokenAddress, wallet);
    const balance = await contractWithWallet.methods.getBalance(owner).view({ from: owner });
    logger(`Account ${owner} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  const deployPrivateTokenContract = async (initialBalance: bigint, owner: AztecAddress) => {
    logger(`Deploying PrivateToken contract...`);
    const tx = PrivateTokenContract.deploy(aztecRpcServerA, initialBalance, owner).send();
    const receipt = await tx.getReceipt();
    await tx.isMined({ interval: 0.1 });
    const minedReceipt = await tx.getReceipt();
    expect(minedReceipt.status).toEqual(TxStatus.MINED);
    logger('L2 contract deployed');

    return receipt.contractAddress!;
  };

  it('transfers fund from user A to B via RPC server A followed by transfer from B to A via RPC server B', async () => {
    const initialBalance = 987n;
    const transferAmount1 = 654n;
    const transferAmount2 = 323n;

    const tokenAddress = await deployPrivateTokenContract(initialBalance, userA.address);

    // Add account B to wallet A
    await aztecRpcServerA.registerRecipient(userB);
    // Add account A to wallet B
    await aztecRpcServerB.registerRecipient(userA);

    // Add privateToken to RPC server B
    await aztecRpcServerB.addContracts([
      {
        abi: PrivateTokenContract.abi,
        address: tokenAddress,
        portalContract: EthAddress.ZERO,
      },
    ]);

    // Check initial balances and logs are as expected
    await expectTokenBalance(walletA, tokenAddress, userA.address, initialBalance);
    await expectTokenBalance(walletB, tokenAddress, userB.address, 0n);
    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 1);
    await expectUnencryptedLogsFromLastBlockToBe(aztecNode, ['Balance set in constructor']);

    // Transfer funds from A to B via RPC server A
    const contractWithWalletA = await PrivateTokenContract.create(tokenAddress, walletA);
    const txAToB = contractWithWalletA.methods
      .transfer(transferAmount1, userA.address, userB.address)
      .send({ origin: userA.address });

    await txAToB.isMined({ interval: 0.1 });
    const receiptAToB = await txAToB.getReceipt();

    expect(receiptAToB.status).toBe(TxStatus.MINED);

    // Check balances and logs are as expected
    await expectTokenBalance(walletA, tokenAddress, userA.address, initialBalance - transferAmount1);
    await expectTokenBalance(walletB, tokenAddress, userB.address, transferAmount1);
    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 2);
    await expectUnencryptedLogsFromLastBlockToBe(aztecNode, ['Coins transferred']);

    // Transfer funds from B to A via RPC server B
    const contractWithWalletB = await PrivateTokenContract.create(tokenAddress, walletB);
    const txBToA = contractWithWalletB.methods
      .transfer(transferAmount2, userB.address, userA.address)
      .send({ origin: userB.address });

    await txBToA.isMined({ interval: 0.1 });
    const receiptBToA = await txBToA.getReceipt();

    expect(receiptBToA.status).toBe(TxStatus.MINED);

    // Check balances and logs are as expected
    await expectTokenBalance(walletA, tokenAddress, userA.address, initialBalance - transferAmount1 + transferAmount2);
    await expectTokenBalance(walletB, tokenAddress, userB.address, transferAmount1 - transferAmount2);
    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 2);
    await expectUnencryptedLogsFromLastBlockToBe(aztecNode, ['Coins transferred']);
  }, 120_000);

  const deployChildContractViaServerA = async () => {
    logger(`Deploying Child contract...`);
    const tx = ChildContract.deploy(aztecRpcServerA).send();
    const receipt = await tx.getReceipt();
    await tx.isMined({ interval: 0.1 });
    const minedReceipt = await tx.getReceipt();
    expect(minedReceipt.status).toEqual(TxStatus.MINED);
    logger('Child contract deployed');

    return receipt.contractAddress!;
  };

  const awaitServerSynchronised = async (server: AztecRPC) => {
    const isServerSynchronised = async () => {
      return await server.isGlobalStateSynchronised();
    };
    await retryUntil(isServerSynchronised, 'server sync', 10);
  };

  const getChildStoredValue = (child: { address: AztecAddress }, aztecRpcServer: AztecRPC) =>
    aztecRpcServer.getPublicStorageAt(child.address, new Fr(1)).then(x => toBigInt(x!));

  it('user calls a public function on a contract deployed by a different user using a different RPC server', async () => {
    const childAddress = await deployChildContractViaServerA();

    await awaitServerSynchronised(aztecRpcServerA);

    // Add Child to RPC server B
    await aztecRpcServerB.addContracts([
      {
        abi: ChildContract.abi,
        address: childAddress,
        portalContract: EthAddress.ZERO,
      },
    ]);

    const newValueToSet = 256n;

    const childContractWithWalletB = await ChildContract.create(childAddress, walletB);
    const tx = childContractWithWalletB.methods.pubStoreValue(newValueToSet).send({ origin: userB.address });
    await tx.isMined({ interval: 0.1 });

    const receipt = await tx.getReceipt();
    expect(receipt.status).toBe(TxStatus.MINED);

    await awaitServerSynchronised(aztecRpcServerA);

    const storedValue = await getChildStoredValue({ address: childAddress }, aztecRpcServerB);
    expect(storedValue).toBe(newValueToSet);
  }, 60_000);
});
