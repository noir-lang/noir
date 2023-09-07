import {
  AztecAddress,
  EthAddress,
  Fr,
  Wallet,
  computeMessageSecretHash,
  createAccounts,
  createAztecRpcClient,
  getL1ContractAddresses,
} from '@aztec/aztec.js';
import { GrumpkinScalar } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { UniswapPortalAbi, UniswapPortalBytecode } from '@aztec/l1-artifacts';
import { SchnorrSingleKeyAccountContractAbi } from '@aztec/noir-contracts/artifacts';
import { NonNativeTokenContract, UniswapContract } from '@aztec/noir-contracts/types';
import { AztecRPC, TxStatus } from '@aztec/types';

import { createPublicClient, createWalletClient, getContract, http, parseEther } from 'viem';
import { mnemonicToAccount } from 'viem/accounts';
import { foundry } from 'viem/chains';

import { delay, deployAndInitializeNonNativeL2TokenContracts, deployL1Contract } from './util.js';

const logger = createDebugLogger('aztec:http-rpc-client');

export const MNEMONIC = 'test test test test test test test test test test test junk';

const INITIAL_BALANCE = 333n;
const wethAmountToBridge = parseEther('1');

const WETH9_ADDRESS = EthAddress.fromString('0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2');
const DAI_ADDRESS = EthAddress.fromString('0x6B175474E89094C44Da98b954EedeAC495271d0F');

const EXPECTED_FORKED_BLOCK = 17514288;

const aztecRpcUrl = 'http://localhost:8080';
const ethRpcUrl = 'http://localhost:8545';

const hdAccount = mnemonicToAccount(MNEMONIC);
const privateKey = GrumpkinScalar.fromBuffer(Buffer.from(hdAccount.getHdKey().privateKey!));

const walletClient = createWalletClient({
  account: hdAccount,
  chain: foundry,
  transport: http(ethRpcUrl),
});
const publicClient = createPublicClient({
  chain: foundry,
  transport: http(ethRpcUrl),
});

if (Number(await publicClient.getBlockNumber()) < EXPECTED_FORKED_BLOCK) {
  throw new Error('This test must be run on a fork of mainnet with the expected fork block');
}

const ethAccount = EthAddress.fromString((await walletClient.getAddresses())[0]);

const aztecRpcClient = createAztecRpcClient(aztecRpcUrl);
let wallet: Wallet;

/**
 * Deploys all l1 / l2 contracts
 * @param owner - Owner address.
 */
async function deployAllContracts(owner: AztecAddress) {
  const l1ContractsAddresses = await getL1ContractAddresses(aztecRpcUrl);
  logger('Deploying DAI Portal, initializing and deploying l2 contract...');
  const daiContracts = await deployAndInitializeNonNativeL2TokenContracts(
    wallet,
    walletClient,
    publicClient,
    l1ContractsAddresses!.registry,
    INITIAL_BALANCE,
    owner,
    DAI_ADDRESS,
  );
  const daiL2Contract = daiContracts.l2Contract;
  const daiContract = daiContracts.underlyingERC20;
  const daiTokenPortalAddress = daiContracts.tokenPortalAddress;

  logger('Deploying WETH Portal, initializing and deploying l2 contract...');
  const wethContracts = await deployAndInitializeNonNativeL2TokenContracts(
    wallet,
    walletClient,
    publicClient,
    l1ContractsAddresses!.registry,
    INITIAL_BALANCE,
    owner,
    WETH9_ADDRESS,
  );
  const wethL2Contract = wethContracts.l2Contract;
  const wethContract = wethContracts.underlyingERC20;
  const wethTokenPortal = wethContracts.tokenPortal;
  const wethTokenPortalAddress = wethContracts.tokenPortalAddress;

  logger('Deploy Uniswap portal on L1 and L2...');
  const uniswapPortalAddress = await deployL1Contract(
    walletClient,
    publicClient,
    UniswapPortalAbi,
    UniswapPortalBytecode,
  );
  const uniswapPortal = getContract({
    address: uniswapPortalAddress.toString(),
    abi: UniswapPortalAbi,
    walletClient,
    publicClient,
  });

  // deploy l2 uniswap contract and attach to portal
  const tx = UniswapContract.deploy(aztecRpcClient).send({ portalContract: uniswapPortalAddress });
  await tx.isMined({ interval: 0.5 });
  const receipt = await tx.getReceipt();
  const uniswapL2Contract = await UniswapContract.at(receipt.contractAddress!, wallet);
  await uniswapL2Contract.attach(uniswapPortalAddress);

  await uniswapPortal.write.initialize(
    [l1ContractsAddresses!.registry.toString(), uniswapL2Contract.address.toString()],
    {} as any,
  );

  return {
    daiL2Contract,
    daiContract,
    daiTokenPortalAddress,
    wethL2Contract,
    wethContract,
    wethTokenPortal,
    wethTokenPortalAddress,
    uniswapL2Contract,
    uniswapPortal,
  };
}

const getL2BalanceOf = async (aztecRpcClient: AztecRPC, owner: AztecAddress, l2Contract: NonNativeTokenContract) => {
  return await l2Contract.methods.getBalance(owner).view({ from: owner });
};

const logExpectedBalanceOnL2 = async (
  aztecRpcClient: AztecRPC,
  owner: AztecAddress,
  expectedBalance: bigint,
  l2Contract: NonNativeTokenContract,
) => {
  const balance = await getL2BalanceOf(aztecRpcClient, owner, l2Contract);
  logger(`Account ${owner} balance: ${balance}. Expected to be: ${expectedBalance}`);
};

const transferWethOnL2 = async (
  _aztecRpcClient: AztecRPC,
  wethL2Contract: NonNativeTokenContract,
  ownerAddress: AztecAddress,
  receiver: AztecAddress,
  transferAmount: bigint,
) => {
  const transferTx = wethL2Contract.methods.transfer(transferAmount, receiver).send({ origin: ownerAddress });
  await transferTx.isMined({ interval: 0.5 });
  const transferReceipt = await transferTx.getReceipt();
  // expect(transferReceipt.status).toBe(TxStatus.MINED);
  logger(`WETH to L2 Transfer Receipt status: ${transferReceipt.status} should be ${TxStatus.MINED}`);
};

/**
 * main fn
 */
async function main() {
  logger('Running L1/L2 messaging test on HTTP interface.');

  wallet = await createAccounts(aztecRpcClient, SchnorrSingleKeyAccountContractAbi, privateKey!, Fr.random(), 2);
  const accounts = await wallet.getAccounts();
  const [owner, receiver] = accounts;

  const result = await deployAllContracts(owner.address);
  const {
    daiL2Contract,
    daiContract,
    daiTokenPortalAddress,
    wethL2Contract,
    wethContract,
    wethTokenPortal,
    wethTokenPortalAddress,
    uniswapL2Contract,
    uniswapPortal,
  } = result;

  // Give me some WETH so I can deposit to L2 and do the swap...
  logger('Getting some weth');
  await walletClient.sendTransaction({ to: WETH9_ADDRESS.toString(), value: parseEther('1') });

  const meBeforeBalance = await wethContract.read.balanceOf([ethAccount.toString()]);
  // 1. Approve weth to be bridged
  await wethContract.write.approve([wethTokenPortalAddress.toString(), wethAmountToBridge], {} as any);

  // 2. Deposit weth into the portal and move to L2
  // generate secret
  const secret = Fr.random();
  const secretHash = await computeMessageSecretHash(secret);
  const secretString = `0x${secretHash.toBuffer().toString('hex')}` as `0x${string}`;
  const deadline = 2 ** 32 - 1; // max uint32 - 1
  logger('Sending messages to L1 portal');
  const args = [owner.toString(), wethAmountToBridge, deadline, secretString, ethAccount.toString()] as const;
  const { result: messageKeyHex } = await wethTokenPortal.simulate.depositToAztec(args, {
    account: ethAccount.toString(),
  } as any);
  await wethTokenPortal.write.depositToAztec(args, {} as any);
  // expect(await wethContract.read.balanceOf([ethAccount.toString()])).toBe(meBeforeBalance - wethAmountToBridge);

  const currentBalance = await wethContract.read.balanceOf([ethAccount.toString()]);
  logger(`Current Balance: ${currentBalance}. Should be: ${meBeforeBalance - wethAmountToBridge}`);
  const messageKey = Fr.fromString(messageKeyHex);

  // Wait for the archiver to process the message
  await delay(5000);
  // send a transfer tx to force through rollup with the message included
  const transferAmount = 1n;
  await transferWethOnL2(aztecRpcClient, wethL2Contract, owner.address, receiver.address, transferAmount);

  // 3. Claim WETH on L2
  logger('Minting weth on L2');
  // Call the mint tokens function on the noir contract
  const consumptionTx = wethL2Contract.methods
    .mint(wethAmountToBridge, owner.address, messageKey, secret, ethAccount.toField())
    .send({ origin: owner.address });
  await consumptionTx.isMined({ interval: 0.5 });
  const consumptionReceipt = await consumptionTx.getReceipt();
  // expect(consumptionReceipt.status).toBe(TxStatus.MINED);
  logger(`Consumption Receipt status: ${consumptionReceipt.status} should be ${TxStatus.MINED}`);
  // await expectBalanceOnL2(ownerAddress, wethAmountToBridge + initialBalance - transferAmount, wethL2Contract);

  // Store balances
  const wethBalanceBeforeSwap = await getL2BalanceOf(aztecRpcClient, owner.address, wethL2Contract);
  const daiBalanceBeforeSwap = await getL2BalanceOf(aztecRpcClient, owner.address, daiL2Contract);

  // 4. Send L2 to L1 message to withdraw funds and another message to swap assets.
  logger('Send L2 tx to withdraw WETH to uniswap portal and send message to swap assets on L1');
  // recipient is the uniswap portal
  const minimumOutputAmount = 0n;

  const withdrawTx = uniswapL2Contract.methods
    .swap(
      wethL2Contract.address.toField(),
      wethAmountToBridge,
      new Fr(3000),
      daiL2Contract.address.toField(),
      new Fr(minimumOutputAmount),
      owner.address,
      owner.address,
      secretHash,
      new Fr(2 ** 32 - 1),
      ethAccount.toField(),
      ethAccount.toField(),
    )
    .send({ origin: owner.address });
  await withdrawTx.isMined({ interval: 0.5 });
  const withdrawReceipt = await withdrawTx.getReceipt();
  // expect(withdrawReceipt.status).toBe(TxStatus.MINED);
  logger(`Withdraw receipt status: ${withdrawReceipt.status} should be ${TxStatus.MINED}`);

  // check weth balance of owner on L2 (we first bridged `wethAmountToBridge` into L2 and now withdrew it!)
  await logExpectedBalanceOnL2(aztecRpcClient, owner.address, INITIAL_BALANCE - transferAmount, wethL2Contract);

  // 5. Consume L2 to L1 message by calling uniswapPortal.swap()
  logger('Execute withdraw and swap on the uniswapPortal!');
  const daiBalanceOfPortalBefore = await daiContract.read.balanceOf([daiTokenPortalAddress.toString()]);
  logger(`DAI balance of portal: ${daiBalanceOfPortalBefore}`);
  const swapArgs = [
    wethTokenPortalAddress.toString(),
    wethAmountToBridge,
    3000,
    daiTokenPortalAddress.toString(),
    minimumOutputAmount,
    owner.address.toString(),
    secretString,
    deadline,
    ethAccount.toString(),
    true,
  ] as const;
  const { result: depositDaiMessageKeyHex } = await uniswapPortal.simulate.swap(swapArgs, {
    account: ethAccount.toString(),
  } as any);
  // this should also insert a message into the inbox.
  await uniswapPortal.write.swap(swapArgs, {} as any);
  const depositDaiMessageKey = Fr.fromString(depositDaiMessageKeyHex);
  // weth was swapped to dai and send to portal
  const daiBalanceOfPortalAfter = await daiContract.read.balanceOf([daiTokenPortalAddress.toString()]);
  // expect(daiBalanceOfPortalAfter).toBeGreaterThan(daiBalanceOfPortalBefore);
  logger(
    `DAI balance in Portal: ${daiBalanceOfPortalAfter} should be bigger than ${daiBalanceOfPortalBefore}. ${
      daiBalanceOfPortalAfter > daiBalanceOfPortalBefore
    }`,
  );
  const daiAmountToBridge = daiBalanceOfPortalAfter - daiBalanceOfPortalBefore;

  // Wait for the archiver to process the message
  await delay(5000);
  // send a transfer tx to force through rollup with the message included
  await transferWethOnL2(aztecRpcClient, wethL2Contract, owner.address, receiver.address, transferAmount);

  // 6. claim dai on L2
  logger('Consuming messages to mint dai on L2');
  // Call the mint tokens function on the noir contract
  const daiMintTx = daiL2Contract.methods
    .mint(daiAmountToBridge, owner.address, depositDaiMessageKey, secret, ethAccount.toField())
    .send({ origin: owner.address });
  await daiMintTx.isMined({ interval: 0.5 });
  const daiMintTxReceipt = await daiMintTx.getReceipt();
  // expect(daiMintTxReceipt.status).toBe(TxStatus.MINED);
  logger(`DAI mint TX status: ${daiMintTxReceipt.status} should be ${TxStatus.MINED}`);
  await logExpectedBalanceOnL2(
    aztecRpcClient,
    owner.address,
    INITIAL_BALANCE + BigInt(daiAmountToBridge),
    daiL2Contract,
  );

  const wethBalanceAfterSwap = await getL2BalanceOf(aztecRpcClient, owner.address, wethL2Contract);
  const daiBalanceAfterSwap = await getL2BalanceOf(aztecRpcClient, owner.address, daiL2Contract);

  logger('WETH balance before swap: ', wethBalanceBeforeSwap.toString());
  logger('DAI balance before swap  : ', daiBalanceBeforeSwap.toString());
  logger('***** 🧚‍♀️ SWAP L2 assets on L1 Uniswap 🧚‍♀️ *****');
  logger('WETH balance after swap : ', wethBalanceAfterSwap.toString());
  logger('DAI balance after swap  : ', daiBalanceAfterSwap.toString());
}

main()
  .then(() => {
    logger('Finished running successfully.');
    process.exit(0);
  })
  .catch(err => {
    logger.error('Error in main fn: ', err);
    process.exit(1);
  });
