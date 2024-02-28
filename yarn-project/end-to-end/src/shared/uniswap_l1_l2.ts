import {
  AccountWallet,
  AztecAddress,
  DebugLogger,
  EthAddress,
  Fr,
  PXE,
  TxStatus,
  computeAuthWitMessageHash,
  sleep,
} from '@aztec/aztec.js';
import { deployL1Contract } from '@aztec/ethereum';
import { UniswapPortalAbi, UniswapPortalBytecode } from '@aztec/l1-artifacts';
import { UniswapContract } from '@aztec/noir-contracts.js/Uniswap';

import { jest } from '@jest/globals';
import { Chain, HttpTransport, PublicClient, getContract, parseEther } from 'viem';

import { publicDeployAccounts } from '../fixtures/utils.js';
import { CrossChainTestHarness } from './cross_chain_test_harness.js';

// PSA: This tests works on forked mainnet. There is a dump of the data in `dumpedState` such that we
// don't need to burn through RPC requests.
// To generate a new dump, use the `dumpChainState` cheatcode.
// To start an actual fork, use the command:
// anvil --fork-url https://mainnet.infura.io/v3/9928b52099854248b3a096be07a6b23c --fork-block-number 17514288 --chain-id 31337
// For CI, this is configured in `run_tests.sh` and `docker-compose.yml`

// docs:start:uniswap_l1_l2_test_setup_const
const TIMEOUT = 360_000;

/** Objects to be returned by the uniswap setup function */
export type UniswapSetupContext = {
  /** The Private eXecution Environment (PXE). */
  pxe: PXE;
  /** Logger instance named as the current test. */
  logger: DebugLogger;
  /** Viem Public client instance. */
  publicClient: PublicClient<HttpTransport, Chain>;
  /** Viem Wallet Client instance. */
  walletClient: any;
  /** The owner wallet. */
  ownerWallet: AccountWallet;
  /** The sponsor wallet. */
  sponsorWallet: AccountWallet;
};
// docs:end:uniswap_l1_l2_test_setup_const

export const uniswapL1L2TestSuite = (
  setup: () => Promise<UniswapSetupContext>,
  cleanup: () => Promise<void>,
  expectedForkBlockNumber = 17514288,
) => {
  // docs:start:uniswap_l1_l2_test_beforeAll
  describe('uniswap_trade_on_l1_from_l2', () => {
    jest.setTimeout(TIMEOUT);

    const WETH9_ADDRESS: EthAddress = EthAddress.fromString('0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2');
    const DAI_ADDRESS: EthAddress = EthAddress.fromString('0x6B175474E89094C44Da98b954EedeAC495271d0F');

    let pxe: PXE;
    let logger: DebugLogger;

    let walletClient: any;

    let ownerWallet: AccountWallet;
    let ownerAddress: AztecAddress;
    let ownerEthAddress: EthAddress;
    // does transactions on behalf of owner on Aztec:
    let sponsorWallet: AccountWallet;
    let sponsorAddress: AztecAddress;

    let daiCrossChainHarness: CrossChainTestHarness;
    let wethCrossChainHarness: CrossChainTestHarness;

    let uniswapPortal: any;
    let uniswapPortalAddress: EthAddress;
    let uniswapL2Contract: UniswapContract;

    const wethAmountToBridge = parseEther('1');
    const uniswapFeeTier = 3000n;
    const minimumOutputAmount = 0n;
    const deadlineForDepositingSwappedDai = BigInt(2 ** 32 - 1); // max uint32

    beforeAll(async () => {
      let publicClient: PublicClient<HttpTransport, Chain>;
      ({ pxe, logger, publicClient, walletClient, ownerWallet, sponsorWallet } = await setup());

      // walletClient = deployL1ContractsValues.walletClient;
      // const publicClient = deployL1ContractsValues.publicClient;

      if (Number(await publicClient.getBlockNumber()) < expectedForkBlockNumber) {
        throw new Error('This test must be run on a fork of mainnet with the expected fork block');
      }

      ownerAddress = ownerWallet.getAddress();
      sponsorAddress = sponsorWallet.getAddress();
      ownerEthAddress = EthAddress.fromString((await walletClient.getAddresses())[0]);

      await publicDeployAccounts(ownerWallet, [ownerAddress, sponsorAddress]);

      logger('Deploying DAI Portal, initializing and deploying l2 contract...');
      daiCrossChainHarness = await CrossChainTestHarness.new(
        pxe,
        publicClient,
        walletClient,
        ownerWallet,
        logger,
        DAI_ADDRESS,
      );

      logger('Deploying WETH Portal, initializing and deploying l2 contract...');
      wethCrossChainHarness = await CrossChainTestHarness.new(
        pxe,
        publicClient,
        walletClient,
        ownerWallet,
        logger,
        WETH9_ADDRESS,
      );

      logger('Deploy Uniswap portal on L1 and L2...');
      uniswapPortalAddress = await deployL1Contract(
        walletClient,
        publicClient,
        UniswapPortalAbi,
        UniswapPortalBytecode,
      );
      uniswapPortal = getContract({
        address: uniswapPortalAddress.toString(),
        abi: UniswapPortalAbi,
        client: walletClient,
      });
      // deploy l2 uniswap contract and attach to portal
      uniswapL2Contract = await UniswapContract.deploy(ownerWallet)
        .send({ portalContract: uniswapPortalAddress })
        .deployed();

      const registryAddress = (await pxe.getNodeInfo()).l1ContractAddresses.registryAddress;
      await uniswapPortal.write.initialize(
        [registryAddress.toString(), uniswapL2Contract.address.toString()],
        {} as any,
      );
    });

    beforeEach(async () => {
      // Give me some WETH so I can deposit to L2 and do the swap...
      logger('Getting some weth');
      await walletClient.sendTransaction({ to: WETH9_ADDRESS.toString(), value: parseEther('1') });
    });
    // docs:end:uniswap_l1_l2_test_beforeAll

    afterAll(async () => {
      await cleanup();
    });
    // docs:start:uniswap_private
    it('should uniswap trade on L1 from L2 funds privately (swaps WETH -> DAI)', async () => {
      const wethL1BeforeBalance = await wethCrossChainHarness.getL1BalanceOf(ownerEthAddress);

      // 1. Approve and deposit weth to the portal and move to L2
      const [secretForMintingWeth, secretHashForMintingWeth] = wethCrossChainHarness.generateClaimSecret();
      const [secretForRedeemingWeth, secretHashForRedeemingWeth] = wethCrossChainHarness.generateClaimSecret();

      const messageKey = await wethCrossChainHarness.sendTokensToPortalPrivate(
        secretHashForRedeemingWeth,
        wethAmountToBridge,
        secretHashForMintingWeth,
      );
      // funds transferred from owner to token portal
      expect(await wethCrossChainHarness.getL1BalanceOf(ownerEthAddress)).toBe(
        wethL1BeforeBalance - wethAmountToBridge,
      );
      expect(await wethCrossChainHarness.getL1BalanceOf(wethCrossChainHarness.tokenPortalAddress)).toBe(
        wethAmountToBridge,
      );

      // Wait for the archiver to process the message
      await sleep(5000);

      // Perform an unrelated transaction on L2 to progress the rollup. Here we mint public tokens.
      await wethCrossChainHarness.mintTokensPublicOnL2(0n);

      // 2. Claim WETH on L2
      logger('Minting weth on L2');
      await wethCrossChainHarness.consumeMessageOnAztecAndMintSecretly(
        secretHashForRedeemingWeth,
        wethAmountToBridge,
        messageKey,
        secretForMintingWeth,
      );
      await wethCrossChainHarness.redeemShieldPrivatelyOnL2(wethAmountToBridge, secretForRedeemingWeth);
      await wethCrossChainHarness.expectPrivateBalanceOnL2(ownerAddress, wethAmountToBridge);

      // Store balances
      const wethL2BalanceBeforeSwap = await wethCrossChainHarness.getL2PrivateBalanceOf(ownerAddress);
      const daiL2BalanceBeforeSwap = await daiCrossChainHarness.getL2PrivateBalanceOf(ownerAddress);

      // before swap - check nonce_for_burn_approval stored on uniswap
      // (which is used by uniswap to approve the bridge to burn funds on its behalf to exit to L1)
      const nonceForBurnApprovalBeforeSwap = await uniswapL2Contract.methods.nonce_for_burn_approval().view();

      // 3. Owner gives uniswap approval to unshield funds to self on its behalf
      logger('Approving uniswap to unshield funds to self on my behalf');
      const nonceForWETHUnshieldApproval = new Fr(1n);
      const unshieldToUniswapMessageHash = computeAuthWitMessageHash(
        uniswapL2Contract.address,
        wethCrossChainHarness.l2Token.methods
          .unshield(ownerAddress, uniswapL2Contract.address, wethAmountToBridge, nonceForWETHUnshieldApproval)
          .request(),
      );
      await ownerWallet.createAuthWitness(unshieldToUniswapMessageHash);

      // 4. Swap on L1 - sends L2 to L1 message to withdraw WETH to L1 and another message to swap assets.
      logger('Withdrawing weth to L1 and sending message to swap to dai');
      const [secretForDepositingSwappedDai, secretHashForDepositingSwappedDai] =
        daiCrossChainHarness.generateClaimSecret();
      const [secretForRedeemingDai, secretHashForRedeemingDai] = daiCrossChainHarness.generateClaimSecret();

      const withdrawReceipt = await uniswapL2Contract.methods
        .swap_private(
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
      // check burn approval nonce incremented:
      const nonceForBurnApprovalAfterSwap = await uniswapL2Contract.methods.nonce_for_burn_approval().view();
      expect(nonceForBurnApprovalAfterSwap).toBe(nonceForBurnApprovalBeforeSwap + 1n);

      // 5. Consume L2 to L1 message by calling uniswapPortal.swap_private()
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
        secretHashForRedeemingDai.toString(),
        secretHashForDepositingSwappedDai.toString(),
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
      await sleep(5000);
      // send a transfer tx to force through rollup with the message included
      await wethCrossChainHarness.mintTokensPublicOnL2(0n);

      // 6. claim dai on L2
      logger('Consuming messages to mint dai on L2');
      await daiCrossChainHarness.consumeMessageOnAztecAndMintSecretly(
        secretHashForRedeemingDai,
        daiAmountToBridge,
        depositDaiMessageKey,
        secretForDepositingSwappedDai,
      );
      await daiCrossChainHarness.redeemShieldPrivatelyOnL2(daiAmountToBridge, secretForRedeemingDai);
      await daiCrossChainHarness.expectPrivateBalanceOnL2(ownerAddress, daiL2BalanceBeforeSwap + daiAmountToBridge);

      const wethL2BalanceAfterSwap = await wethCrossChainHarness.getL2PrivateBalanceOf(ownerAddress);
      const daiL2BalanceAfterSwap = await daiCrossChainHarness.getL2PrivateBalanceOf(ownerAddress);

      logger('WETH balance before swap: ' + wethL2BalanceBeforeSwap.toString());
      logger('DAI balance before swap  : ' + daiL2BalanceBeforeSwap.toString());
      logger('***** 🧚‍♀️ SWAP L2 assets on L1 Uniswap 🧚‍♀️ *****');
      logger('WETH balance after swap : ', wethL2BalanceAfterSwap.toString());
      logger('DAI balance after swap  : ', daiL2BalanceAfterSwap.toString());
    });
    // docs:end:uniswap_private

    // docs:start:uniswap_public
    it('should uniswap trade on L1 from L2 funds publicly (swaps WETH -> DAI)', async () => {
      const wethL1BeforeBalance = await wethCrossChainHarness.getL1BalanceOf(ownerEthAddress);

      // 1. Approve and deposit weth to the portal and move to L2
      const [secretForMintingWeth, secretHashForMintingWeth] = wethCrossChainHarness.generateClaimSecret();

      const messageKey = await wethCrossChainHarness.sendTokensToPortalPublic(
        wethAmountToBridge,
        secretHashForMintingWeth,
      );
      // funds transferred from owner to token portal
      expect(await wethCrossChainHarness.getL1BalanceOf(ownerEthAddress)).toBe(
        wethL1BeforeBalance - wethAmountToBridge,
      );
      expect(await wethCrossChainHarness.getL1BalanceOf(wethCrossChainHarness.tokenPortalAddress)).toBe(
        wethAmountToBridge,
      );

      // Wait for the archiver to process the message
      await sleep(5000);

      // Perform an unrelated transaction on L2 to progress the rollup. Here we transfer 0 tokens
      await wethCrossChainHarness.mintTokensPublicOnL2(0n);

      // 2. Claim WETH on L2
      logger('Minting weth on L2');
      await wethCrossChainHarness.consumeMessageOnAztecAndMintPublicly(
        wethAmountToBridge,
        messageKey,
        secretForMintingWeth,
      );
      await wethCrossChainHarness.expectPublicBalanceOnL2(ownerAddress, wethAmountToBridge);

      // Store balances
      const wethL2BalanceBeforeSwap = await wethCrossChainHarness.getL2PublicBalanceOf(ownerAddress);
      const daiL2BalanceBeforeSwap = await daiCrossChainHarness.getL2PublicBalanceOf(ownerAddress);

      // 3. Owner gives uniswap approval to transfer funds on its behalf
      const nonceForWETHTransferApproval = new Fr(1n);
      const transferMessageHash = computeAuthWitMessageHash(
        uniswapL2Contract.address,
        wethCrossChainHarness.l2Token.methods
          .transfer_public(ownerAddress, uniswapL2Contract.address, wethAmountToBridge, nonceForWETHTransferApproval)
          .request(),
      );
      await ownerWallet.setPublicAuth(transferMessageHash, true).send().wait();

      // before swap - check nonce_for_burn_approval stored on uniswap
      // (which is used by uniswap to approve the bridge to burn funds on its behalf to exit to L1)
      const nonceForBurnApprovalBeforeSwap = await uniswapL2Contract.methods.nonce_for_burn_approval().view();

      // 4. Swap on L1 - sends L2 to L1 message to withdraw WETH to L1 and another message to swap assets.
      const [secretForDepositingSwappedDai, secretHashForDepositingSwappedDai] =
        daiCrossChainHarness.generateClaimSecret();

      // 4.1 Owner approves user to swap on their behalf:
      const nonceForSwap = new Fr(3n);
      const action = uniswapL2Contract
        .withWallet(sponsorWallet)
        .methods.swap_public(
          ownerAddress,
          wethCrossChainHarness.l2Bridge.address,
          wethAmountToBridge,
          daiCrossChainHarness.l2Bridge.address,
          nonceForWETHTransferApproval,
          uniswapFeeTier,
          minimumOutputAmount,
          ownerAddress,
          secretHashForDepositingSwappedDai,
          deadlineForDepositingSwappedDai,
          ownerEthAddress,
          ownerEthAddress,
          nonceForSwap,
        );
      const swapMessageHash = computeAuthWitMessageHash(sponsorAddress, action.request());
      await ownerWallet.setPublicAuth(swapMessageHash, true).send().wait();

      // 4.2 Call swap_public from user2 on behalf of owner
      const withdrawReceipt = await action.send().wait();
      expect(withdrawReceipt.status).toBe(TxStatus.MINED);

      // check weth balance of owner on L2 (we first bridged `wethAmountToBridge` into L2 and now withdrew it!)
      await wethCrossChainHarness.expectPublicBalanceOnL2(ownerAddress, wethL2BalanceBeforeSwap - wethAmountToBridge);

      // check burn approval nonce incremented:
      const nonceForBurnApprovalAfterSwap = await uniswapL2Contract.methods.nonce_for_burn_approval().view();
      expect(nonceForBurnApprovalAfterSwap).toBe(nonceForBurnApprovalBeforeSwap + 1n);

      // 5. Perform the swap on L1 with the `uniswapPortal.swap_private()` (consuming L2 to L1 messages)
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
        ownerAddress.toString(),
        secretHashForDepositingSwappedDai.toString(),
        deadlineForDepositingSwappedDai,
        ownerEthAddress.toString(),
        true,
      ] as const;
      const { result: depositDaiMessageKeyHex } = await uniswapPortal.simulate.swapPublic(swapArgs, {
        account: ownerEthAddress.toString(),
      } as any);

      // this should also insert a message into the inbox.
      await uniswapPortal.write.swapPublic(swapArgs, {} as any);
      const depositDaiMessageKey = Fr.fromString(depositDaiMessageKeyHex);
      // weth was swapped to dai and send to portal
      const daiL1BalanceOfPortalAfter = await daiCrossChainHarness.getL1BalanceOf(
        daiCrossChainHarness.tokenPortalAddress,
      );
      expect(daiL1BalanceOfPortalAfter).toBeGreaterThan(daiL1BalanceOfPortalBeforeSwap);
      const daiAmountToBridge = BigInt(daiL1BalanceOfPortalAfter - daiL1BalanceOfPortalBeforeSwap);

      // Wait for the archiver to process the message
      await sleep(5000);
      // send a transfer tx to force through rollup with the message included
      await wethCrossChainHarness.mintTokensPublicOnL2(0n);

      // 6. claim dai on L2
      logger('Consuming messages to mint dai on L2');
      await daiCrossChainHarness.consumeMessageOnAztecAndMintPublicly(
        daiAmountToBridge,
        depositDaiMessageKey,
        secretForDepositingSwappedDai,
      );
      await daiCrossChainHarness.expectPublicBalanceOnL2(ownerAddress, daiL2BalanceBeforeSwap + daiAmountToBridge);

      const wethL2BalanceAfterSwap = await wethCrossChainHarness.getL2PublicBalanceOf(ownerAddress);
      const daiL2BalanceAfterSwap = await daiCrossChainHarness.getL2PublicBalanceOf(ownerAddress);

      logger('WETH balance before swap: ', wethL2BalanceBeforeSwap.toString());
      logger('DAI balance before swap  : ', daiL2BalanceBeforeSwap.toString());
      logger('***** 🧚‍♀️ SWAP L2 assets on L1 Uniswap 🧚‍♀️ *****');
      logger('WETH balance after swap : ', wethL2BalanceAfterSwap.toString());
      logger('DAI balance after swap  : ', daiL2BalanceAfterSwap.toString());
    }, 360_000);
    // docs:end:uniswap_public

    // Edge cases for the private flow:
    // note - tests for uniswapPortal.sol and minting asset on L2 are covered in other tests.

    it('swap_private reverts without unshield approval', async () => {
      // swap should fail since no withdraw approval to uniswap:
      const nonceForWETHUnshieldApproval = new Fr(2n);

      const expectedMessageHash = computeAuthWitMessageHash(
        uniswapL2Contract.address,
        wethCrossChainHarness.l2Token.methods
          .unshield(ownerAddress, uniswapL2Contract.address, wethAmountToBridge, nonceForWETHUnshieldApproval)
          .request(),
      );

      await expect(
        uniswapL2Contract.methods
          .swap_private(
            wethCrossChainHarness.l2Token.address,
            wethCrossChainHarness.l2Bridge.address,
            wethAmountToBridge,
            daiCrossChainHarness.l2Bridge.address,
            nonceForWETHUnshieldApproval,
            uniswapFeeTier,
            minimumOutputAmount,
            Fr.random(),
            Fr.random(),
            deadlineForDepositingSwappedDai,
            ownerEthAddress,
            ownerEthAddress,
          )
          .simulate(),
      ).rejects.toThrowError(`Unknown auth witness for message hash ${expectedMessageHash.toString()}`);
    });

    it("can't swap if user passes a token different to what the bridge tracks", async () => {
      // 1. give user private funds on L2:
      const [secretForRedeemingWeth, secretHashForRedeemingWeth] = wethCrossChainHarness.generateClaimSecret();
      await wethCrossChainHarness.mintTokensPrivateOnL2(wethAmountToBridge, secretHashForRedeemingWeth);
      await wethCrossChainHarness.redeemShieldPrivatelyOnL2(wethAmountToBridge, secretForRedeemingWeth);
      await wethCrossChainHarness.expectPrivateBalanceOnL2(ownerAddress, wethAmountToBridge);

      // 2. owner gives uniswap approval to unshield funds:
      logger('Approving uniswap to unshield funds to self on my behalf');
      const nonceForWETHUnshieldApproval = new Fr(3n);
      const unshieldToUniswapMessageHash = computeAuthWitMessageHash(
        uniswapL2Contract.address,
        wethCrossChainHarness.l2Token.methods
          .unshield(ownerAddress, uniswapL2Contract.address, wethAmountToBridge, nonceForWETHUnshieldApproval)
          .request(),
      );
      await ownerWallet.createAuthWitness(unshieldToUniswapMessageHash);

      // 3. Swap but send the wrong token address
      logger('Swap but send the wrong token address');
      await expect(
        uniswapL2Contract.methods
          .swap_private(
            wethCrossChainHarness.l2Token.address, // send weth token
            daiCrossChainHarness.l2Bridge.address, // but dai bridge!
            wethAmountToBridge,
            daiCrossChainHarness.l2Bridge.address,
            nonceForWETHUnshieldApproval,
            uniswapFeeTier,
            minimumOutputAmount,
            Fr.random(),
            Fr.random(),
            deadlineForDepositingSwappedDai,
            ownerEthAddress,
            ownerEthAddress,
          )
          .simulate(),
      ).rejects.toThrowError('Assertion failed: input_asset address is not the same as seen in the bridge contract');
    });

    // edge cases for public flow:

    it("I don't need approval to call swap_public if I'm swapping on my own behalf", async () => {
      // 1. get tokens on l2
      await wethCrossChainHarness.mintTokensPublicOnL2(wethAmountToBridge);

      // 2. Give approval to uniswap to transfer funds to itself
      const nonceForWETHTransferApproval = new Fr(2n);
      const transferMessageHash = computeAuthWitMessageHash(
        uniswapL2Contract.address,
        wethCrossChainHarness.l2Token.methods
          .transfer_public(ownerAddress, uniswapL2Contract.address, wethAmountToBridge, nonceForWETHTransferApproval)
          .request(),
      );
      await ownerWallet.setPublicAuth(transferMessageHash, true).send().wait();

      // No approval to call `swap` but should work even without it:
      const [_, secretHashForDepositingSwappedDai] = daiCrossChainHarness.generateClaimSecret();

      const withdrawReceipt = await uniswapL2Contract.methods
        .swap_public(
          ownerAddress,
          wethCrossChainHarness.l2Bridge.address,
          wethAmountToBridge,
          daiCrossChainHarness.l2Bridge.address,
          nonceForWETHTransferApproval,
          uniswapFeeTier,
          minimumOutputAmount,
          ownerAddress,
          secretHashForDepositingSwappedDai,
          deadlineForDepositingSwappedDai,
          ownerEthAddress,
          ownerEthAddress,
          Fr.ZERO, // nonce for swap -> doesn't matter
        )
        .send()
        .wait();
      expect(withdrawReceipt.status).toBe(TxStatus.MINED);
      // check weth balance of owner on L2 (we first bridged `wethAmountToBridge` into L2 and now withdrew it!)
      await wethCrossChainHarness.expectPublicBalanceOnL2(ownerAddress, 0n);
    });

    it("someone can't call swap_public on my behalf without approval", async () => {
      // Owner approves a a user to swap_public:
      const approvedUser = AztecAddress.random();

      const nonceForWETHTransferApproval = new Fr(3n);
      const nonceForSwap = new Fr(3n);
      const secretHashForDepositingSwappedDai = new Fr(4n);
      const action = uniswapL2Contract
        .withWallet(sponsorWallet)
        .methods.swap_public(
          ownerAddress,
          wethCrossChainHarness.l2Bridge.address,
          wethAmountToBridge,
          daiCrossChainHarness.l2Bridge.address,
          nonceForWETHTransferApproval,
          uniswapFeeTier,
          minimumOutputAmount,
          ownerAddress,
          secretHashForDepositingSwappedDai,
          deadlineForDepositingSwappedDai,
          ownerEthAddress,
          ownerEthAddress,
          nonceForSwap,
        );
      const swapMessageHash = computeAuthWitMessageHash(approvedUser, action.request());
      await ownerWallet.setPublicAuth(swapMessageHash, true).send().wait();

      // Swap!
      await expect(action.simulate()).rejects.toThrowError(
        "Assertion failed: Message not authorized by account 'is_valid == true'",
      );
    });

    it("uniswap can't pull funds without transfer approval", async () => {
      // swap should fail since no transfer approval to uniswap:
      const nonceForWETHTransferApproval = new Fr(4n);

      const transferMessageHash = computeAuthWitMessageHash(
        uniswapL2Contract.address,
        wethCrossChainHarness.l2Token.methods
          .transfer_public(ownerAddress, uniswapL2Contract.address, wethAmountToBridge, nonceForWETHTransferApproval)
          .request(),
      );
      await ownerWallet.setPublicAuth(transferMessageHash, true).send().wait();

      await expect(
        uniswapL2Contract.methods
          .swap_public(
            ownerAddress,
            wethCrossChainHarness.l2Bridge.address,
            wethAmountToBridge,
            daiCrossChainHarness.l2Bridge.address,
            new Fr(420), // using a different nonce
            uniswapFeeTier,
            minimumOutputAmount,
            ownerAddress,
            Fr.random(),
            deadlineForDepositingSwappedDai,
            ownerEthAddress,
            ownerEthAddress,
            Fr.ZERO,
          )
          .simulate(),
      ).rejects.toThrowError(`Assertion failed: Message not authorized by account 'is_valid == true'`);
    });

    // tests when trying to mix private and public flows:
    it("can't call swap_public on L1 if called swap_private on L2", async () => {
      // get tokens on L2:
      const [secretForRedeemingWeth, secretHashForRedeemingWeth] = wethCrossChainHarness.generateClaimSecret();
      logger('minting weth on L2');
      await wethCrossChainHarness.mintTokensPrivateOnL2(wethAmountToBridge, secretHashForRedeemingWeth);
      await wethCrossChainHarness.redeemShieldPrivatelyOnL2(wethAmountToBridge, secretForRedeemingWeth);

      // Owner gives uniswap approval to unshield funds to self on its behalf
      logger('Approving uniswap to unshield funds to self on my behalf');
      const nonceForWETHUnshieldApproval = new Fr(4n);

      const unshieldToUniswapMessageHash = computeAuthWitMessageHash(
        uniswapL2Contract.address,
        wethCrossChainHarness.l2Token.methods
          .unshield(ownerAddress, uniswapL2Contract.address, wethAmountToBridge, nonceForWETHUnshieldApproval)
          .request(),
      );
      await ownerWallet.createAuthWitness(unshieldToUniswapMessageHash);
      const wethL2BalanceBeforeSwap = await wethCrossChainHarness.getL2PrivateBalanceOf(ownerAddress);

      // Swap
      logger('Withdrawing weth to L1 and sending message to swap to dai');
      const secretHashForDepositingSwappedDai = Fr.random();

      const withdrawReceipt = await uniswapL2Contract.methods
        .swap_private(
          wethCrossChainHarness.l2Token.address,
          wethCrossChainHarness.l2Bridge.address,
          wethAmountToBridge,
          daiCrossChainHarness.l2Bridge.address,
          nonceForWETHUnshieldApproval,
          uniswapFeeTier,
          minimumOutputAmount,
          Fr.random(),
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

      // On L1 call swap_public!
      logger('call swap_public on L1');
      const swapArgs = [
        wethCrossChainHarness.tokenPortalAddress.toString(),
        wethAmountToBridge,
        uniswapFeeTier,
        daiCrossChainHarness.tokenPortalAddress.toString(),
        minimumOutputAmount,
        ownerAddress.toString(),
        secretHashForDepositingSwappedDai.toString(),
        deadlineForDepositingSwappedDai,
        ownerEthAddress.toString(),
        true,
      ] as const;
      await expect(
        uniswapPortal.simulate.swapPublic(swapArgs, {
          account: ownerEthAddress.toString(),
        } as any),
      ).rejects.toThrowError('The contract function "swapPublic" reverted.');
    });

    it("can't call swap_private on L1 if called swap_public on L2", async () => {
      // get tokens on L2:
      await wethCrossChainHarness.mintTokensPublicOnL2(wethAmountToBridge);

      // Owner gives uniswap approval to transfer funds on its behalf
      const nonceForWETHTransferApproval = new Fr(5n);
      const transferMessageHash = computeAuthWitMessageHash(
        uniswapL2Contract.address,
        wethCrossChainHarness.l2Token.methods
          .transfer_public(ownerAddress, uniswapL2Contract.address, wethAmountToBridge, nonceForWETHTransferApproval)
          .request(),
      );
      await ownerWallet.setPublicAuth(transferMessageHash, true).send().wait();

      // Call swap_public on L2
      const secretHashForDepositingSwappedDai = Fr.random();
      const withdrawReceipt = await uniswapL2Contract.methods
        .swap_public(
          ownerAddress,
          wethCrossChainHarness.l2Bridge.address,
          wethAmountToBridge,
          daiCrossChainHarness.l2Bridge.address,
          nonceForWETHTransferApproval,
          uniswapFeeTier,
          minimumOutputAmount,
          ownerAddress,
          secretHashForDepositingSwappedDai,
          deadlineForDepositingSwappedDai,
          ownerEthAddress,
          ownerEthAddress,
          Fr.ZERO,
        )
        .send()
        .wait();
      expect(withdrawReceipt.status).toBe(TxStatus.MINED);
      // check weth balance of owner on L2 (we first bridged `wethAmountToBridge` into L2 and now withdrew it!)
      await wethCrossChainHarness.expectPublicBalanceOnL2(ownerAddress, 0n);

      // Call swap_private on L1
      const secretHashForRedeemingDai = Fr.random(); // creating my own secret hash
      logger('Execute withdraw and swap on the uniswapPortal!');
      const swapArgs = [
        wethCrossChainHarness.tokenPortalAddress.toString(),
        wethAmountToBridge,
        uniswapFeeTier,
        daiCrossChainHarness.tokenPortalAddress.toString(),
        minimumOutputAmount,
        secretHashForRedeemingDai.toString(),
        secretHashForDepositingSwappedDai.toString(),
        deadlineForDepositingSwappedDai,
        ownerEthAddress.toString(),
        true,
      ] as const;
      await expect(
        uniswapPortal.simulate.swapPrivate(swapArgs, {
          account: ownerEthAddress.toString(),
        } as any),
      ).rejects.toThrowError('The contract function "swapPrivate" reverted.');
    });
  });
};
