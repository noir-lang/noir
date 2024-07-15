import {
  type AccountWallet,
  AztecAddress,
  type AztecNode,
  type DebugLogger,
  EthAddress,
  Fr,
  type PXE,
  computeAuthWitMessageHash,
} from '@aztec/aztec.js';
import { deployL1Contract } from '@aztec/ethereum';
import { sha256ToField } from '@aztec/foundation/crypto';
import { InboxAbi, UniswapPortalAbi, UniswapPortalBytecode } from '@aztec/l1-artifacts';
import { UniswapContract } from '@aztec/noir-contracts.js/Uniswap';

import { jest } from '@jest/globals';
import {
  type Account,
  type Chain,
  type GetContractReturnType,
  type HttpTransport,
  type PublicClient,
  type WalletClient,
  decodeEventLog,
  getContract,
  parseEther,
  toFunctionSelector,
} from 'viem';

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
  /** Aztec Node instance */
  aztecNode: AztecNode;
  /** The Private eXecution Environment (PXE). */
  pxe: PXE;
  /** Logger instance named as the current test. */
  logger: DebugLogger;
  /** Viem Public client instance. */
  publicClient: PublicClient<HttpTransport, Chain>;
  /** Viem Wallet Client instance. */
  walletClient: WalletClient<HttpTransport, Chain, Account>;
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

    let aztecNode: AztecNode;
    let pxe: PXE;
    let logger: DebugLogger;

    let walletClient: WalletClient<HttpTransport, Chain, Account>;
    let publicClient: PublicClient<HttpTransport, Chain>;

    let ownerWallet: AccountWallet;
    let ownerAddress: AztecAddress;
    let ownerEthAddress: EthAddress;
    // does transactions on behalf of owner on Aztec:
    let sponsorWallet: AccountWallet;
    // let sponsorAddress: AztecAddress;

    let daiCrossChainHarness: CrossChainTestHarness;
    let wethCrossChainHarness: CrossChainTestHarness;

    let uniswapPortal: GetContractReturnType<typeof UniswapPortalAbi, WalletClient<HttpTransport, Chain>>;
    let uniswapPortalAddress: EthAddress;
    let uniswapL2Contract: UniswapContract;

    const wethAmountToBridge = parseEther('1');
    const uniswapFeeTier = 3000n;
    const minimumOutputAmount = 0n;

    beforeAll(async () => {
      ({ aztecNode, pxe, logger, publicClient, walletClient, ownerWallet, sponsorWallet } = await setup());

      if (Number(await publicClient.getBlockNumber()) < expectedForkBlockNumber) {
        throw new Error('This test must be run on a fork of mainnet with the expected fork block');
      }

      ownerAddress = ownerWallet.getAddress();
      // sponsorAddress = sponsorWallet.getAddress();
      ownerEthAddress = EthAddress.fromString((await walletClient.getAddresses())[0]);

      await publicDeployAccounts(ownerWallet, [ownerWallet, sponsorWallet]);

      logger.info('Deploying DAI Portal, initializing and deploying l2 contract...');
      daiCrossChainHarness = await CrossChainTestHarness.new(
        aztecNode,
        pxe,
        publicClient,
        walletClient,
        ownerWallet,
        logger,
        DAI_ADDRESS,
      );

      logger.info('Deploying WETH Portal, initializing and deploying l2 contract...');
      wethCrossChainHarness = await CrossChainTestHarness.new(
        aztecNode,
        pxe,
        publicClient,
        walletClient,
        ownerWallet,
        logger,
        WETH9_ADDRESS,
      );

      logger.info('Deploy Uniswap portal on L1 and L2...');
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
      uniswapL2Contract = await UniswapContract.deploy(ownerWallet, uniswapPortalAddress).send().deployed();

      const registryAddress = (await pxe.getNodeInfo()).l1ContractAddresses.registryAddress;

      await uniswapPortal.write.initialize(
        [registryAddress.toString(), uniswapL2Contract.address.toString()],
        {} as any,
      );

      // Give me some WETH so I can deposit to L2 and do the swap...
      logger.info('Getting some weth');
      const hash = await walletClient.sendTransaction({ to: WETH9_ADDRESS.toString(), value: parseEther('1000') });
      await publicClient.waitForTransactionReceipt({ hash });

      const wethBalance = await wethCrossChainHarness.getL1BalanceOf(ownerEthAddress);
      expect(wethBalance).toBe(parseEther('1000'));
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

      const tokenDepositMsgHash = await wethCrossChainHarness.sendTokensToPortalPrivate(
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

      await wethCrossChainHarness.makeMessageConsumable(tokenDepositMsgHash);

      // 2. Claim WETH on L2
      logger.info('Minting weth on L2');
      await wethCrossChainHarness.consumeMessageOnAztecAndMintPrivately(
        secretHashForRedeemingWeth,
        wethAmountToBridge,
        secretForMintingWeth,
      );
      await wethCrossChainHarness.redeemShieldPrivatelyOnL2(wethAmountToBridge, secretForRedeemingWeth);
      await wethCrossChainHarness.expectPrivateBalanceOnL2(ownerAddress, wethAmountToBridge);

      // Store balances
      const wethL2BalanceBeforeSwap = await wethCrossChainHarness.getL2PrivateBalanceOf(ownerAddress);
      const daiL2BalanceBeforeSwap = await daiCrossChainHarness.getL2PrivateBalanceOf(ownerAddress);

      // 3. Owner gives uniswap approval to unshield funds to self on its behalf
      logger.info('Approving uniswap to unshield funds to self on my behalf');
      const nonceForWETHUnshieldApproval = new Fr(1n);
      await ownerWallet.createAuthWit({
        caller: uniswapL2Contract.address,
        action: wethCrossChainHarness.l2Token.methods.unshield(
          ownerAddress,
          uniswapL2Contract.address,
          wethAmountToBridge,
          nonceForWETHUnshieldApproval,
        ),
      });

      // 4. Swap on L1 - sends L2 to L1 message to withdraw WETH to L1 and another message to swap assets.
      logger.info('Withdrawing weth to L1 and sending message to swap to dai');
      const [secretForDepositingSwappedDai, secretHashForDepositingSwappedDai] =
        daiCrossChainHarness.generateClaimSecret();
      const [secretForRedeemingDai, secretHashForRedeemingDai] = daiCrossChainHarness.generateClaimSecret();

      const l2UniswapInteractionReceipt = await uniswapL2Contract.methods
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
          ownerEthAddress,
        )
        .send()
        .wait();

      const swapPrivateContent = sha256ToField([
        Buffer.from(
          toFunctionSelector('swap_private(address,uint256,uint24,address,uint256,bytes32,bytes32,address)').substring(
            2,
          ),
          'hex',
        ),
        wethCrossChainHarness.tokenPortalAddress.toBuffer32(),
        new Fr(wethAmountToBridge),
        new Fr(uniswapFeeTier),
        daiCrossChainHarness.tokenPortalAddress.toBuffer32(),
        new Fr(minimumOutputAmount),
        secretHashForRedeemingDai,
        secretHashForDepositingSwappedDai,
        ownerEthAddress.toBuffer32(),
      ]);

      const swapPrivateLeaf = sha256ToField([
        uniswapL2Contract.address,
        new Fr(1), // aztec version
        EthAddress.fromString(uniswapPortal.address).toBuffer32(),
        new Fr(publicClient.chain.id), // chain id
        swapPrivateContent,
      ]);

      const withdrawContent = sha256ToField([
        Buffer.from(toFunctionSelector('withdraw(address,uint256,address)').substring(2), 'hex'),
        uniswapPortalAddress.toBuffer32(),
        new Fr(wethAmountToBridge),
        uniswapPortalAddress.toBuffer32(),
      ]);

      const withdrawLeaf = sha256ToField([
        wethCrossChainHarness.l2Bridge.address,
        new Fr(1), // aztec version
        wethCrossChainHarness.tokenPortalAddress.toBuffer32(),
        new Fr(publicClient.chain.id), // chain id
        withdrawContent,
      ]);

      // ensure that user's funds were burnt
      await wethCrossChainHarness.expectPrivateBalanceOnL2(ownerAddress, wethL2BalanceBeforeSwap - wethAmountToBridge);
      // ensure that uniswap contract didn't eat the funds.
      await wethCrossChainHarness.expectPublicBalanceOnL2(uniswapL2Contract.address, 0n);

      // 5. Consume L2 to L1 message by calling uniswapPortal.swap_private()
      logger.info('Execute withdraw and swap on the uniswapPortal!');
      const daiL1BalanceOfPortalBeforeSwap = await daiCrossChainHarness.getL1BalanceOf(
        daiCrossChainHarness.tokenPortalAddress,
      );

      const [swapPrivateL2MessageIndex, swapPrivateSiblingPath] = await aztecNode.getL2ToL1MessageMembershipWitness(
        l2UniswapInteractionReceipt.blockNumber!,
        swapPrivateLeaf,
      );
      const [withdrawL2MessageIndex, withdrawSiblingPath] = await aztecNode.getL2ToL1MessageMembershipWitness(
        l2UniswapInteractionReceipt.blockNumber!,
        withdrawLeaf,
      );

      const withdrawMessageMetadata = {
        _l2BlockNumber: BigInt(l2UniswapInteractionReceipt.blockNumber!),
        _leafIndex: BigInt(withdrawL2MessageIndex),
        _path: withdrawSiblingPath
          .toBufferArray()
          .map((buf: Buffer) => `0x${buf.toString('hex')}`) as readonly `0x${string}`[],
      };

      const swapPrivateMessageMetadata = {
        _l2BlockNumber: BigInt(l2UniswapInteractionReceipt.blockNumber!),
        _leafIndex: BigInt(swapPrivateL2MessageIndex),
        _path: swapPrivateSiblingPath
          .toBufferArray()
          .map((buf: Buffer) => `0x${buf.toString('hex')}`) as readonly `0x${string}`[],
      };

      const swapArgs = [
        wethCrossChainHarness.tokenPortalAddress.toString(),
        wethAmountToBridge,
        Number(uniswapFeeTier),
        daiCrossChainHarness.tokenPortalAddress.toString(),
        minimumOutputAmount,
        secretHashForRedeemingDai.toString(),
        secretHashForDepositingSwappedDai.toString(),
        true,
        [withdrawMessageMetadata, swapPrivateMessageMetadata],
      ] as const;

      // this should also insert a message into the inbox.
      const txHash = await uniswapPortal.write.swapPrivate(swapArgs, {} as any);

      // We get the msg leaf from event so that we can later wait for it to be available for consumption
      let tokenOutMsgHash: Fr;
      {
        const txReceipt = await daiCrossChainHarness.publicClient.waitForTransactionReceipt({
          hash: txHash,
        });

        const txLog = txReceipt.logs[9];
        const topics = decodeEventLog({
          abi: InboxAbi,
          data: txLog.data,
          topics: txLog.topics,
        });
        tokenOutMsgHash = Fr.fromString(topics.args.hash);
      }

      // weth was swapped to dai and send to portal
      const daiL1BalanceOfPortalAfter = await daiCrossChainHarness.getL1BalanceOf(
        daiCrossChainHarness.tokenPortalAddress,
      );
      expect(daiL1BalanceOfPortalAfter).toBeGreaterThan(daiL1BalanceOfPortalBeforeSwap);
      const daiAmountToBridge = BigInt(daiL1BalanceOfPortalAfter - daiL1BalanceOfPortalBeforeSwap);

      // Wait for the message to be available for consumption
      await daiCrossChainHarness.makeMessageConsumable(tokenOutMsgHash);

      // 6. claim dai on L2
      logger.info('Consuming messages to mint dai on L2');
      await daiCrossChainHarness.consumeMessageOnAztecAndMintPrivately(
        secretHashForRedeemingDai,
        daiAmountToBridge,
        secretForDepositingSwappedDai,
      );
      await daiCrossChainHarness.redeemShieldPrivatelyOnL2(daiAmountToBridge, secretForRedeemingDai);
      await daiCrossChainHarness.expectPrivateBalanceOnL2(ownerAddress, daiL2BalanceBeforeSwap + daiAmountToBridge);

      const wethL2BalanceAfterSwap = await wethCrossChainHarness.getL2PrivateBalanceOf(ownerAddress);
      const daiL2BalanceAfterSwap = await daiCrossChainHarness.getL2PrivateBalanceOf(ownerAddress);

      logger.info('WETH balance before swap: ' + wethL2BalanceBeforeSwap.toString());
      logger.info('DAI balance before swap  : ' + daiL2BalanceBeforeSwap.toString());
      logger.info('***** 🧚‍♀️ SWAP L2 assets on L1 Uniswap 🧚‍♀️ *****');
      logger.info('WETH balance after swap : ', wethL2BalanceAfterSwap.toString());
      logger.info('DAI balance after swap  : ', daiL2BalanceAfterSwap.toString());
    });
    // docs:end:uniswap_private

    // TODO(#7463): reenable look into this failure https://github.com/AztecProtocol/aztec-packages/actions/runs/9912612912/job/27388320150?pr=7462
    // // docs:start:uniswap_public
    // it('should uniswap trade on L1 from L2 funds publicly (swaps WETH -> DAI)', async () => {
    //   const wethL1BeforeBalance = await wethCrossChainHarness.getL1BalanceOf(ownerEthAddress);

    //   // 1. Approve and deposit weth to the portal and move to L2
    //   const [secretForMintingWeth, secretHashForMintingWeth] = wethCrossChainHarness.generateClaimSecret();

    //   const wethDepositMsgHash = await wethCrossChainHarness.sendTokensToPortalPublic(
    //     wethAmountToBridge,
    //     secretHashForMintingWeth,
    //   );
    //   // funds transferred from owner to token portal
    //   expect(await wethCrossChainHarness.getL1BalanceOf(ownerEthAddress)).toBe(
    //     wethL1BeforeBalance - wethAmountToBridge,
    //   );
    //   expect(await wethCrossChainHarness.getL1BalanceOf(wethCrossChainHarness.tokenPortalAddress)).toBe(
    //     wethAmountToBridge,
    //   );

    //   // Wait for the message to be available for consumption
    //   await wethCrossChainHarness.makeMessageConsumable(wethDepositMsgHash);

    //   // Get message leaf index, needed for claiming in public
    //   const wethDepositMaybeIndexAndPath = await aztecNode.getL1ToL2MessageMembershipWitness(
    //     'latest',
    //     wethDepositMsgHash,
    //     0n,
    //   );
    //   assert(wethDepositMaybeIndexAndPath !== undefined, 'Message not found in tree');
    //   const wethDepositMessageLeafIndex = wethDepositMaybeIndexAndPath[0];

    //   // 2. Claim WETH on L2
    //   logger.info('Minting weth on L2');
    //   await wethCrossChainHarness.consumeMessageOnAztecAndMintPublicly(
    //     wethAmountToBridge,
    //     secretForMintingWeth,
    //     wethDepositMessageLeafIndex,
    //   );
    //   await wethCrossChainHarness.expectPublicBalanceOnL2(ownerAddress, wethAmountToBridge);

    //   // Store balances
    //   const wethL2BalanceBeforeSwap = await wethCrossChainHarness.getL2PublicBalanceOf(ownerAddress);
    //   const daiL2BalanceBeforeSwap = await daiCrossChainHarness.getL2PublicBalanceOf(ownerAddress);

    //   // 3. Owner gives uniswap approval to transfer funds on its behalf
    //   const nonceForWETHTransferApproval = new Fr(1n);

    //   await ownerWallet
    //     .setPublicAuthWit(
    //       {
    //         caller: uniswapL2Contract.address,
    //         action: wethCrossChainHarness.l2Token.methods
    //           .transfer_public(
    //             ownerAddress,
    //             uniswapL2Contract.address,
    //             wethAmountToBridge,
    //             nonceForWETHTransferApproval,
    //           )
    //           .request(),
    //       },
    //       true,
    //     )
    //     .send()
    //     .wait();

    //   // 4. Swap on L1 - sends L2 to L1 message to withdraw WETH to L1 and another message to swap assets.
    //   const [secretForDepositingSwappedDai, secretHashForDepositingSwappedDai] =
    //     daiCrossChainHarness.generateClaimSecret();

    //   // 4.1 Owner approves user to swap on their behalf:
    //   const nonceForSwap = new Fr(3n);
    //   const action = uniswapL2Contract
    //     .withWallet(sponsorWallet)
    //     .methods.swap_public(
    //       ownerAddress,
    //       wethCrossChainHarness.l2Bridge.address,
    //       wethAmountToBridge,
    //       daiCrossChainHarness.l2Bridge.address,
    //       nonceForWETHTransferApproval,
    //       uniswapFeeTier,
    //       minimumOutputAmount,
    //       ownerAddress,
    //       secretHashForDepositingSwappedDai,
    //       ownerEthAddress,
    //       nonceForSwap,
    //     );
    //   await ownerWallet.setPublicAuthWit({ caller: sponsorAddress, action }, true).send().wait();

    //   // 4.2 Call swap_public from user2 on behalf of owner
    //   const uniswapL2Interaction = await action.send().wait();

    //   const swapPublicContent = sha256ToField([
    //     Buffer.from(
    //       toFunctionSelector('swap_public(address,uint256,uint24,address,uint256,bytes32,bytes32,address)').substring(
    //         2,
    //       ),
    //       'hex',
    //     ),
    //     wethCrossChainHarness.tokenPortalAddress.toBuffer32(),
    //     new Fr(wethAmountToBridge),
    //     new Fr(uniswapFeeTier),
    //     daiCrossChainHarness.tokenPortalAddress.toBuffer32(),
    //     new Fr(minimumOutputAmount),
    //     ownerAddress,
    //     secretHashForDepositingSwappedDai,
    //     ownerEthAddress.toBuffer32(),
    //   ]);

    //   const swapPublicLeaf = sha256ToField([
    //     uniswapL2Contract.address,
    //     new Fr(1), // aztec version
    //     EthAddress.fromString(uniswapPortal.address).toBuffer32(),
    //     new Fr(publicClient.chain.id), // chain id
    //     swapPublicContent,
    //   ]);

    //   const withdrawContent = sha256ToField([
    //     Buffer.from(toFunctionSelector('withdraw(address,uint256,address)').substring(2), 'hex'),
    //     uniswapPortalAddress.toBuffer32(),
    //     new Fr(wethAmountToBridge),
    //     uniswapPortalAddress.toBuffer32(),
    //   ]);

    //   const withdrawLeaf = sha256ToField([
    //     wethCrossChainHarness.l2Bridge.address,
    //     new Fr(1), // aztec version
    //     wethCrossChainHarness.tokenPortalAddress.toBuffer32(),
    //     new Fr(publicClient.chain.id), // chain id
    //     withdrawContent,
    //   ]);

    //   // check weth balance of owner on L2 (we first bridged `wethAmountToBridge` into L2 and now withdrew it!)
    //   await wethCrossChainHarness.expectPublicBalanceOnL2(ownerAddress, wethL2BalanceBeforeSwap - wethAmountToBridge);

    //   // 5. Perform the swap on L1 with the `uniswapPortal.swap_private()` (consuming L2 to L1 messages)
    //   logger.info('Execute withdraw and swap on the uniswapPortal!');
    //   const daiL1BalanceOfPortalBeforeSwap = await daiCrossChainHarness.getL1BalanceOf(
    //     daiCrossChainHarness.tokenPortalAddress,
    //   );

    //   const [swapPrivateL2MessageIndex, swapPrivateSiblingPath] = await aztecNode.getL2ToL1MessageMembershipWitness(
    //     uniswapL2Interaction.blockNumber!,
    //     swapPublicLeaf,
    //   );
    //   const [withdrawL2MessageIndex, withdrawSiblingPath] = await aztecNode.getL2ToL1MessageMembershipWitness(
    //     uniswapL2Interaction.blockNumber!,
    //     withdrawLeaf,
    //   );

    //   const withdrawMessageMetadata = {
    //     _l2BlockNumber: BigInt(uniswapL2Interaction.blockNumber!),
    //     _leafIndex: BigInt(withdrawL2MessageIndex),
    //     _path: withdrawSiblingPath
    //       .toBufferArray()
    //       .map((buf: Buffer) => `0x${buf.toString('hex')}`) as readonly `0x${string}`[],
    //   };

    //   const swapPrivateMessageMetadata = {
    //     _l2BlockNumber: BigInt(uniswapL2Interaction.blockNumber!),
    //     _leafIndex: BigInt(swapPrivateL2MessageIndex),
    //     _path: swapPrivateSiblingPath
    //       .toBufferArray()
    //       .map((buf: Buffer) => `0x${buf.toString('hex')}`) as readonly `0x${string}`[],
    //   };

    //   const swapArgs = [
    //     wethCrossChainHarness.tokenPortalAddress.toString(),
    //     wethAmountToBridge,
    //     Number(uniswapFeeTier),
    //     daiCrossChainHarness.tokenPortalAddress.toString(),
    //     minimumOutputAmount,
    //     ownerAddress.toString(),
    //     secretHashForDepositingSwappedDai.toString(),
    //     true,
    //     [withdrawMessageMetadata, swapPrivateMessageMetadata],
    //   ] as const;

    //   // this should also insert a message into the inbox.
    //   const txHash = await uniswapPortal.write.swapPublic(swapArgs, {} as any);

    //   // We get the msg leaf from event so that we can later wait for it to be available for consumption
    //   let outTokenDepositMsgHash: Fr;
    //   {
    //     const txReceipt = await daiCrossChainHarness.publicClient.waitForTransactionReceipt({
    //       hash: txHash,
    //     });

    //     const txLog = txReceipt.logs[9];
    //     const topics = decodeEventLog({
    //       abi: InboxAbi,
    //       data: txLog.data,
    //       topics: txLog.topics,
    //     });
    //     outTokenDepositMsgHash = Fr.fromString(topics.args.hash);
    //   }

    //   // weth was swapped to dai and send to portal
    //   const daiL1BalanceOfPortalAfter = await daiCrossChainHarness.getL1BalanceOf(
    //     daiCrossChainHarness.tokenPortalAddress,
    //   );
    //   expect(daiL1BalanceOfPortalAfter).toBeGreaterThan(daiL1BalanceOfPortalBeforeSwap);
    //   const daiAmountToBridge = BigInt(daiL1BalanceOfPortalAfter - daiL1BalanceOfPortalBeforeSwap);

    //   // Wait for the message to be available for consumption
    //   await daiCrossChainHarness.makeMessageConsumable(outTokenDepositMsgHash);

    //   // Get message leaf index, needed for claiming in public
    //   const outTokenDepositMaybeIndexAndPath = await aztecNode.getL1ToL2MessageMembershipWitness(
    //     'latest',
    //     outTokenDepositMsgHash,
    //     0n,
    //   );
    //   assert(outTokenDepositMaybeIndexAndPath !== undefined, 'Message not found in tree');
    //   const outTokenDepositMessageLeafIndex = outTokenDepositMaybeIndexAndPath[0];

    //   // 6. claim dai on L2
    //   logger.info('Consuming messages to mint dai on L2');
    //   await daiCrossChainHarness.consumeMessageOnAztecAndMintPublicly(
    //     daiAmountToBridge,
    //     secretForDepositingSwappedDai,
    //     outTokenDepositMessageLeafIndex,
    //   );
    //   await daiCrossChainHarness.expectPublicBalanceOnL2(ownerAddress, daiL2BalanceBeforeSwap + daiAmountToBridge);

    //   const wethL2BalanceAfterSwap = await wethCrossChainHarness.getL2PublicBalanceOf(ownerAddress);
    //   const daiL2BalanceAfterSwap = await daiCrossChainHarness.getL2PublicBalanceOf(ownerAddress);

    //   logger.info('WETH balance before swap: ', wethL2BalanceBeforeSwap.toString());
    //   logger.info('DAI balance before swap  : ', daiL2BalanceBeforeSwap.toString());
    //   logger.info('***** 🧚‍♀️ SWAP L2 assets on L1 Uniswap 🧚‍♀️ *****');
    //   logger.info('WETH balance after swap : ', wethL2BalanceAfterSwap.toString());
    //   logger.info('DAI balance after swap  : ', daiL2BalanceAfterSwap.toString());
    // });
    // // docs:end:uniswap_public

    // Edge cases for the private flow:
    // note - tests for uniswapPortal.sol and minting asset on L2 are covered in other tests.

    it('swap_private reverts without unshield approval', async () => {
      // swap should fail since no withdraw approval to uniswap:
      const nonceForWETHUnshieldApproval = new Fr(2n);

      const expectedMessageHash = computeAuthWitMessageHash(
        {
          caller: uniswapL2Contract.address,
          action: wethCrossChainHarness.l2Token.methods
            .unshield(ownerAddress, uniswapL2Contract.address, wethAmountToBridge, nonceForWETHUnshieldApproval)
            .request(),
        },
        { chainId: ownerWallet.getChainId(), version: ownerWallet.getVersion() },
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
            ownerEthAddress,
          )
          .prove(),
      ).rejects.toThrow(`Unknown auth witness for message hash ${expectedMessageHash.toString()}`);
    });

    it("can't swap if user passes a token different to what the bridge tracks", async () => {
      // 1. give user private funds on L2:
      const [secretForRedeemingWeth, secretHashForRedeemingWeth] = wethCrossChainHarness.generateClaimSecret();
      await wethCrossChainHarness.mintTokensPrivateOnL2(wethAmountToBridge, secretHashForRedeemingWeth);
      await wethCrossChainHarness.redeemShieldPrivatelyOnL2(wethAmountToBridge, secretForRedeemingWeth);
      await wethCrossChainHarness.expectPrivateBalanceOnL2(ownerAddress, wethAmountToBridge);

      // 2. owner gives uniswap approval to unshield funds:
      logger.info('Approving uniswap to unshield funds to self on my behalf');
      const nonceForWETHUnshieldApproval = new Fr(3n);
      await ownerWallet.createAuthWit({
        caller: uniswapL2Contract.address,
        action: wethCrossChainHarness.l2Token.methods.unshield(
          ownerAddress,
          uniswapL2Contract.address,
          wethAmountToBridge,
          nonceForWETHUnshieldApproval,
        ),
      });

      // 3. Swap but send the wrong token address
      logger.info('Swap but send the wrong token address');
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
            ownerEthAddress,
          )
          .prove(),
      ).rejects.toThrow('Assertion failed: input_asset address is not the same as seen in the bridge contract');
    });

    // edge cases for public flow:

    it("I don't need approval to call swap_public if I'm swapping on my own behalf", async () => {
      // 1. get tokens on l2
      await wethCrossChainHarness.mintTokensPublicOnL2(wethAmountToBridge);

      // 2. Give approval to uniswap to transfer funds to itself
      const nonceForWETHTransferApproval = new Fr(2n);
      await ownerWallet
        .setPublicAuthWit(
          {
            caller: uniswapL2Contract.address,
            action: wethCrossChainHarness.l2Token.methods
              .transfer_public(
                ownerAddress,
                uniswapL2Contract.address,
                wethAmountToBridge,
                nonceForWETHTransferApproval,
              )
              .request(),
          },
          true,
        )
        .send()
        .wait();

      // No approval to call `swap` but should work even without it:
      const [_, secretHashForDepositingSwappedDai] = daiCrossChainHarness.generateClaimSecret();

      await uniswapL2Contract.methods
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
          ownerEthAddress,
          Fr.ZERO, // nonce for swap -> doesn't matter
        )
        .send()
        .wait();
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
          ownerEthAddress,
          nonceForSwap,
        );
      await ownerWallet.setPublicAuthWit({ caller: approvedUser, action }, true).send().wait();

      await expect(action.simulate()).rejects.toThrow(/unauthorized/);
    });

    it("uniswap can't pull funds without transfer approval", async () => {
      // swap should fail since no transfer approval to uniswap:
      const nonceForWETHTransferApproval = new Fr(4n);

      await ownerWallet
        .setPublicAuthWit(
          {
            caller: uniswapL2Contract.address,
            action: wethCrossChainHarness.l2Token.methods
              .transfer_public(
                ownerAddress,
                uniswapL2Contract.address,
                wethAmountToBridge,
                nonceForWETHTransferApproval,
              )
              .request(),
          },
          true,
        )
        .send()
        .wait();

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
            ownerEthAddress,
            Fr.ZERO,
          )
          .simulate(),
      ).rejects.toThrow(/unauthorized/);
    });

    // tests when trying to mix private and public flows:
    it("can't call swap_public on L1 if called swap_private on L2", async () => {
      // get tokens on L2:
      const [secretForRedeemingWeth, secretHashForRedeemingWeth] = wethCrossChainHarness.generateClaimSecret();
      logger.info('minting weth on L2');
      await wethCrossChainHarness.mintTokensPrivateOnL2(wethAmountToBridge, secretHashForRedeemingWeth);
      await wethCrossChainHarness.redeemShieldPrivatelyOnL2(wethAmountToBridge, secretForRedeemingWeth);

      // Owner gives uniswap approval to unshield funds to self on its behalf
      logger.info('Approving uniswap to unshield funds to self on my behalf');
      const nonceForWETHUnshieldApproval = new Fr(4n);
      await ownerWallet.createAuthWit({
        caller: uniswapL2Contract.address,
        action: wethCrossChainHarness.l2Token.methods.unshield(
          ownerAddress,
          uniswapL2Contract.address,
          wethAmountToBridge,
          nonceForWETHUnshieldApproval,
        ),
      });
      const wethL2BalanceBeforeSwap = await wethCrossChainHarness.getL2PrivateBalanceOf(ownerAddress);

      // Swap
      logger.info('Withdrawing weth to L1 and sending message to swap to dai');
      const [, secretHashForRedeemingDai] = daiCrossChainHarness.generateClaimSecret();

      const [, secretHashForDepositingSwappedDai] = daiCrossChainHarness.generateClaimSecret();
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
          ownerEthAddress,
        )
        .send()
        .wait();

      const swapPrivateContent = sha256ToField([
        Buffer.from(
          toFunctionSelector('swap_private(address,uint256,uint24,address,uint256,bytes32,bytes32,address)').substring(
            2,
          ),
          'hex',
        ),
        wethCrossChainHarness.tokenPortalAddress.toBuffer32(),
        new Fr(wethAmountToBridge),
        new Fr(uniswapFeeTier),
        daiCrossChainHarness.tokenPortalAddress.toBuffer32(),
        new Fr(minimumOutputAmount),
        secretHashForRedeemingDai,
        secretHashForDepositingSwappedDai,
        ownerEthAddress.toBuffer32(),
      ]);

      const swapPrivateLeaf = sha256ToField([
        uniswapL2Contract.address,
        new Fr(1), // aztec version
        EthAddress.fromString(uniswapPortal.address).toBuffer32(),
        new Fr(publicClient.chain.id), // chain id
        swapPrivateContent,
      ]);

      const withdrawContent = sha256ToField([
        Buffer.from(toFunctionSelector('withdraw(address,uint256,address)').substring(2), 'hex'),
        uniswapPortalAddress.toBuffer32(),
        new Fr(wethAmountToBridge),
        uniswapPortalAddress.toBuffer32(),
      ]);

      const withdrawLeaf = sha256ToField([
        wethCrossChainHarness.l2Bridge.address,
        new Fr(1), // aztec version
        wethCrossChainHarness.tokenPortalAddress.toBuffer32(),
        new Fr(publicClient.chain.id), // chain id
        withdrawContent,
      ]);

      const [swapPrivateL2MessageIndex, swapPrivateSiblingPath] = await aztecNode.getL2ToL1MessageMembershipWitness(
        withdrawReceipt.blockNumber!,
        swapPrivateLeaf,
      );
      const [withdrawL2MessageIndex, withdrawSiblingPath] = await aztecNode.getL2ToL1MessageMembershipWitness(
        withdrawReceipt.blockNumber!,
        withdrawLeaf,
      );

      const withdrawMessageMetadata = {
        _l2BlockNumber: BigInt(withdrawReceipt.blockNumber!),
        _leafIndex: BigInt(withdrawL2MessageIndex),
        _path: withdrawSiblingPath
          .toBufferArray()
          .map((buf: Buffer) => `0x${buf.toString('hex')}`) as readonly `0x${string}`[],
      };

      const swapPrivateMessageMetadata = {
        _l2BlockNumber: BigInt(withdrawReceipt.blockNumber!),
        _leafIndex: BigInt(swapPrivateL2MessageIndex),
        _path: swapPrivateSiblingPath
          .toBufferArray()
          .map((buf: Buffer) => `0x${buf.toString('hex')}`) as readonly `0x${string}`[],
      };

      // ensure that user's funds were burnt
      await wethCrossChainHarness.expectPrivateBalanceOnL2(ownerAddress, wethL2BalanceBeforeSwap - wethAmountToBridge);

      // On L1 call swap_public!
      logger.info('call swap_public on L1');
      const swapArgs = [
        wethCrossChainHarness.tokenPortalAddress.toString(),
        wethAmountToBridge,
        Number(uniswapFeeTier),
        daiCrossChainHarness.tokenPortalAddress.toString(),
        minimumOutputAmount,
        ownerAddress.toString(),
        secretHashForDepositingSwappedDai.toString(),
        true,
        [withdrawMessageMetadata, swapPrivateMessageMetadata],
      ] as const;
      await expect(
        uniswapPortal.simulate.swapPublic(swapArgs, {
          account: ownerEthAddress.toString(),
        } as any),
      ).rejects.toThrow('The contract function "swapPublic" reverted');
    });

    it("can't call swap_private on L1 if called swap_public on L2", async () => {
      // get tokens on L2:
      await wethCrossChainHarness.mintTokensPublicOnL2(wethAmountToBridge);

      // Owner gives uniswap approval to transfer funds on its behalf
      const nonceForWETHTransferApproval = new Fr(5n);
      await ownerWallet
        .setPublicAuthWit(
          {
            caller: uniswapL2Contract.address,
            action: wethCrossChainHarness.l2Token.methods
              .transfer_public(
                ownerAddress,
                uniswapL2Contract.address,
                wethAmountToBridge,
                nonceForWETHTransferApproval,
              )
              .request(),
          },
          true,
        )
        .send()
        .wait();

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
          ownerEthAddress,
          Fr.ZERO,
        )
        .send()
        .wait();

      const swapPublicContent = sha256ToField([
        Buffer.from(
          toFunctionSelector('swap_public(address,uint256,uint24,address,uint256,bytes32,bytes32,address)').substring(
            2,
          ),
          'hex',
        ),
        wethCrossChainHarness.tokenPortalAddress.toBuffer32(),
        new Fr(wethAmountToBridge),
        new Fr(uniswapFeeTier),
        daiCrossChainHarness.tokenPortalAddress.toBuffer32(),
        new Fr(minimumOutputAmount),
        ownerAddress,
        secretHashForDepositingSwappedDai,
        ownerEthAddress.toBuffer32(),
      ]);

      const swapPublicLeaf = sha256ToField([
        uniswapL2Contract.address,
        new Fr(1), // aztec version
        EthAddress.fromString(uniswapPortal.address).toBuffer32(),
        new Fr(publicClient.chain.id), // chain id
        swapPublicContent,
      ]);

      const withdrawContent = sha256ToField([
        Buffer.from(toFunctionSelector('withdraw(address,uint256,address)').substring(2), 'hex'),
        uniswapPortalAddress.toBuffer32(),
        new Fr(wethAmountToBridge),
        uniswapPortalAddress.toBuffer32(),
      ]);

      const withdrawLeaf = sha256ToField([
        wethCrossChainHarness.l2Bridge.address,
        new Fr(1), // aztec version
        wethCrossChainHarness.tokenPortalAddress.toBuffer32(),
        new Fr(publicClient.chain.id), // chain id
        withdrawContent,
      ]);

      const [swapPublicL2MessageIndex, swapPublicSiblingPath] = await aztecNode.getL2ToL1MessageMembershipWitness(
        withdrawReceipt.blockNumber!,
        swapPublicLeaf,
      );
      const [withdrawL2MessageIndex, withdrawSiblingPath] = await aztecNode.getL2ToL1MessageMembershipWitness(
        withdrawReceipt.blockNumber!,
        withdrawLeaf,
      );

      const withdrawMessageMetadata = {
        _l2BlockNumber: BigInt(withdrawReceipt.blockNumber!),
        _leafIndex: BigInt(withdrawL2MessageIndex),
        _path: withdrawSiblingPath
          .toBufferArray()
          .map((buf: Buffer) => `0x${buf.toString('hex')}`) as readonly `0x${string}`[],
      };

      const swapPublicMessageMetadata = {
        _l2BlockNumber: BigInt(withdrawReceipt.blockNumber!),
        _leafIndex: BigInt(swapPublicL2MessageIndex),
        _path: swapPublicSiblingPath
          .toBufferArray()
          .map((buf: Buffer) => `0x${buf.toString('hex')}`) as readonly `0x${string}`[],
      };

      // check weth balance of owner on L2 (we first bridged `wethAmountToBridge` into L2 and now withdrew it!)
      await wethCrossChainHarness.expectPublicBalanceOnL2(ownerAddress, 0n);

      // Call swap_private on L1
      const secretHashForRedeemingDai = Fr.random(); // creating my own secret hash
      logger.info('Execute withdraw and swap on the uniswapPortal!');

      const swapArgs = [
        wethCrossChainHarness.tokenPortalAddress.toString(),
        wethAmountToBridge,
        Number(uniswapFeeTier),
        daiCrossChainHarness.tokenPortalAddress.toString(),
        minimumOutputAmount,
        secretHashForRedeemingDai.toString(),
        secretHashForDepositingSwappedDai.toString(),
        true,
        [withdrawMessageMetadata, swapPublicMessageMetadata],
      ] as const;

      await expect(
        uniswapPortal.simulate.swapPrivate(swapArgs, {
          account: ownerEthAddress.toString(),
        } as any),
      ).rejects.toThrow('The contract function "swapPrivate" reverted.');
    });
  });
};
