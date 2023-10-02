import {
  AccountWallet,
  AztecAddress,
  EthAddress,
  Fr,
  NotePreimage,
  TxHash,
  TxStatus,
  computeMessageSecretHash,
  createDebugLogger,
  createPXEClient,
  getL1ContractAddresses,
  getSandboxAccountsWallets,
  sleep,
  waitForSandbox,
} from '@aztec/aztec.js';
import { FunctionSelector } from '@aztec/circuits.js';
import { UniswapPortalAbi, UniswapPortalBytecode } from '@aztec/l1-artifacts';
import { TokenBridgeContract, TokenContract, UniswapContract } from '@aztec/noir-contracts/types';

import {
  HDAccount,
  HttpTransport,
  PublicClient,
  WalletClient,
  createPublicClient,
  createWalletClient,
  getContract,
  http,
  parseEther,
} from 'viem';
import { mnemonicToAccount } from 'viem/accounts';
import { Chain, foundry } from 'viem/chains';

import { deployAndInitializeTokenAndBridgeContracts, deployL1Contract, hashPayload } from './utils.js';

const logger = createDebugLogger('aztec:canary');

const { SANDBOX_URL = 'http://localhost:8080', ETHEREUM_HOST = 'http://localhost:8545' } = process.env;

export const MNEMONIC = 'test test test test test test test test test test test junk';

const WETH9_ADDRESS = EthAddress.fromString('0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2');
const DAI_ADDRESS = EthAddress.fromString('0x6B175474E89094C44Da98b954EedeAC495271d0F');

const EXPECTED_FORKED_BLOCK = 17514288;

const pxeRpcUrl = SANDBOX_URL;
const ethRpcUrl = ETHEREUM_HOST;

const hdAccount = mnemonicToAccount(MNEMONIC);

const pxe = createPXEClient(pxeRpcUrl);

const wethAmountToBridge: bigint = parseEther('1');
const uniswapFeeTier = 3000;
const minimumOutputAmount = 0n;
const deadline = 2 ** 32 - 1; // max uint32 - 1

/**
 * Deploys all l1 / l2 contracts
 * @param owner - Owner address.
 */
async function deployAllContracts(
  ownerWallet: AccountWallet,
  ownerAddress: AztecAddress,
  publicClient: PublicClient<HttpTransport, Chain>,
  walletClient: WalletClient<HttpTransport, Chain, HDAccount>,
) {
  const l1ContractsAddresses = await getL1ContractAddresses(pxeRpcUrl);
  logger('Deploying DAI Portal, initializing and deploying l2 contract...');
  const daiContracts = await deployAndInitializeTokenAndBridgeContracts(
    ownerWallet,
    walletClient,
    publicClient,
    l1ContractsAddresses!.registryAddress,
    ownerAddress,
    DAI_ADDRESS,
  );
  const daiL2Contract = daiContracts.token;
  const daiL2Bridge = daiContracts.bridge;
  const daiContract = daiContracts.underlyingERC20;
  const daiTokenPortalAddress = daiContracts.tokenPortalAddress;

  logger('Deploying WETH Portal, initializing and deploying l2 contract...');
  const wethContracts = await deployAndInitializeTokenAndBridgeContracts(
    ownerWallet,
    walletClient,
    publicClient,
    l1ContractsAddresses!.registryAddress,
    ownerAddress,
    WETH9_ADDRESS,
  );
  const wethL2Contract = wethContracts.token;
  const wethL2Bridge = wethContracts.bridge;
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
  const uniswapL2Contract = await UniswapContract.deploy(ownerWallet)
    .send({ portalContract: uniswapPortalAddress })
    .deployed();
  await uniswapL2Contract.attach(uniswapPortalAddress);

  await uniswapPortal.write.initialize(
    [l1ContractsAddresses!.registryAddress.toString(), uniswapL2Contract.address.toString()],
    {} as any,
  );

  return {
    daiL2Contract,
    daiL2Bridge,
    daiContract,
    daiTokenPortalAddress,
    wethL2Contract,
    wethL2Bridge,
    wethContract,
    wethTokenPortal,
    wethTokenPortalAddress,
    uniswapL2Contract,
    uniswapPortal,
    uniswapPortalAddress,
  };
}

const getL2PrivateBalanceOf = async (owner: AztecAddress, l2Contract: TokenContract) => {
  return await l2Contract.methods.balance_of_private(owner).view({ from: owner });
};

const getL2PublicBalanceOf = async (owner: AztecAddress, l2Contract: TokenContract) => {
  return await l2Contract.methods.balance_of_public(owner).view();
};

const expectPrivateBalanceOnL2 = async (owner: AztecAddress, expectedBalance: bigint, l2Contract: TokenContract) => {
  const balance = await getL2PrivateBalanceOf(owner, l2Contract);
  logger(`Account ${owner} balance: ${balance}. Expected to be: ${expectedBalance}`);
  expect(balance).toBe(expectedBalance);
};

const expectPublicBalanceOnL2 = async (owner: AztecAddress, expectedBalance: bigint, l2Contract: TokenContract) => {
  const balance = await getL2PublicBalanceOf(owner, l2Contract);
  logger(`Account ${owner} balance: ${balance}. Expected to be: ${expectedBalance}`);
  expect(balance).toBe(expectedBalance);
};

const generateClaimSecret = async () => {
  const secret = Fr.random();
  const secretHash = await computeMessageSecretHash(secret);
  return [secret, secretHash];
};

const transferWethOnL2 = async (
  wethL2Contract: TokenContract,
  ownerAddress: AztecAddress,
  receiver: AztecAddress,
  transferAmount: bigint,
) => {
  const transferTx = wethL2Contract.methods.transfer_public(ownerAddress, receiver, transferAmount, 0).send();
  const receipt = await transferTx.wait();
  expect(receipt.status).toBe(TxStatus.MINED);
};

const consumeMessageOnAztecAndMintSecretly = async (
  l2Bridge: TokenBridgeContract,
  bridgeAmount: bigint,
  secretHashForRedeemingMintedNotes: Fr,
  canceller: EthAddress,
  messageKey: Fr,
  secretForL2MessageConsumption: Fr,
) => {
  logger('Consuming messages on L2 secretively');
  // Call the mint tokens function on the Aztec.nr contract
  const consumptionTx = l2Bridge.methods
    .claim_private(
      bridgeAmount,
      secretHashForRedeemingMintedNotes,
      canceller,
      messageKey,
      secretForL2MessageConsumption,
    )
    .send();
  const consumptionReceipt = await consumptionTx.wait();
  expect(consumptionReceipt.status).toBe(TxStatus.MINED);
  return consumptionReceipt.txHash;
};

const redeemShieldPrivatelyOnL2 = async (
  l2Contract: TokenContract,
  to: AztecAddress,
  shieldAmount: bigint,
  secret: Fr,
  secretHash: Fr,
  txHash: TxHash,
) => {
  // Add the note to the pxe.
  const storageSlot = new Fr(5);
  const preimage = new NotePreimage([new Fr(shieldAmount), secretHash]);
  await pxe.addNote(to, l2Contract.address, storageSlot, preimage, txHash);

  logger('Spending commitment in private call');
  const privateTx = l2Contract.methods.redeem_shield(to, shieldAmount, secret).send();
  const privateReceipt = await privateTx.wait();
  expect(privateReceipt.status).toBe(TxStatus.MINED);
};

describe('uniswap_trade_on_l1_from_l2', () => {
  let publicClient: PublicClient<HttpTransport, Chain>;
  let walletClient: WalletClient<HttpTransport, Chain, HDAccount>;
  let ownerWallet: AccountWallet;
  let ownerAddress: AztecAddress;
  let ownerEthAddress: EthAddress;

  beforeAll(async () => {
    await waitForSandbox(pxe);

    walletClient = createWalletClient({
      account: hdAccount,
      chain: foundry,
      transport: http(ethRpcUrl),
    });
    publicClient = createPublicClient({
      chain: foundry,
      transport: http(ethRpcUrl),
    });

    if (Number(await publicClient.getBlockNumber()) < EXPECTED_FORKED_BLOCK) {
      throw new Error('This test must be run on a fork of mainnet with the expected fork block');
    }

    ownerEthAddress = EthAddress.fromString((await walletClient.getAddresses())[0]);
  }, 60_000);
  it('should uniswap trade on L1 from L2 funds privately (swaps WETH -> DAI)', async () => {
    logger('Running L1/L2 messaging test on HTTP interface.');

    [ownerWallet] = await getSandboxAccountsWallets(pxe);
    const accounts = await ownerWallet.getRegisteredAccounts();
    ownerAddress = accounts[0].address;
    const receiver = accounts[1].address;

    const result = await deployAllContracts(ownerWallet, ownerAddress, publicClient, walletClient);
    const {
      daiL2Contract,
      daiL2Bridge,
      daiContract,
      daiTokenPortalAddress,
      wethL2Contract,
      wethL2Bridge,
      wethContract,
      wethTokenPortal,
      wethTokenPortalAddress,
      uniswapL2Contract,
      uniswapPortal,
    } = result;

    const ownerInitialBalance = await getL2PrivateBalanceOf(ownerAddress, wethL2Contract);
    logger(`Owner's initial L2 WETH balance: ${ownerInitialBalance}`);

    // Give me some WETH so I can deposit to L2 and do the swap...
    logger('Getting some weth');
    await walletClient.sendTransaction({
      to: WETH9_ADDRESS.toString(),
      value: parseEther('1'),
    });

    const wethL1BeforeBalance = await wethContract.read.balanceOf([ownerEthAddress.toString()]);
    // 1. Approve weth to be bridged
    await wethContract.write.approve([wethTokenPortalAddress.toString(), wethAmountToBridge], {} as any);

    // 2. Deposit weth into the portal and move to L2
    // generate secret
    const [secretForMintingWeth, secretHashForMintingWeth] = await generateClaimSecret();
    const [secretForRedeemingWeth, secretHashForRedeemingWeth] = await generateClaimSecret();
    logger('Sending messages to L1 portal');
    const args = [
      wethAmountToBridge,
      secretHashForRedeemingWeth.toString(true),
      ownerEthAddress.toString(),
      deadline,
      secretHashForMintingWeth.toString(true),
    ] as const;
    const { result: messageKeyHex } = await wethTokenPortal.simulate.depositToAztecPrivate(args, {
      account: ownerEthAddress.toString(),
    } as any);
    await wethTokenPortal.write.depositToAztecPrivate(args, {} as any);

    const currentL1Balance = await wethContract.read.balanceOf([ownerEthAddress.toString()]);
    logger(`Initial Balance: ${currentL1Balance}. Should be: ${wethL1BeforeBalance - wethAmountToBridge}`);
    expect(currentL1Balance).toBe(wethL1BeforeBalance - wethAmountToBridge);
    const messageKey = Fr.fromString(messageKeyHex);

    // Wait for the archiver to process the message
    await sleep(5000);
    // send a transfer tx to force through rollup with the message included
    await transferWethOnL2(wethL2Contract, ownerAddress, receiver, 0n);

    // 3. Claim WETH on L2
    logger('Minting weth on L2');
    const redeemingWethTxHash = await consumeMessageOnAztecAndMintSecretly(
      wethL2Bridge,
      wethAmountToBridge,
      secretHashForRedeemingWeth,
      ownerEthAddress,
      messageKey,
      secretForMintingWeth,
    );
    await redeemShieldPrivatelyOnL2(
      wethL2Contract,
      ownerAddress,
      wethAmountToBridge,
      secretForRedeemingWeth,
      secretHashForRedeemingWeth,
      redeemingWethTxHash,
    );
    await expectPrivateBalanceOnL2(ownerAddress, wethAmountToBridge + BigInt(ownerInitialBalance), wethL2Contract);

    // Store balances
    const wethL2BalanceBeforeSwap = await getL2PrivateBalanceOf(ownerAddress, wethL2Contract);
    const daiL2BalanceBeforeSwap = await getL2PrivateBalanceOf(ownerAddress, daiL2Contract);

    // 4. Owner gives uniswap approval to unshield funds to self on its behalf
    logger('Approving uniswap to unshield funds to self on my behalf');
    const nonceForWETHUnshieldApproval = new Fr(2n);
    const unshieldToUniswapMessageHash = await hashPayload([
      uniswapL2Contract.address.toField(),
      wethL2Contract.address.toField(),
      FunctionSelector.fromSignature('unshield((Field),(Field),Field,Field)').toField(),
      ownerAddress.toField(),
      uniswapL2Contract.address.toField(),
      new Fr(wethAmountToBridge),
      nonceForWETHUnshieldApproval,
    ]);
    await ownerWallet.createAuthWitness(Fr.fromBuffer(unshieldToUniswapMessageHash));

    // 5. Swap on L1 - sends L2 to L1 message to withdraw WETH to L1 and another message to swap assets.
    logger('Withdrawing weth to L1 and sending message to swap to dai');

    const [secretForDepositingSwappedDai, secretHashForDepositingSwappedDai] = await generateClaimSecret();
    const [secretForRedeemingDai, secretHashForRedeemingDai] = await generateClaimSecret();

    const withdrawReceipt = await uniswapL2Contract.methods
      .swap(
        wethL2Contract.address,
        wethL2Bridge.address,
        wethAmountToBridge,
        daiL2Bridge.address,
        nonceForWETHUnshieldApproval,
        uniswapFeeTier,
        minimumOutputAmount,
        secretHashForRedeemingDai,
        secretHashForDepositingSwappedDai,
        deadline,
        ownerEthAddress,
        ownerEthAddress,
      )
      .send()
      .wait();
    expect(withdrawReceipt.status).toBe(TxStatus.MINED);
    // ensure that user's funds were burnt
    await expectPrivateBalanceOnL2(ownerAddress, wethL2BalanceBeforeSwap - wethAmountToBridge, wethL2Contract);
    // ensure that uniswap contract didn't eat the funds.
    await expectPublicBalanceOnL2(uniswapL2Contract.address, 0n, wethL2Contract);

    // 6. Consume L2 to L1 message by calling uniswapPortal.swap()
    logger('Execute withdraw and swap on the uniswapPortal!');
    const daiL1BalanceOfPortalBeforeSwap = await daiContract.read.balanceOf([daiTokenPortalAddress.toString()]);
    const swapArgs = [
      wethTokenPortalAddress.toString(),
      wethAmountToBridge,
      uniswapFeeTier,
      daiTokenPortalAddress.toString(),
      minimumOutputAmount,
      secretHashForRedeemingDai.toString(true),
      secretHashForDepositingSwappedDai.toString(true),
      deadline,
      ownerEthAddress.toString(),
      true,
    ] as const;
    const { result: depositDaiMessageKeyHex } = await uniswapPortal.simulate.swapPrivate(swapArgs, {
      account: ownerEthAddress.toString(),
    } as any);

    // this should also insert a message into the inbox.
    await uniswapPortal.write.swapPrivate(swapArgs, {} as any);
    const depositDaiMessageKey = Fr.fromString(depositDaiMessageKeyHex);

    // weth was swapped to dai and send to portal
    const daiL1BalanceOfPortalAfter = await daiContract.read.balanceOf([daiTokenPortalAddress.toString()]);
    logger(
      `DAI balance in Portal: balance after (${daiL1BalanceOfPortalAfter}) should be bigger than balance before (${daiL1BalanceOfPortalBeforeSwap})`,
    );
    expect(daiL1BalanceOfPortalAfter).toBeGreaterThan(daiL1BalanceOfPortalBeforeSwap);
    const daiAmountToBridge = BigInt(daiL1BalanceOfPortalAfter - daiL1BalanceOfPortalBeforeSwap);

    // Wait for the archiver to process the message
    await sleep(5000);
    // send a transfer tx to force through rollup with the message included
    await transferWethOnL2(wethL2Contract, ownerAddress, receiver, 0n);

    // 7. claim dai on L2
    logger('Consuming messages to mint dai on L2');
    const redeemingDaiTxHash = await consumeMessageOnAztecAndMintSecretly(
      daiL2Bridge,
      daiAmountToBridge,
      secretHashForRedeemingDai,
      ownerEthAddress,
      depositDaiMessageKey,
      secretForDepositingSwappedDai,
    );
    await redeemShieldPrivatelyOnL2(
      daiL2Contract,
      ownerAddress,
      daiAmountToBridge,
      secretForRedeemingDai,
      secretHashForRedeemingDai,
      redeemingDaiTxHash,
    );
    await expectPrivateBalanceOnL2(ownerAddress, daiL2BalanceBeforeSwap + daiAmountToBridge, daiL2Contract);

    const wethL2BalanceAfterSwap = await getL2PrivateBalanceOf(ownerAddress, wethL2Contract);
    const daiL2BalanceAfterSwap = await getL2PrivateBalanceOf(ownerAddress, daiL2Contract);

    logger('WETH balance before swap: ', wethL2BalanceBeforeSwap.toString());
    logger('DAI balance before swap  : ', daiL2BalanceBeforeSwap.toString());
    logger('***** 🧚‍♀️ SWAP L2 assets on L1 Uniswap 🧚‍♀️ *****');
    logger('WETH balance after swap : ', wethL2BalanceAfterSwap.toString());
    logger('DAI balance after swap  : ', daiL2BalanceAfterSwap.toString());
  }, 140_000);
});
