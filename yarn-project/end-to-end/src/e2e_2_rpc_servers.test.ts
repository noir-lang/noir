import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer, EthAddress, Fr } from '@aztec/aztec-rpc';
import { AztecAddress, Wallet, computeMessageSecretHash } from '@aztec/aztec.js';
import { DebugLogger } from '@aztec/foundation/log';
import { retryUntil } from '@aztec/foundation/retry';
import { toBigInt } from '@aztec/foundation/serialize';
import { ChildContract, TokenContract } from '@aztec/noir-contracts/types';
import { AztecRPC, CompleteAddress, TxStatus } from '@aztec/types';

import { jest } from '@jest/globals';

import { expectsNumOfEncryptedLogsInTheLastBlockToBe, setup, setupAztecRPCServer } from './fixtures/utils.js';

const { SANDBOX_URL = '' } = process.env;

const TIMEOUT = 60_000;

describe('e2e_2_rpc_servers', () => {
  jest.setTimeout(TIMEOUT);

  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServerA: AztecRPC;
  let aztecRpcServerB: AztecRPC;
  let walletA: Wallet;
  let walletB: Wallet;
  let userA: CompleteAddress;
  let userB: CompleteAddress;
  let logger: DebugLogger;
  let teardownA: () => Promise<void>;

  beforeEach(async () => {
    // this test can't be run against the sandbox as it requires 2 RPC servers
    if (SANDBOX_URL) {
      throw new Error(`Test can't be run against the sandbox as 2 RPC servers are required`);
    }
    let accounts: CompleteAddress[] = [];
    ({
      aztecNode,
      aztecRpcServer: aztecRpcServerA,
      accounts,
      wallets: [walletA],
      logger,
      teardown: teardownA,
    } = await setup(1));
    [userA] = accounts;

    ({
      aztecRpcServer: aztecRpcServerB,
      accounts: accounts,
      wallets: [walletB],
    } = await setupAztecRPCServer(1, aztecNode!, undefined, true));
    [userB] = accounts;
  }, 100_000);

  afterEach(async () => {
    await teardownA();
    if (aztecRpcServerB instanceof AztecRPCServer) await aztecRpcServerB.stop();
  });

  const awaitUserSynchronized = async (wallet: Wallet, owner: AztecAddress) => {
    const isUserSynchronized = async () => {
      return await wallet.isAccountStateSynchronized(owner);
    };
    await retryUntil(isUserSynchronized, `synch of user ${owner.toString()}`, 10);
  };

  const expectTokenBalance = async (
    wallet: Wallet,
    tokenAddress: AztecAddress,
    owner: AztecAddress,
    expectedBalance: bigint,
    checkIfSynchronized = true,
  ) => {
    if (checkIfSynchronized) {
      // First wait until the corresponding RPC server has synchronized the account
      await awaitUserSynchronized(wallet, owner);
    }

    // Then check the balance
    const contractWithWallet = await TokenContract.at(tokenAddress, wallet);
    const balance = await contractWithWallet.methods.balance_of_private(owner).view({ from: owner });
    logger(`Account ${owner} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  const deployTokenContract = async (initialAdminBalance: bigint, admin: AztecAddress) => {
    logger(`Deploying Token contract...`);
    const contract = await TokenContract.deploy(walletA).send().deployed();
    expect((await contract.methods._initialize(admin).send().wait()).status).toBe(TxStatus.MINED);

    if (initialAdminBalance > 0n) {
      await mintTokens(contract, admin, initialAdminBalance);
    }

    logger('L2 contract deployed');

    return contract.completeAddress;
  };

  const mintTokens = async (contract: TokenContract, recipient: AztecAddress, balance: bigint) => {
    const secret = Fr.random();
    const secretHash = await computeMessageSecretHash(secret);

    expect((await contract.methods.mint_private(balance, secretHash).send().wait()).status).toEqual(TxStatus.MINED);
    expect((await contract.methods.redeem_shield(recipient, balance, secret).send().wait()).status).toEqual(
      TxStatus.MINED,
    );
  };

  it('transfers fund from user A to B via RPC server A followed by transfer from B to A via RPC server B', async () => {
    const initialBalance = 987n;
    const transferAmount1 = 654n;
    const transferAmount2 = 323n;

    const completeTokenAddress = await deployTokenContract(initialBalance, userA.address);
    const tokenAddress = completeTokenAddress.address;

    // Add account B to wallet A
    await aztecRpcServerA.registerRecipient(userB);
    // Add account A to wallet B
    await aztecRpcServerB.registerRecipient(userA);

    // Add token to RPC server B (RPC server A already has it because it was deployed through it)
    await aztecRpcServerB.addContracts([
      {
        abi: TokenContract.abi,
        completeAddress: completeTokenAddress,
        portalContract: EthAddress.ZERO,
      },
    ]);

    // Check initial balances and logs are as expected
    await expectTokenBalance(walletA, tokenAddress, userA.address, initialBalance);
    await expectTokenBalance(walletB, tokenAddress, userB.address, 0n);
    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 1);

    // Transfer funds from A to B via RPC server A
    const contractWithWalletA = await TokenContract.at(tokenAddress, walletA);
    const receiptAToB = await contractWithWalletA.methods
      .transfer(userA.address, userB.address, transferAmount1, 0)
      .send()
      .wait();
    expect(receiptAToB.status).toBe(TxStatus.MINED);

    // Check balances and logs are as expected
    await expectTokenBalance(walletA, tokenAddress, userA.address, initialBalance - transferAmount1);
    await expectTokenBalance(walletB, tokenAddress, userB.address, transferAmount1);
    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 2);

    // Transfer funds from B to A via RPC server B
    const contractWithWalletB = await TokenContract.at(tokenAddress, walletB);
    await contractWithWalletB.methods
      .transfer(userB.address, userA.address, transferAmount2, 0)
      .send()
      .wait({ interval: 0.1 });

    // Check balances and logs are as expected
    await expectTokenBalance(walletA, tokenAddress, userA.address, initialBalance - transferAmount1 + transferAmount2);
    await expectTokenBalance(walletB, tokenAddress, userB.address, transferAmount1 - transferAmount2);
    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 2);
  }, 120_000);

  const deployChildContractViaServerA = async () => {
    logger(`Deploying Child contract...`);
    const contract = await ChildContract.deploy(walletA).send().deployed();
    logger('Child contract deployed');

    return contract.completeAddress;
  };

  const awaitServerSynchronized = async (server: AztecRPC) => {
    const isServerSynchronized = async () => {
      return await server.isGlobalStateSynchronized();
    };
    await retryUntil(isServerSynchronized, 'server sync', 10);
  };

  const getChildStoredValue = (child: { address: AztecAddress }, aztecRpcServer: AztecRPC) =>
    aztecRpcServer.getPublicStorageAt(child.address, new Fr(1)).then(x => toBigInt(x!));

  it('user calls a public function on a contract deployed by a different user using a different RPC server', async () => {
    const childCompleteAddress = await deployChildContractViaServerA();

    await awaitServerSynchronized(aztecRpcServerA);

    // Add Child to RPC server B
    await aztecRpcServerB.addContracts([
      {
        abi: ChildContract.abi,
        completeAddress: childCompleteAddress,
        portalContract: EthAddress.ZERO,
      },
    ]);

    const newValueToSet = 256n;

    const childContractWithWalletB = await ChildContract.at(childCompleteAddress.address, walletB);
    await childContractWithWalletB.methods.pubIncValue(newValueToSet).send().wait({ interval: 0.1 });

    await awaitServerSynchronized(aztecRpcServerA);

    const storedValue = await getChildStoredValue(childCompleteAddress, aztecRpcServerB);
    expect(storedValue).toBe(newValueToSet);
  });

  it('private state is "zero" when Aztec RPC Server does not have the account private key', async () => {
    const userABalance = 100n;
    const userBBalance = 150n;

    const completeTokenAddress = await deployTokenContract(userABalance, userA.address);
    const contractWithWalletA = await TokenContract.at(completeTokenAddress.address, walletA);

    // Add account B to wallet A
    await aztecRpcServerA.registerRecipient(userB);
    // Add account A to wallet B
    await aztecRpcServerB.registerRecipient(userA);

    // Add token to RPC server B (RPC server A already has it because it was deployed through it)
    await aztecRpcServerB.addContracts([
      {
        abi: TokenContract.abi,
        completeAddress: completeTokenAddress,
        portalContract: EthAddress.ZERO,
      },
    ]);

    // Mint tokens to user B
    await mintTokens(contractWithWalletA, userB.address, userBBalance);

    // Check that user A balance is 100 on server A
    await expectTokenBalance(walletA, completeTokenAddress.address, userA.address, userABalance);
    // Check that user B balance is 150 on server B
    await expectTokenBalance(walletB, completeTokenAddress.address, userB.address, userBBalance);

    // CHECK THAT PRIVATE BALANCES ARE 0 WHEN ACCOUNT'S PRIVATE KEYS ARE NOT REGISTERED
    // Note: Not checking if the account is synchronized because it is not registered as an account (it would throw).
    const checkIfSynchronized = false;
    // Check that user A balance is 0 on server B
    await expectTokenBalance(walletB, completeTokenAddress.address, userA.address, 0n, checkIfSynchronized);
    // Check that user B balance is 0 on server A
    await expectTokenBalance(walletA, completeTokenAddress.address, userB.address, 0n, checkIfSynchronized);
  });
});
