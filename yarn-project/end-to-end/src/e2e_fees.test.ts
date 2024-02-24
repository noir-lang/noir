import {
  AztecAddress,
  ExtendedNote,
  Fr,
  FunctionSelector,
  Note,
  PrivateFeePaymentMethod,
  TxHash,
  computeAuthWitMessageHash,
  computeMessageSecretHash,
} from '@aztec/aztec.js';
import { decodeFunctionSignature } from '@aztec/foundation/abi';
import { TokenContract as BananaCoin, FPCContract, GasTokenContract } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import {
  EndToEndContext,
  PublicBalancesFn,
  assertPublicBalances,
  getPublicBalancesFn,
  setup,
} from './fixtures/utils.js';
import { GasBridgingTestHarness } from './shared/gas_portal_test_harness.js';

const TOKEN_NAME = 'BananaCoin';
const TOKEN_SYMBOL = 'BAC';
const TOKEN_DECIMALS = 18n;

jest.setTimeout(100_000);

describe('e2e_fees', () => {
  let aliceAddress: AztecAddress;
  let sequencerAddress: AztecAddress;
  let gasTokenContract: GasTokenContract;
  let bananaCoin: BananaCoin;
  let bananaFPC: FPCContract;

  let gasBridgeTestHarness: GasBridgingTestHarness;
  let e2eContext: EndToEndContext;

  let gasBalances: PublicBalancesFn;
  let bananaBalances: PublicBalancesFn;

  beforeAll(async () => {
    process.env.PXE_URL = '';
    e2eContext = await setup(3);

    const { wallets, accounts, logger, aztecNode, pxe, deployL1ContractsValues } = e2eContext;

    gasBridgeTestHarness = await GasBridgingTestHarness.new(
      pxe,
      deployL1ContractsValues.publicClient,
      deployL1ContractsValues.walletClient,
      wallets[0],
      logger,
    );

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

    gasBalances = getPublicBalancesFn('⛽', gasTokenContract, e2eContext.logger);
    bananaBalances = getPublicBalancesFn('🍌', bananaCoin, e2eContext.logger);
    await assertPublicBalances(
      gasBalances,
      [sequencerAddress, aliceAddress, bananaFPC.address],
      [0n, 0n, InitialFPCGas],
    );
    await assertPublicBalances(bananaBalances, [sequencerAddress, aliceAddress, bananaFPC.address], [0n, 0n, 0n]);
  }, 100_000);

  it('mint banana privately, pay privately with banana via FPC', async () => {
    const PrivateInitialBananasAmount = 100n;
    const MintedBananasAmount = 1000n;
    const FeeAmount = 1n;
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

    // set up auth wit for FPC for to unshield Alice's bananas to itself
    const nonce = 1;
    const messageHash = computeAuthWitMessageHash(
      bananaFPC.address,
      bananaCoin.methods.unshield(accounts[0].address, bananaFPC.address, FeeAmount, nonce).request(),
    );
    await wallets[0].createAuthWitness(messageHash);

    await bananaCoin.methods
      .mint_public(aliceAddress, MintedBananasAmount)
      .send({
        fee: {
          maxFee: FeeAmount,
          paymentMethod: new PrivateFeePaymentMethod(bananaCoin.address, bananaFPC.address, wallets[0]),
        },
      })
      .wait();

    await assertPublicBalances(
      gasBalances,
      [sequencerAddress, aliceAddress, bananaFPC.address],
      [FeeAmount, 0n, InitialFPCGas - FeeAmount],
    );

    await assertPublicBalances(
      bananaBalances,
      [sequencerAddress, aliceAddress, bananaFPC.address],
      [0n, MintedBananasAmount, FeeAmount],
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
