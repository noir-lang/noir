import { AztecNodeService } from '@aztec/aztec-node';
import { AztecAddress, AztecRPCServer, Contract, ContractDeployer, TxStatus } from '@aztec/aztec.js';
import { EthAddress } from '@aztec/foundation/eth-address';
import { NonNativeTokenContractAbi } from '@aztec/noir-contracts/examples';

import { CircuitsWasm } from '@aztec/circuits.js';
import { computeSecretMessageHash } from '@aztec/circuits.js/abis';
import { DeployL1Contracts, deployL1Contract } from '@aztec/ethereum';
import { toBigIntBE, toBufferBE } from '@aztec/foundation/bigint-buffer';
import { Fr, Point } from '@aztec/foundation/fields';
import { DebugLogger } from '@aztec/foundation/log';
import {
  OutboxAbi,
  PortalERC20Abi,
  PortalERC20Bytecode,
  TokenPortalAbi,
  TokenPortalBytecode,
} from '@aztec/l1-artifacts';
import { Chain, HttpTransport, PublicClient, getContract } from 'viem';
import { setup } from './setup.js';
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

  let contract: Contract;
  let ethAccount: EthAddress;

  let tokenPortalAddress: EthAddress;
  let underlyingERC20Address: EthAddress;
  let rollupRegistryAddress: EthAddress;
  let tokenPortal: any;
  let underlyingERC20: any;
  let outbox: any;
  let publicClient: PublicClient<HttpTransport, Chain>;
  let walletClient: any;

  beforeEach(async () => {
    let deployL1ContractsValues: DeployL1Contracts;
    ({ aztecNode, aztecRpcServer, deployL1ContractsValues, accounts, logger } = await setup(2));
    rollupRegistryAddress = deployL1ContractsValues!.registryAddress;

    walletClient = deployL1ContractsValues.walletClient;
    publicClient = deployL1ContractsValues.publicClient;

    ethAccount = EthAddress.fromString((await walletClient.getAddresses())[0]);

    // Deploy portal contracts
    underlyingERC20Address = await deployL1Contract(walletClient, publicClient, PortalERC20Abi, PortalERC20Bytecode);
    tokenPortalAddress = await deployL1Contract(walletClient, publicClient, TokenPortalAbi, TokenPortalBytecode);
    underlyingERC20 = getContract({
      address: underlyingERC20Address.toString(),
      abi: PortalERC20Abi,
      walletClient,
      publicClient,
    });
    tokenPortal = getContract({
      address: tokenPortalAddress.toString(),
      abi: TokenPortalAbi,
      walletClient,
      publicClient,
    });
    outbox = getContract({
      address: deployL1ContractsValues.outboxAddress.toString(),
      abi: OutboxAbi,
      publicClient,
    });
  }, 30_000);

  afterEach(async () => {
    await aztecNode?.stop();
    await aztecRpcServer?.stop();
  });

  const expectBalance = async (owner: AztecAddress, expectedBalance: bigint) => {
    const ownerPublicKey = await aztecRpcServer.getAccountPublicKey(owner);
    const [balance] = await contract.methods.getBalance(pointToPublicKey(ownerPublicKey)).view({ from: owner });
    logger(`Account ${owner} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  const pointToPublicKey = (point: Point) => {
    const x = point.buffer.subarray(0, 32);
    const y = point.buffer.subarray(32, 64);
    return {
      x: toBigIntBE(x),
      y: toBigIntBE(y),
    };
  };

  const deployContract = async (initialBalance = 0n, owner = { x: 0n, y: 0n }) => {
    logger(`Deploying L2 Token contract...`);
    const deployer = new ContractDeployer(NonNativeTokenContractAbi, aztecRpcServer);
    const tx = deployer.deploy(initialBalance, owner).send({
      portalContract: tokenPortalAddress,
    });
    const receipt = await tx.getReceipt();
    contract = new Contract(receipt.contractAddress!, NonNativeTokenContractAbi, aztecRpcServer);
    await contract.attach(tokenPortalAddress);

    await tx.isMined(0, 0.1);
    await tx.getReceipt();
    logger('L2 contract deployed');
    return contract;
  };

  it('Milestone 2: Deposit funds from L1 -> L2 and withdraw back to L1', async () => {
    const initialBalance = 10n;
    const [ownerAddress, receiver] = accounts;
    const ownerPub = await aztecRpcServer.getAccountPublicKey(ownerAddress);
    const deployedL2Contract = await deployContract(initialBalance, pointToPublicKey(ownerPub));
    await expectBalance(accounts[0], initialBalance);

    const l2TokenAddress = deployedL2Contract.address.toString() as `0x${string}`;

    logger('Initializing the TokenPortal contract');
    await tokenPortal.write.initialize(
      [rollupRegistryAddress.toString(), underlyingERC20Address.toString(), l2TokenAddress],
      {} as any,
    );
    logger('Successfully initialized the TokenPortal contract');

    // Generate a claim secret using pedersen
    // TODO: make this into an aztec.js utility function
    logger("Generating a claim secret using pedersen's hash function");
    const wasm = await CircuitsWasm.get();
    const secret = Fr.random();
    const claimSecretHash = computeSecretMessageHash(wasm, secret);
    logger('Generated claim secret: ', claimSecretHash);

    logger('Minting tokens on L1');
    await underlyingERC20.write.mint([ethAccount.toString(), 1000000n], {} as any);
    await underlyingERC20.write.approve([tokenPortalAddress.toString(), 1000n], {} as any);

    expect(await underlyingERC20.read.balanceOf([ethAccount.toString()])).toBe(1000000n);

    // Deposit tokens to the TokenPortal
    const secretString = `0x${claimSecretHash.toBuffer().toString('hex')}` as `0x${string}`;
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
    const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));
    await delay(5000); /// waiting 5 seconds.

    // send a transfer tx to force through rollup with the message included
    const transferAmount = 1n;
    const transferTx = contract.methods
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

    const consumptionTx = deployedL2Contract.methods
      .mint(mintAmount, pointToPublicKey(ownerPub), messageKey, secret)
      .send({ from: ownerAddress });

    await consumptionTx.isMined(0, 0.1);
    const consumptionReceipt = await consumptionTx.getReceipt();

    expect(consumptionReceipt.status).toBe(TxStatus.MINED);
    await expectBalance(ownerAddress, mintAmount + initialBalance - transferAmount);

    // time to withdraw the funds again!
    const withdrawAmount = 9n;

    logger('Withdrawing funds from L2');

    logger('Ensure that the entry is not in outbox yet');
    const contractInfo = await aztecNode.getContractInfo(contract.address);
    // 0x00f714ce, selector for "withdraw(uint256,address)"
    const content = sha256ToField(
      Buffer.concat([Buffer.from([0x00, 0xf7, 0x14, 0xce]), toBufferBE(withdrawAmount, 32), ethAccount.toBuffer32()]),
    );
    const entryKey = sha256ToField(
      Buffer.concat([
        contract.address.toBuffer(),
        new Fr(1).toBuffer(), // aztec version
        contractInfo?.portalContractAddress.toBuffer32() ?? Buffer.alloc(32, 0),
        new Fr(publicClient.chain.id).toBuffer(), // chain id
        content.toBuffer(),
      ]),
    );
    expect(await outbox.read.contains([entryKey.toString()])).toBeFalsy();

    logger('Send L2 tx to withdraw funds');
    const withdrawTx = deployedL2Contract.methods
      .withdraw(withdrawAmount, pointToPublicKey(ownerPub), ethAccount)
      .send({ from: ownerAddress });

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

    expect(withdrawEntryKey).toBe(entryKey.toString());

    expect(await outbox.read.contains([withdrawEntryKey])).toBeTruthy();

    await walletClient.writeContract(withdrawRequest);
    expect(await underlyingERC20.read.balanceOf([ethAccount.toString()])).toBe(1000000n - mintAmount + withdrawAmount);

    expect(await outbox.read.contains([withdrawEntryKey])).toBeFalsy();
  }, 120_000);
});
