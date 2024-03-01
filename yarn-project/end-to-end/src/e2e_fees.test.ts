import {
  AztecAddress,
  ExtendedNote,
  Fr,
  FunctionSelector,
  Note,
  PrivateFeePaymentMethod,
  TxHash,
  computeMessageSecretHash,
} from '@aztec/aztec.js';
import { decodeFunctionSignature } from '@aztec/foundation/abi';
import { TokenContract as BananaCoin, FPCContract, GasTokenContract } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import { BalancesFn, EndToEndContext, expectMapping, getBalancesFn, setup } from './fixtures/utils.js';
import { GasPortalTestingHarnessFactory, IGasBridgingTestHarness } from './shared/gas_portal_test_harness.js';

const TOKEN_NAME = 'BananaCoin';
const TOKEN_SYMBOL = 'BC';
const TOKEN_DECIMALS = 18n;

jest.setTimeout(100_000);

describe('e2e_fees', () => {
  let aliceAddress: AztecAddress;
  let sequencerAddress: AztecAddress;
  let gasTokenContract: GasTokenContract;
  let bananaCoin: BananaCoin;
  let bananaFPC: FPCContract;

  let gasBridgeTestHarness: IGasBridgingTestHarness;
  let e2eContext: EndToEndContext;

  let gasBalances: BalancesFn;
  let bananaPublicBalances: BalancesFn;
  let bananaPrivateBalances: BalancesFn;

  beforeAll(async () => {
    process.env.PXE_URL = '';
    e2eContext = await setup(3);

    const { wallets, accounts, logger, aztecNode, pxe, deployL1ContractsValues } = e2eContext;

    gasBridgeTestHarness = await GasPortalTestingHarnessFactory.create({
      pxeService: pxe,
      publicClient: deployL1ContractsValues.publicClient,
      walletClient: deployL1ContractsValues.walletClient,
      wallet: wallets[0],
      logger,
      mockL1: false,
    });

    gasTokenContract = gasBridgeTestHarness.l2Token;

    BananaCoin.artifact.functions.forEach(fn => {
      const sig = decodeFunctionSignature(fn.name, fn.parameters);
      logger(`Function ${sig} and the selector: ${FunctionSelector.fromNameAndParameters(fn.name, fn.parameters)}`);
    });
    FPCContract.artifact.functions.forEach(fn => {
      const sig = decodeFunctionSignature(fn.name, fn.parameters);
      logger(`Function ${sig} and the selector: ${FunctionSelector.fromNameAndParameters(fn.name, fn.parameters)}`);
    });
    GasTokenContract.artifact.functions.forEach(fn => {
      const sig = decodeFunctionSignature(fn.name, fn.parameters);
      logger(`Function ${sig} and the selector: ${FunctionSelector.fromNameAndParameters(fn.name, fn.parameters)}`);
    });

    await aztecNode.setConfig({
      feeRecipient: accounts.at(-1)!.address,
    });

    aliceAddress = accounts.at(0)!.address;
    sequencerAddress = accounts.at(-1)!.address;
  }, 30_000);

  const InitialFPCGas = 500n;
  beforeEach(async () => {
    bananaCoin = await BananaCoin.deploy(
      e2eContext.wallets[0],
      e2eContext.accounts[0],
      TOKEN_NAME,
      TOKEN_SYMBOL,
      TOKEN_DECIMALS,
    )
      .send()
      .deployed();

    e2eContext.logger(`BananaCoin deployed at ${bananaCoin.address}`);

    bananaFPC = await FPCContract.deploy(e2eContext.wallets[0], bananaCoin.address, gasTokenContract.address)
      .send()
      .deployed();
    e2eContext.logger(`bananaPay deployed at ${bananaFPC.address}`);
    await gasBridgeTestHarness.bridgeFromL1ToL2(InitialFPCGas + 1n, InitialFPCGas, bananaFPC.address);

    gasBalances = getBalancesFn('⛽', gasTokenContract.methods.balance_of_public, e2eContext.logger);
    bananaPublicBalances = getBalancesFn('🍌.public', bananaCoin.methods.balance_of_public, e2eContext.logger);
    bananaPrivateBalances = getBalancesFn('🍌.private', bananaCoin.methods.balance_of_private, e2eContext.logger);
    await expectMapping(bananaPrivateBalances, [aliceAddress, bananaFPC.address, sequencerAddress], [0n, 0n, 0n]);
    await expectMapping(bananaPublicBalances, [aliceAddress, bananaFPC.address, sequencerAddress], [0n, 0n, 0n]);
    await expectMapping(gasBalances, [aliceAddress, bananaFPC.address, sequencerAddress], [0n, InitialFPCGas, 0n]);
  }, 100_000);

  it('mint banana privately, pay privately with banana via FPC', async () => {
    const PrivateInitialBananasAmount = 100n;
    const MintedBananasAmount = 1000n;
    const FeeAmount = 1n;
    const RefundAmount = 2n;
    const MaxFee = FeeAmount + RefundAmount;
    const { wallets, accounts } = e2eContext;

    // Mint bananas privately
    const secret = Fr.random();
    const secretHash = computeMessageSecretHash(secret);
    const receipt = await bananaCoin.methods.mint_private(PrivateInitialBananasAmount, secretHash).send().wait();

    // Setup auth wit
    await addPendingShieldNoteToPXE(0, PrivateInitialBananasAmount, secretHash, receipt.txHash);
    const txClaim = bananaCoin.methods.redeem_shield(accounts[0].address, PrivateInitialBananasAmount, secret).send();
    const receiptClaim = await txClaim.wait({ debug: true });
    const { visibleNotes } = receiptClaim.debugInfo!;
    expect(visibleNotes[0].note.items[0].toBigInt()).toBe(PrivateInitialBananasAmount);

    await expectMapping(
      bananaPrivateBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [PrivateInitialBananasAmount, 0n, 0n],
    );
    await expectMapping(bananaPublicBalances, [aliceAddress, bananaFPC.address, sequencerAddress], [0n, 0n, 0n]);
    await expectMapping(gasBalances, [aliceAddress, bananaFPC.address, sequencerAddress], [0n, InitialFPCGas, 0n]);

    /**
     * PRIVATE SETUP
     * check authwit
     * reduce alice BC.private by MaxFee
     * enqueue public call to increase FPC BC.public by MaxFee
     * enqueue public call for fpc.pay_fee
     *
     * PUBLIC SETUP
     * increase FPC BC.public by MaxFee
     *
     * PUBLIC APP LOGIC
     * increase alice BC.public by MintedBananasAmount
     * increase BC total supply by MintedBananasAmount
     *
     * PUBLIC TEARDOWN
     * call gas.pay_fee
     *   decrease FPC AZT by FeeAmount
     *   increase sequencer AZT by FeeAmount
     * call banana.transfer_public
     *   decrease FPC BC.public by RefundAmount
     *   increase alice BC.public by RefundAmount
     *
     */
    await bananaCoin.methods
      .mint_public(aliceAddress, MintedBananasAmount)
      .send({
        fee: {
          maxFee: MaxFee,
          paymentMethod: new PrivateFeePaymentMethod(bananaCoin.address, bananaFPC.address, wallets[0]),
        },
      })
      .wait();

    await expectMapping(
      bananaPrivateBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [PrivateInitialBananasAmount - MaxFee, 0n, 0n],
    );
    await expectMapping(
      bananaPublicBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [MintedBananasAmount + RefundAmount, MaxFee - RefundAmount, 0n],
    );
    await expectMapping(
      gasBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [0n, InitialFPCGas - FeeAmount, FeeAmount],
    );
  }, 100_000);

  const addPendingShieldNoteToPXE = async (accountIndex: number, amount: bigint, secretHash: Fr, txHash: TxHash) => {
    const storageSlot = new Fr(5); // The storage slot of `pending_shields` is 5.
    const noteTypeId = new Fr(84114971101151129711410111011678111116101n); // TransparentNote

    const note = new Note([new Fr(amount), secretHash]);
    const extendedNote = new ExtendedNote(
      note,
      e2eContext.accounts[accountIndex].address,
      bananaCoin.address,
      storageSlot,
      noteTypeId,
      txHash,
    );
    await e2eContext.wallets[accountIndex].addNote(extendedNote);
  };
});
