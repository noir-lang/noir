import { AztecNodeService } from '@aztec/aztec-node';
import {
  AztecAddress,
  AztecRPCServer,
  Contract,
  ContractDeployer,
  Fr,
  TxStatus,
  computeMessageSecretHash,
} from '@aztec/aztec.js';
import { deployL1Contract } from '@aztec/ethereum';

import { EthAddress } from '@aztec/foundation/eth-address';
import { delay, deployAndInitializeNonNativeL2TokenContracts, pointToPublicKey, setup } from './utils.js';
import { DebugLogger } from '@aztec/foundation/log';
import { Chain, HttpTransport, PublicClient, getContract, parseEther } from 'viem';
import { DeployL1Contracts } from '@aztec/ethereum';
import { UniswapPortalAbi, UniswapPortalBytecode } from '@aztec/l1-artifacts';
import { UniswapContractAbi } from '@aztec/noir-contracts/examples';

// PSA: this works on a fork of mainnet but with the default anvil chain id. Start it with the command:
// anvil --fork-url https://mainnet.infura.io/v3/9928b52099854248b3a096be07a6b23c --fork-block-number 17514288 --chain-id 31337
// For CI, this is configured in `run_tests.sh` and `docker-compose.yml`

// Should mint WETH on L2, swap to DAI using L1 Uniswap and mint this DAI back on L2
describe('uniswap_trade_on_l1_from_l2', () => {
  // test runs on a forked version of mainnet at this block.
  const EXPECTED_FORKED_BLOCK = 17514288;
  // We tell the archiver to only sync from this block.
  process.env.SEARCH_START_BLOCK = EXPECTED_FORKED_BLOCK.toString();
  const WETH9_ADDRESS: EthAddress = EthAddress.fromString('0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2');
  const DAI_ADDRESS: EthAddress = EthAddress.fromString('0x6B175474E89094C44Da98b954EedeAC495271d0F');

  let aztecNode: AztecNodeService;
  let aztecRpcServer: AztecRPCServer;
  let accounts: AztecAddress[];
  let logger: DebugLogger;

  let publicClient: PublicClient<HttpTransport, Chain>;
  let walletClient: any;

  let ethAccount: EthAddress;
  let ownerAddress: AztecAddress;
  let receiver: AztecAddress;
  let ownerPub: { x: bigint; y: bigint };
  const initialBalance = 10n;
  const wethAmountToBridge = parseEther('1');

  let uniswapPortal: any;
  let uniswapPortalAddress: EthAddress;
  let uniswapL2Contract: Contract;

  let wethContract: any;
  let wethTokenPortalAddress: EthAddress;
  let wethTokenPortal: any;
  let wethL2Contract: Contract;

  let daiContract: any;
  let daiTokenPortalAddress: EthAddress;
  let daiL2Contract: Contract;

  beforeEach(async () => {
    let deployL1ContractsValues: DeployL1Contracts;
    ({ aztecNode, aztecRpcServer, deployL1ContractsValues, accounts, logger } = await setup(2));

    walletClient = deployL1ContractsValues.walletClient;
    publicClient = deployL1ContractsValues.publicClient;

    if (Number(await publicClient.getBlockNumber()) < EXPECTED_FORKED_BLOCK) {
      throw new Error('This test must be run on a fork of mainnet with the expected fork block');
    }

    ethAccount = EthAddress.fromString((await walletClient.getAddresses())[0]);
    [ownerAddress, receiver] = accounts;
    ownerPub = pointToPublicKey(await aztecRpcServer.getAccountPublicKey(ownerAddress));

    logger('Deploying DAI Portal, initializing and deploying l2 contract...');
    const daiContracts = await deployAndInitializeNonNativeL2TokenContracts(
      aztecRpcServer,
      walletClient,
      publicClient,
      deployL1ContractsValues!.registryAddress,
      initialBalance,
      ownerPub,
      DAI_ADDRESS,
    );
    daiL2Contract = daiContracts.l2Contract;
    daiContract = daiContracts.underlyingERC20;
    daiTokenPortalAddress = daiContracts.tokenPortalAddress;

    logger('Deploying WETH Portal, initializing and deploying l2 contract...');
    const wethContracts = await deployAndInitializeNonNativeL2TokenContracts(
      aztecRpcServer,
      walletClient,
      publicClient,
      deployL1ContractsValues!.registryAddress,
      initialBalance,
      ownerPub,
      WETH9_ADDRESS,
    );
    wethL2Contract = wethContracts.l2Contract;
    wethContract = wethContracts.underlyingERC20;
    wethTokenPortal = wethContracts.tokenPortal;
    wethTokenPortalAddress = wethContracts.tokenPortalAddress;

    logger('Deploy Uniswap portal on L1 and L2...');
    uniswapPortalAddress = await deployL1Contract(walletClient, publicClient, UniswapPortalAbi, UniswapPortalBytecode);
    uniswapPortal = getContract({
      address: uniswapPortalAddress.toString(),
      abi: UniswapPortalAbi,
      walletClient,
      publicClient,
    });
    // deploy l2 uniswap contract and attach to portal
    const deployer = new ContractDeployer(UniswapContractAbi, aztecRpcServer);
    const tx = deployer.deploy().send({ portalContract: uniswapPortalAddress });
    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();
    uniswapL2Contract = new Contract(receipt.contractAddress!, UniswapContractAbi, aztecRpcServer);
    await uniswapL2Contract.attach(uniswapPortalAddress);

    await uniswapPortal.write.initialize(
      [deployL1ContractsValues!.registryAddress.toString(), uniswapL2Contract.address.toString()],
      {} as any,
    );

    // Give me some WETH so I can deposit to L2 and do the swap...
    logger('Getting some weth');
    await walletClient.sendTransaction({ to: WETH9_ADDRESS.toString(), value: parseEther('1') });
  }, 100_000);

  afterEach(async () => {
    await aztecNode?.stop();
    await aztecRpcServer?.stop();
  });

  const getL2BalanceOf = async (owner: AztecAddress, l2Contract: any) => {
    const ownerPublicKey = await aztecRpcServer.getAccountPublicKey(owner);
    const [balance] = await l2Contract.methods.getBalance(pointToPublicKey(ownerPublicKey)).view({ from: owner });
    return balance;
  };

  const expectBalanceOnL2 = async (owner: AztecAddress, expectedBalance: bigint, l2Contract: any) => {
    const balance = await getL2BalanceOf(owner, l2Contract);
    logger(`Account ${owner} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  const transferWethOnL2 = async (transferAmount: bigint) => {
    const transferTx = wethL2Contract.methods
      .transfer(
        transferAmount,
        pointToPublicKey(await aztecRpcServer.getAccountPublicKey(ownerAddress)),
        pointToPublicKey(await aztecRpcServer.getAccountPublicKey(receiver)),
      )
      .send({ from: accounts[0] });
    await transferTx.isMined(0, 0.1);
    const transferReceipt = await transferTx.getReceipt();
    expect(transferReceipt.status).toBe(TxStatus.MINED);
  };

  it('should uniswap trade on L1 from L2 funds privately (swaps WETH -> DAI)', async () => {
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
    const args = [ownerAddress.toString(), wethAmountToBridge, deadline, secretString, ethAccount.toString()] as const;
    const { result: messageKeyHex } = await wethTokenPortal.simulate.depositToAztec(args, {
      account: ethAccount.toString(),
    } as any);
    await wethTokenPortal.write.depositToAztec(args, {} as any);
    expect(await wethContract.read.balanceOf([ethAccount.toString()])).toBe(meBeforeBalance - wethAmountToBridge);
    const messageKey = Fr.fromString(messageKeyHex);

    // Wait for the archiver to process the message
    await delay(5000);
    // send a transfer tx to force through rollup with the message included
    const transferAmount = 1n;
    await transferWethOnL2(transferAmount);

    // 3. Claim WETH on L2
    logger('Minting weth on L2');
    // Call the mint tokens function on the noir contract
    const consumptionTx = wethL2Contract.methods
      .mint(wethAmountToBridge, ownerPub, ownerAddress, messageKey, secret, ethAccount.toField())
      .send({ from: ownerAddress });
    await consumptionTx.isMined(0, 0.1);
    const consumptionReceipt = await consumptionTx.getReceipt();
    expect(consumptionReceipt.status).toBe(TxStatus.MINED);
    await expectBalanceOnL2(ownerAddress, wethAmountToBridge + initialBalance - transferAmount, wethL2Contract);

    // Store balances
    const wethBalanceBeforeSwap = await getL2BalanceOf(ownerAddress, wethL2Contract);
    const daiBalanceBeforeSwap = await getL2BalanceOf(ownerAddress, daiL2Contract);

    // 4. Send L2 to L1 message to withdraw funds and another message to swap assets.
    logger('Send L2 tx to withdraw WETH to uniswap portal and send message to swap assets on L1');
    // recipient is the uniswap portal
    const selector = Fr.fromBuffer(wethL2Contract.methods.withdraw.selector);
    const minimumOutputAmount = 0;

    const withdrawTx = uniswapL2Contract.methods
      .swap(
        selector,
        wethL2Contract.address.toField(),
        wethTokenPortalAddress.toField(),
        wethAmountToBridge,
        new Fr(3000),
        daiL2Contract.address.toField(),
        daiTokenPortalAddress.toField(),
        new Fr(minimumOutputAmount),
        ownerPub,
        ownerAddress,
        secretHash,
        new Fr(2 ** 32 - 1),
        ethAccount.toField(),
        uniswapPortalAddress,
        ethAccount.toField(),
      )
      .send({ from: ownerAddress });
    await withdrawTx.isMined(0, 0.1);
    const withdrawReceipt = await withdrawTx.getReceipt();
    expect(withdrawReceipt.status).toBe(TxStatus.MINED);

    // check weth balance of owner on L2 (we first briedged `wethAmountToBridge` into L2 and now withdrew it!)
    await expectBalanceOnL2(ownerAddress, initialBalance - transferAmount, wethL2Contract);

    // 5. Consume L2 to L1 message by calling uniswapPortal.swap()
    logger('Execute withdraw and swap on the uniswapPortal!');
    const daiBalanceOfPortalBefore = await daiContract.read.balanceOf([daiTokenPortalAddress.toString()]);
    const swapArgs = [
      wethTokenPortalAddress.toString(),
      wethAmountToBridge,
      3000,
      daiTokenPortalAddress.toString(),
      minimumOutputAmount,
      ownerAddress.toString(),
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
    expect(daiBalanceOfPortalAfter).toBeGreaterThan(daiBalanceOfPortalBefore);
    const daiAmountToBridge = daiBalanceOfPortalAfter - daiBalanceOfPortalBefore;

    // Wait for the archiver to process the message
    await delay(5000);
    // send a transfer tx to force through rollup with the message included
    await transferWethOnL2(transferAmount);

    // 6. claim dai on L2
    logger('Consuming messages to mint dai on L2');
    // Call the mint tokens function on the noir contract
    const daiMintTx = daiL2Contract.methods
      .mint(daiAmountToBridge, ownerPub, ownerAddress, depositDaiMessageKey, secret, ethAccount.toField())
      .send({ from: ownerAddress });
    await daiMintTx.isMined(0, 0.1);
    const daiMintTxReceipt = await daiMintTx.getReceipt();
    expect(daiMintTxReceipt.status).toBe(TxStatus.MINED);
    await expectBalanceOnL2(ownerAddress, initialBalance + BigInt(daiAmountToBridge), daiL2Contract);

    const wethBalanceAfterSwap = await getL2BalanceOf(ownerAddress, wethL2Contract);
    const daiBalanceAfterSwap = await getL2BalanceOf(ownerAddress, daiL2Contract);

    logger('WETH balance before swap: ', wethBalanceBeforeSwap.toString());
    logger('DAI balance before swap  : ', daiBalanceBeforeSwap.toString());
    logger('***** 🧚‍♀️ SWAP L2 assets on L1 Uniswap 🧚‍♀️ *****');
    logger('WETH balance after swap : ', wethBalanceAfterSwap.toString());
    logger('DAI balance after swap  : ', daiBalanceAfterSwap.toString());
  }, 240_000);
});
