import { AztecNodeService } from '@aztec/aztec-node';
import { AztecAddress, AztecRPCServer, Contract, TxStatus, computeMessageSecretHash } from '@aztec/aztec.js';
import { EthAddress } from '@aztec/foundation/eth-address';

import { DeployL1Contracts } from '@aztec/ethereum';
import { toBigIntBE, toBufferBE } from '@aztec/foundation/bigint-buffer';
import { Fr } from '@aztec/foundation/fields';
import { DebugLogger } from '@aztec/foundation/log';
import { OutboxAbi } from '@aztec/l1-artifacts';
import { Chain, HttpTransport, PublicClient, getContract } from 'viem';
import { delay, deployAndInitializeNonNativeL2TokenContracts, pointToPublicKey, setup } from './utils.js';
import { sha256 } from '@aztec/foundation/crypto';

const sha256ToField = (buf: Buffer): Fr => {
  const tempContent = toBigIntBE(sha256(buf));
  return Fr.fromBuffer(toBufferBE(tempContent % Fr.MODULUS, 32));
};

describe('e2e_cross_chain_messaging', () => {
  let aztecNode: AztecNodeService;
  let aztecRpcServer: AztecRPCServer;
  let accounts: AztecAddress[];
  let logger: DebugLogger;

  let l2Contract: Contract;
  let ethAccount: EthAddress;

  let tokenPortalAddress: EthAddress;
  let tokenPortal: any;
  let underlyingERC20: any;
  let outbox: any;
  let publicClient: PublicClient<HttpTransport, Chain>;
  let walletClient: any;

  const initialBalance = 10n;
  let ownerAddress: AztecAddress;
  let receiver: AztecAddress;
  let ownerPub: { x: bigint; y: bigint };

  beforeEach(async () => {
    let deployL1ContractsValues: DeployL1Contracts;
    ({ aztecNode, aztecRpcServer, deployL1ContractsValues, accounts, logger } = await setup(2));

    walletClient = deployL1ContractsValues.walletClient;
    publicClient = deployL1ContractsValues.publicClient;

    ethAccount = EthAddress.fromString((await walletClient.getAddresses())[0]);
    [ownerAddress, receiver] = accounts;
    ownerPub = pointToPublicKey(await aztecRpcServer.getAccountPublicKey(ownerAddress));

    outbox = getContract({
      address: deployL1ContractsValues.outboxAddress.toString(),
      abi: OutboxAbi,
      publicClient,
    });

    // Deploy and initialize all required contracts
    logger('Deploying Portal, initializing and deploying l2 contract...');
    const contracts = await deployAndInitializeNonNativeL2TokenContracts(
      aztecRpcServer,
      walletClient,
      publicClient,
      deployL1ContractsValues!.registryAddress,
      initialBalance,
      ownerPub,
    );
    l2Contract = contracts.l2Contract;
    underlyingERC20 = contracts.underlyingERC20;
    tokenPortal = contracts.tokenPortal;
    tokenPortalAddress = contracts.tokenPortalAddress;
    await expectBalance(accounts[0], initialBalance);
    logger('Successfully deployed contracts and initialized portal');
  }, 30_000);

  afterEach(async () => {
    await aztecNode?.stop();
    await aztecRpcServer?.stop();
  });

  const expectBalance = async (owner: AztecAddress, expectedBalance: bigint) => {
    const ownerPublicKey = await aztecRpcServer.getAccountPublicKey(owner);
    const [balance] = await l2Contract.methods.getBalance(pointToPublicKey(ownerPublicKey)).view({ from: owner });
    logger(`Account ${owner} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  it('Milestone 2: Deposit funds from L1 -> L2 and withdraw back to L1', async () => {
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
    const deadline = 2 ** 32 - 1; // max uint32 - 1

    const mintAmount = 100n;

    logger('Sending messages to L1 portal');
    const args = [ownerAddress.toString(), mintAmount, deadline, secretString] as const;
    const { result: messageKeyHex } = await tokenPortal.simulate.depositToAztec(args, {
      account: ethAccount.toString(),
    } as any);
    await tokenPortal.write.depositToAztec(args, {} as any);
    expect(await underlyingERC20.read.balanceOf([ethAccount.toString()])).toBe(1000000n - mintAmount);

    const messageKey = Fr.fromString(messageKeyHex);

    // Wait for the archiver to process the message
    await delay(5000); /// waiting 5 seconds.

    // send a transfer tx to force through rollup with the message included
    const transferAmount = 1n;
    const transferTx = l2Contract.methods
      .transfer(
        transferAmount,
        pointToPublicKey(await aztecRpcServer.getAccountPublicKey(ownerAddress)),
        pointToPublicKey(await aztecRpcServer.getAccountPublicKey(receiver)),
      )
      .send({ from: accounts[0] });

    await transferTx.isMined(0, 0.1);
    const transferReceipt = await transferTx.getReceipt();

    expect(transferReceipt.status).toBe(TxStatus.MINED);

    logger('Consuming messages on L2');
    // Call the mint tokens function on the noir contract

    const consumptionTx = l2Contract.methods
      .mint(mintAmount, ownerPub, messageKey, secret)
      .send({ from: ownerAddress });

    await consumptionTx.isMined(0, 0.1);
    const consumptionReceipt = await consumptionTx.getReceipt();

    expect(consumptionReceipt.status).toBe(TxStatus.MINED);
    await expectBalance(ownerAddress, mintAmount + initialBalance - transferAmount);

    // time to withdraw the funds again!
    const withdrawAmount = 9n;

    logger('Withdrawing funds from L2');

    logger('Ensure that the entry is not in outbox yet');
    const contractInfo = await aztecNode.getContractInfo(l2Contract.address);
    // 0x00f714ce, selector for "withdraw(uint256,address)"
    const content = sha256ToField(
      Buffer.concat([Buffer.from([0x00, 0xf7, 0x14, 0xce]), toBufferBE(withdrawAmount, 32), ethAccount.toBuffer32()]),
    );
    const entryKey = sha256ToField(
      Buffer.concat([
        l2Contract.address.toBuffer(),
        new Fr(1).toBuffer(), // aztec version
        contractInfo?.portalContractAddress.toBuffer32() ?? Buffer.alloc(32, 0),
        new Fr(publicClient.chain.id).toBuffer(), // chain id
        content.toBuffer(),
      ]),
    );
    expect(await outbox.read.contains([entryKey.toString(true)])).toBeFalsy();

    logger('Send L2 tx to withdraw funds');
    const withdrawTx = l2Contract.methods.withdraw(withdrawAmount, ownerPub, ethAccount).send({ from: ownerAddress });

    await withdrawTx.isMined(0, 0.1);
    const withdrawReceipt = await withdrawTx.getReceipt();

    expect(withdrawReceipt.status).toBe(TxStatus.MINED);
    await expectBalance(ownerAddress, mintAmount + initialBalance - transferAmount - withdrawAmount);
    expect(await underlyingERC20.read.balanceOf([ethAccount.toString()])).toBe(1000000n - mintAmount);

    logger('Send L1 tx to consume entry and withdraw funds');
    // Call function on L1 contract to consume the message
    const { request: withdrawRequest, result: withdrawEntryKey } = await tokenPortal.simulate.withdraw([
      withdrawAmount,
      ethAccount.toString(),
    ]);

    expect(withdrawEntryKey).toBe(entryKey.toString(true));

    expect(await outbox.read.contains([withdrawEntryKey])).toBeTruthy();

    await walletClient.writeContract(withdrawRequest);
    expect(await underlyingERC20.read.balanceOf([ethAccount.toString()])).toBe(1000000n - mintAmount + withdrawAmount);

    expect(await outbox.read.contains([withdrawEntryKey])).toBeFalsy();
  }, 120_000);
});
