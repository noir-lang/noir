import { Archiver } from '@aztec/archiver';
import { AztecNodeConfig, AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer } from '@aztec/aztec-rpc';
import { AztecAddress, AztecRPC, CompleteAddress, Wallet, computeMessageSecretHash } from '@aztec/aztec.js';
import { DeployL1Contracts } from '@aztec/ethereum';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { DebugLogger } from '@aztec/foundation/log';
import { NonNativeTokenContract } from '@aztec/noir-contracts/types';

import { Chain, HttpTransport, PublicClient } from 'viem';

import { delay, deployAndInitializeNonNativeL2TokenContracts, setNextBlockTimestamp, setup } from './fixtures/utils.js';

describe('archiver integration with l1 to l2 messages', () => {
  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServer: AztecRPC;
  let wallet: Wallet;
  let archiver: Archiver;
  let logger: DebugLogger;
  let config: AztecNodeConfig;

  let l2Contract: NonNativeTokenContract;
  let ethAccount: EthAddress;

  let tokenPortalAddress: EthAddress;
  let tokenPortal: any;
  let underlyingERC20: any;
  let publicClient: PublicClient<HttpTransport, Chain>;

  const initialBalance = 10n;
  let owner: AztecAddress;
  let receiver: AztecAddress;

  beforeEach(async () => {
    let deployL1ContractsValues: DeployL1Contracts | undefined;
    let accounts: CompleteAddress[];
    ({ aztecNode, aztecRpcServer, wallet, deployL1ContractsValues, accounts, config, logger } = await setup(2));
    archiver = await Archiver.createAndSync(config);

    const walletClient = deployL1ContractsValues.walletClient;
    publicClient = deployL1ContractsValues.publicClient;

    ethAccount = EthAddress.fromString((await walletClient.getAddresses())[0]);
    owner = accounts[0].address;
    receiver = accounts[1].address;

    // Deploy and initialize all required contracts
    logger('Deploying Portal, initializing and deploying l2 contract...');
    const contracts = await deployAndInitializeNonNativeL2TokenContracts(
      wallet,
      walletClient,
      publicClient,
      deployL1ContractsValues!.registryAddress,
      initialBalance,
      owner,
    );
    l2Contract = contracts.l2Contract;
    underlyingERC20 = contracts.underlyingERC20;
    tokenPortal = contracts.tokenPortal;
    tokenPortalAddress = contracts.tokenPortalAddress;
    await expectBalance(owner, initialBalance);
    logger('Successfully deployed contracts and initialized portal');
  }, 100_000);

  afterEach(async () => {
    await archiver.stop();
    await aztecNode?.stop();
    if (aztecRpcServer instanceof AztecRPCServer) {
      await aztecRpcServer?.stop();
    }
  }, 30_000);

  const expectBalance = async (owner: AztecAddress, expectedBalance: bigint) => {
    const balance = await l2Contract.methods.getBalance(owner).view({ from: owner });
    logger(`Account ${owner} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  it('cancelled l1 to l2 messages cannot be consumed by archiver', async () => {
    // create a message, then cancel it

    // Generate a claim secret using pedersen
    logger("Generating a claim secret using pedersen's hash function");
    const secret = Fr.random();
    const secretHash = await computeMessageSecretHash(secret);
    const secretString = `0x${secretHash.toBuffer().toString('hex')}` as `0x${string}`;
    logger('Generated claim secret: ', secretString);

    logger('Minting tokens on L1');
    await underlyingERC20.write.mint([ethAccount.toString(), 1000000n], {} as any);
    await underlyingERC20.write.approve([tokenPortalAddress.toString(), 1000n], {} as any);

    expect(await underlyingERC20.read.balanceOf([ethAccount.toString()])).toBe(1000000n);

    // Deposit tokens to the TokenPortal
    const deadline = Number((await publicClient.getBlock()).timestamp + 1000n);
    const mintAmount = 100n;

    logger('Sending messages to L1 portal');
    const args = [owner.toString(), mintAmount, deadline, secretString, ethAccount.toString()] as const;
    await tokenPortal.write.depositToAztec(args, {} as any);
    expect(await underlyingERC20.read.balanceOf([ethAccount.toString()])).toBe(1000000n - mintAmount);

    // Wait for the archiver to process the message
    await delay(5000); /// waiting 5 seconds.

    // set the block timestamp to be after the deadline (so we can cancel the message)
    await setNextBlockTimestamp(config.rpcUrl, deadline + 1);

    // cancel the message
    logger('cancelling the l1 to l2 message');
    const argsCancel = [owner.toString(), 100n, deadline, secretString, 0n] as const;
    await tokenPortal.write.cancelL1ToAztecMessage(argsCancel, { gas: 1_000_000n } as any);
    expect(await underlyingERC20.read.balanceOf([ethAccount.toString()])).toBe(1000000n);
    // let archiver sync up
    await delay(5000);

    // archiver shouldn't have any pending messages.
    expect((await archiver.getPendingL1ToL2Messages(10)).length).toEqual(0);
  }, 80_000);

  it('archiver handles l1 to l2 message correctly even when l2block has no such messages', async () => {
    // send a transfer tx to force through rollup with the message included
    const transferAmount = 1n;
    l2Contract.methods.transfer(transferAmount, owner, receiver).send({ origin: owner });

    expect((await archiver.getPendingL1ToL2Messages(10)).length).toEqual(0);
    expect(() => archiver.getConfirmedL1ToL2Message(Fr.ZERO)).toThrow();
  });
});
