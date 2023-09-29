import { AztecNodeService } from '@aztec/aztec-node';
import { AccountWallet, AztecAddress } from '@aztec/aztec.js';
import { Fr, FunctionSelector } from '@aztec/circuits.js';
import { deployL1Contract } from '@aztec/ethereum';
import { EthAddress } from '@aztec/foundation/eth-address';
import { DebugLogger } from '@aztec/foundation/log';
import { UniswapPortalAbi, UniswapPortalBytecode } from '@aztec/l1-artifacts';
import { UniswapContract } from '@aztec/noir-contracts/types';
import { PXE, TxStatus } from '@aztec/types';

import { getContract, parseEther } from 'viem';

import { CrossChainTestHarness } from './fixtures/cross_chain_test_harness.js';
import { delay, hashPayload, setup } from './fixtures/utils.js';

// PSA: This tests works on forked mainnet. There is a dump of the data in `dumpedState` such that we
// don't need to burn through RPC requests.
// To generate a new dump, use the `dumpChainState` cheatcode.
// To start an actual fork, use the command:
// anvil --fork-url https://mainnet.infura.io/v3/9928b52099854248b3a096be07a6b23c --fork-block-number 17514288 --chain-id 31337
// For CI, this is configured in `run_tests.sh` and `docker-compose.yml`

const dumpedState = 'src/fixtures/dumps/uniswap_state';
// When taking a dump use the block number of the fork to improve speed.
const EXPECTED_FORKED_BLOCK = 0; //17514288;
// We tell the archiver to only sync from this block.
process.env.SEARCH_START_BLOCK = EXPECTED_FORKED_BLOCK.toString();

// Should mint WETH on L2, swap to DAI using L1 Uniswap and mint this DAI back on L2
describe('uniswap_trade_on_l1_from_l2', () => {
  const WETH9_ADDRESS: EthAddress = EthAddress.fromString('0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2');
  const DAI_ADDRESS: EthAddress = EthAddress.fromString('0x6B175474E89094C44Da98b954EedeAC495271d0F');

  let aztecNode: AztecNodeService | undefined;
  let pxe: PXE;
  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  let ownerWallet: AccountWallet;
  let ownerAddress: AztecAddress;
  let ownerEthAddress: EthAddress;

  let daiCrossChainHarness: CrossChainTestHarness;
  let wethCrossChainHarness: CrossChainTestHarness;

  let uniswapPortal: any;
  let uniswapPortalAddress: EthAddress;
  let uniswapL2Contract: UniswapContract;

  const wethAmountToBridge = parseEther('1');
  const uniswapFeeTier = 3000n;
  const minimumOutputAmount = 0n;

  beforeEach(async () => {
    const {
      teardown: teardown_,
      aztecNode: aztecNode_,
      pxe: pxe_,
      deployL1ContractsValues,
      accounts,
      logger: logger_,
      wallet,
      cheatCodes,
    } = await setup(2, dumpedState);
    const walletClient = deployL1ContractsValues.walletClient;
    const publicClient = deployL1ContractsValues.publicClient;

    if (Number(await publicClient.getBlockNumber()) < EXPECTED_FORKED_BLOCK) {
      throw new Error('This test must be run on a fork of mainnet with the expected fork block');
    }

    aztecNode = aztecNode_;
    pxe = pxe_;
    logger = logger_;
    teardown = teardown_;
    ownerWallet = wallet;
    ownerAddress = accounts[0].address;
    ownerEthAddress = EthAddress.fromString((await walletClient.getAddresses())[0]);

    logger('Deploying DAI Portal, initializing and deploying l2 contract...');
    daiCrossChainHarness = await CrossChainTestHarness.new(
      aztecNode,
      pxe,
      deployL1ContractsValues,
      accounts,
      wallet,
      logger,
      cheatCodes,
      DAI_ADDRESS,
    );

    logger('Deploying WETH Portal, initializing and deploying l2 contract...');
    wethCrossChainHarness = await CrossChainTestHarness.new(
      aztecNode,
      pxe,
      deployL1ContractsValues,
      accounts,
      wallet,
      logger,
      cheatCodes,
      WETH9_ADDRESS,
    );

    logger('Deploy Uniswap portal on L1 and L2...');
    uniswapPortalAddress = await deployL1Contract(walletClient, publicClient, UniswapPortalAbi, UniswapPortalBytecode);
    uniswapPortal = getContract({
      address: uniswapPortalAddress.toString(),
      abi: UniswapPortalAbi,
      walletClient,
      publicClient,
    });
    // deploy l2 uniswap contract and attach to portal
    uniswapL2Contract = await UniswapContract.deploy(wallet).send({ portalContract: uniswapPortalAddress }).deployed();
    await uniswapL2Contract.attach(uniswapPortalAddress);

    await uniswapPortal.write.initialize(
      [deployL1ContractsValues!.l1ContractAddresses.registryAddress!.toString(), uniswapL2Contract.address.toString()],
      {} as any,
    );

    // Give me some WETH so I can deposit to L2 and do the swap...
    logger('Getting some weth');
    await walletClient.sendTransaction({ to: WETH9_ADDRESS.toString(), value: parseEther('1') });
  }, 100_000);

  afterEach(async () => {
    await teardown();
    await wethCrossChainHarness.stop();
    await daiCrossChainHarness.stop();
  });

  it('should uniswap trade on L1 from L2 funds privately (swaps WETH -> DAI)', async () => {
    const wethL1BeforeBalance = await wethCrossChainHarness.getL1BalanceOf(ownerEthAddress);

    // 1. Approve and deposit weth to the portal and move to L2
    const [secretForMintingWeth, secretHashForMintingWeth] = await wethCrossChainHarness.generateClaimSecret();
    const [secretForRedeemingWeth, secretHashForRedeemingWeth] = await wethCrossChainHarness.generateClaimSecret();

    const messageKey = await wethCrossChainHarness.sendTokensToPortalPrivate(
      wethAmountToBridge,
      secretHashForMintingWeth,
      secretHashForRedeemingWeth,
    );
    // funds transferred from owner to token portal
    expect(await wethCrossChainHarness.getL1BalanceOf(ownerEthAddress)).toBe(wethL1BeforeBalance - wethAmountToBridge);
    expect(await wethCrossChainHarness.getL1BalanceOf(wethCrossChainHarness.tokenPortalAddress)).toBe(
      wethAmountToBridge,
    );

    // Wait for the archiver to process the message
    await delay(5000);

    // Perform an unrelated transaction on L2 to progress the rollup. Here we mint public tokens.
    const unrelatedMintAmount = 1n;
    await wethCrossChainHarness.mintTokensPublicOnL2(unrelatedMintAmount);
    await wethCrossChainHarness.expectPublicBalanceOnL2(ownerAddress, unrelatedMintAmount);

    // 2. Claim WETH on L2
    logger('Minting weth on L2');
    await wethCrossChainHarness.consumeMessageOnAztecAndMintSecretly(
      wethAmountToBridge,
      secretHashForRedeemingWeth,
      messageKey,
      secretForMintingWeth,
    );
    await wethCrossChainHarness.redeemShieldPrivatelyOnL2(wethAmountToBridge, secretForRedeemingWeth);
    await wethCrossChainHarness.expectPrivateBalanceOnL2(ownerAddress, wethAmountToBridge);

    // Store balances
    const wethL2BalanceBeforeSwap = await wethCrossChainHarness.getL2PrivateBalanceOf(ownerAddress);
    const daiL2BalanceBeforeSwap = await daiCrossChainHarness.getL2PrivateBalanceOf(ownerAddress);

    // 3. Owner gives uniswap approval to unshield funds to self on its behalf
    logger('Approving uniswap to unshield funds to self on my behalf');
    const nonceForWETHUnshieldApproval = new Fr(2n);
    const unshieldToUniswapMessageHash = await hashPayload([
      uniswapL2Contract.address.toField(),
      wethCrossChainHarness.l2Token.address.toField(),
      FunctionSelector.fromSignature('unshield((Field),(Field),Field,Field)').toField(),
      ownerAddress.toField(),
      uniswapL2Contract.address.toField(),
      new Fr(wethAmountToBridge),
      nonceForWETHUnshieldApproval,
    ]);
    await ownerWallet.createAuthWitness(Fr.fromBuffer(unshieldToUniswapMessageHash));

    // 4. Swap on L1 - sends L2 to L1 message to withdraw WETH to L1 and another message to swap assets.
    logger('Withdrawing weth to L1 and sending message to swap to dai');
    const deadlineForDepositingSwappedDai = BigInt(2 ** 32 - 1); // max uint32 - 1
    const [secretForDepositingSwappedDai, secretHashForDepositingSwappedDai] =
      await daiCrossChainHarness.generateClaimSecret();
    const [secretForRedeemingDai, secretHashForRedeemingDai] = await daiCrossChainHarness.generateClaimSecret();

    const withdrawReceipt = await uniswapL2Contract.methods
      .swap(
        wethCrossChainHarness.l2Token.address,
        wethCrossChainHarness.l2Bridge.address,
        wethAmountToBridge,
        daiCrossChainHarness.l2Bridge.address,
        nonceForWETHUnshieldApproval,
        uniswapFeeTier,
        minimumOutputAmount,
        secretHashForRedeemingDai,
        secretHashForDepositingSwappedDai,
        deadlineForDepositingSwappedDai,
        ownerEthAddress,
        ownerEthAddress,
      )
      .send()
      .wait();
    expect(withdrawReceipt.status).toBe(TxStatus.MINED);
    // ensure that user's funds were burnt
    await wethCrossChainHarness.expectPrivateBalanceOnL2(ownerAddress, wethL2BalanceBeforeSwap - wethAmountToBridge);
    // ensure that uniswap contract didn't eat the funds.
    await wethCrossChainHarness.expectPublicBalanceOnL2(uniswapL2Contract.address, 0n);

    // 5. Consume L2 to L1 message by calling uniswapPortal.swap()
    logger('Execute withdraw and swap on the uniswapPortal!');
    const daiL1BalanceOfPortalBeforeSwap = await daiCrossChainHarness.getL1BalanceOf(
      daiCrossChainHarness.tokenPortalAddress,
    );
    const swapArgs = [
      wethCrossChainHarness.tokenPortalAddress.toString(),
      wethAmountToBridge,
      uniswapFeeTier,
      daiCrossChainHarness.tokenPortalAddress.toString(),
      minimumOutputAmount,
      secretHashForRedeemingDai.toString(true),
      secretHashForDepositingSwappedDai.toString(true),
      deadlineForDepositingSwappedDai,
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
    const daiL1BalanceOfPortalAfter = await daiCrossChainHarness.getL1BalanceOf(
      daiCrossChainHarness.tokenPortalAddress,
    );
    expect(daiL1BalanceOfPortalAfter).toBeGreaterThan(daiL1BalanceOfPortalBeforeSwap);
    const daiAmountToBridge = BigInt(daiL1BalanceOfPortalAfter - daiL1BalanceOfPortalBeforeSwap);

    // Wait for the archiver to process the message
    await delay(5000);
    // send a transfer tx to force through rollup with the message included
    await wethCrossChainHarness.performL2Transfer(0n);

    // 6. claim dai on L2
    logger('Consuming messages to mint dai on L2');
    await daiCrossChainHarness.consumeMessageOnAztecAndMintSecretly(
      daiAmountToBridge,
      secretHashForRedeemingDai,
      depositDaiMessageKey,
      secretForDepositingSwappedDai,
    );
    await daiCrossChainHarness.redeemShieldPrivatelyOnL2(daiAmountToBridge, secretForRedeemingDai);
    await daiCrossChainHarness.expectPrivateBalanceOnL2(ownerAddress, daiL2BalanceBeforeSwap + daiAmountToBridge);

    const wethL2BalanceAfterSwap = await wethCrossChainHarness.getL2PrivateBalanceOf(ownerAddress);
    const daiL2BalanceAfterSwap = await daiCrossChainHarness.getL2PrivateBalanceOf(ownerAddress);

    logger('WETH balance before swap: ', wethL2BalanceBeforeSwap.toString());
    logger('DAI balance before swap  : ', daiL2BalanceBeforeSwap.toString());
    logger('***** 🧚‍♀️ SWAP L2 assets on L1 Uniswap 🧚‍♀️ *****');
    logger('WETH balance after swap : ', wethL2BalanceAfterSwap.toString());
    logger('DAI balance after swap  : ', daiL2BalanceAfterSwap.toString());
  }, 140_000);
});
