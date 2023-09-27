import { AztecNodeService } from '@aztec/aztec-node';
import { AztecAddress, CheatCodes, Fr, Wallet } from '@aztec/aztec.js';
import { DeployL1Contracts, deployL1Contract } from '@aztec/ethereum';
import { EthAddress } from '@aztec/foundation/eth-address';
import { DebugLogger } from '@aztec/foundation/log';
import { UniswapPortalAbi, UniswapPortalBytecode } from '@aztec/l1-artifacts';
import { UniswapContract } from '@aztec/noir-contracts/types';
import { CompleteAddress, PXE, TxStatus } from '@aztec/types';

import { getContract, parseEther } from 'viem';

import { CrossChainTestHarness } from './fixtures/cross_chain_test_harness.js';
import { delay, setup } from './fixtures/utils.js';

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
// TODO(2167) - Fix this! Adapt to new portal standard and new cross chain harness.
describe.skip('uniswap_trade_on_l1_from_l2', () => {
  const WETH9_ADDRESS: EthAddress = EthAddress.fromString('0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2');
  const DAI_ADDRESS: EthAddress = EthAddress.fromString('0x6B175474E89094C44Da98b954EedeAC495271d0F');

  let aztecNode: AztecNodeService | undefined;
  let pxe: PXE;
  let wallet: Wallet;
  let accounts: CompleteAddress[];
  let logger: DebugLogger;
  let cheatCodes: CheatCodes;
  let teardown: () => Promise<void>;

  let ethAccount: EthAddress;
  let owner: AztecAddress;
  const initialBalance = 10n;
  const wethAmountToBridge = parseEther('1');

  let daiCrossChainHarness: CrossChainTestHarness;
  let wethCrossChainHarness: CrossChainTestHarness;

  let uniswapPortal: any;
  let uniswapPortalAddress: EthAddress;
  let uniswapL2Contract: UniswapContract;

  beforeEach(async () => {
    let deployL1ContractsValues: DeployL1Contracts;
    ({ teardown, aztecNode, pxe, deployL1ContractsValues, accounts, logger, wallet, cheatCodes } = await setup(
      2,
      dumpedState,
    ));

    const walletClient = deployL1ContractsValues.walletClient;
    const publicClient = deployL1ContractsValues.publicClient;

    if (Number(await publicClient.getBlockNumber()) < EXPECTED_FORKED_BLOCK) {
      throw new Error('This test must be run on a fork of mainnet with the expected fork block');
    }

    ethAccount = EthAddress.fromString((await walletClient.getAddresses())[0]);
    owner = accounts[0].address;

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
      initialBalance,
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
      initialBalance,
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
    const tx = UniswapContract.deploy(pxe).send({ portalContract: uniswapPortalAddress });
    await tx.isMined({ interval: 0.1 });
    const receipt = await tx.getReceipt();
    expect(receipt.status).toEqual(TxStatus.MINED);
    uniswapL2Contract = await UniswapContract.at(receipt.contractAddress!, wallet);
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
    const meBeforeBalance = await wethCrossChainHarness.getL1BalanceOf(ethAccount);

    // 1. Approve and deposit weth to the portal and move to L2
    const [secretForL2MessageConsumption, secretHashForL2MessageConsumption] =
      await wethCrossChainHarness.generateClaimSecret();
    const [, secretHashForRedeemingMintedNotes] = await wethCrossChainHarness.generateClaimSecret();

    const messageKey = await wethCrossChainHarness.sendTokensToPortalPrivate(
      wethAmountToBridge,
      secretHashForL2MessageConsumption,
      secretHashForRedeemingMintedNotes,
    );
    expect(await wethCrossChainHarness.getL1BalanceOf(ethAccount)).toBe(meBeforeBalance - wethAmountToBridge);

    // Wait for the archiver to process the message
    await delay(5000);

    // send a transfer tx to force through rollup with the message included
    const transferAmount = 1n;
    await wethCrossChainHarness.performL2Transfer(transferAmount);

    // 3. Claim WETH on L2
    logger('Minting weth on L2');
    await wethCrossChainHarness.consumeMessageOnAztecAndMintSecretly(
      wethAmountToBridge,
      messageKey,
      secretForL2MessageConsumption,
      secretHashForRedeemingMintedNotes,
    );
    await wethCrossChainHarness.expectPrivateBalanceOnL2(owner, wethAmountToBridge + initialBalance - transferAmount);

    // Store balances
    const wethBalanceBeforeSwap = await wethCrossChainHarness.getL2PrivateBalanceOf(owner);
    const daiBalanceBeforeSwap = await daiCrossChainHarness.getL2PrivateBalanceOf(owner);

    // 4. Send L2 to L1 message to withdraw funds and another message to swap assets.
    logger('Send L2 tx to withdraw WETH to uniswap portal and send message to swap assets on L1');
    const minimumOutputAmount = 0;

    const withdrawTx = uniswapL2Contract.methods
      .swap(
        wethCrossChainHarness.l2Token.address.toField(),
        wethAmountToBridge,
        new Fr(3000),
        daiCrossChainHarness.l2Token.address.toField(),
        new Fr(minimumOutputAmount),
        owner,
        owner,
        secretHashForL2MessageConsumption,
        new Fr(2 ** 32 - 1),
        ethAccount.toField(),
        ethAccount.toField(),
      )
      .send();
    await withdrawTx.isMined({ interval: 0.1 });
    const withdrawReceipt = await withdrawTx.getReceipt();
    expect(withdrawReceipt.status).toBe(TxStatus.MINED);

    // check weth balance of owner on L2 (we first bridged `wethAmountToBridge` into L2 and now withdrew it!)
    await wethCrossChainHarness.expectPrivateBalanceOnL2(owner, initialBalance - transferAmount);

    // 5. Consume L2 to L1 message by calling uniswapPortal.swap()
    logger('Execute withdraw and swap on the uniswapPortal!');
    const daiBalanceOfPortalBefore = await daiCrossChainHarness.getL1BalanceOf(daiCrossChainHarness.tokenPortalAddress);
    const deadline = 2 ** 32 - 1; // max uint32 - 1
    const swapArgs = [
      wethCrossChainHarness.tokenPortalAddress.toString(),
      wethAmountToBridge,
      3000,
      daiCrossChainHarness.tokenPortalAddress.toString(),
      minimumOutputAmount,
      owner.toString(),
      secretHashForL2MessageConsumption.toString(true),
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
    const daiBalanceOfPortalAfter = await daiCrossChainHarness.getL1BalanceOf(daiCrossChainHarness.tokenPortalAddress);
    expect(daiBalanceOfPortalAfter).toBeGreaterThan(daiBalanceOfPortalBefore);
    const daiAmountToBridge = BigInt(daiBalanceOfPortalAfter - daiBalanceOfPortalBefore);

    // Wait for the archiver to process the message
    await delay(5000);
    // send a transfer tx to force through rollup with the message included
    await wethCrossChainHarness.performL2Transfer(transferAmount);

    // 6. claim dai on L2
    logger('Consuming messages to mint dai on L2');
    await daiCrossChainHarness.consumeMessageOnAztecAndMintSecretly(
      daiAmountToBridge,
      depositDaiMessageKey,
      secretForL2MessageConsumption,
      secretHashForRedeemingMintedNotes,
    );
    await daiCrossChainHarness.expectPrivateBalanceOnL2(owner, initialBalance + daiAmountToBridge);

    const wethBalanceAfterSwap = await wethCrossChainHarness.getL2PrivateBalanceOf(owner);
    const daiBalanceAfterSwap = await daiCrossChainHarness.getL2PrivateBalanceOf(owner);

    logger('WETH balance before swap: ', wethBalanceBeforeSwap.toString());
    logger('DAI balance before swap  : ', daiBalanceBeforeSwap.toString());
    logger('***** 🧚‍♀️ SWAP L2 assets on L1 Uniswap 🧚‍♀️ *****');
    logger('WETH balance after swap : ', wethBalanceAfterSwap.toString());
    logger('DAI balance after swap  : ', daiBalanceAfterSwap.toString());
  }, 240_000);
});
