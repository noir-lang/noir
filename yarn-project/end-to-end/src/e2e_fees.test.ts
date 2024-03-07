import {
  AztecAddress,
  DebugLogger,
  ExtendedNote,
  Fr,
  FunctionSelector,
  Note,
  PrivateFeePaymentMethod,
  PublicFeePaymentMethod,
  TxHash,
  computeMessageSecretHash,
} from '@aztec/aztec.js';
import { ContractArtifact, decodeFunctionSignature } from '@aztec/foundation/abi';
import {
  TokenContract as BananaCoin,
  FPCContract,
  GasTokenContract,
  SchnorrAccountContract,
} from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import {
  BalancesFn,
  EndToEndContext,
  expectMapping,
  getBalancesFn,
  publicDeployAccounts,
  setup,
} from './fixtures/utils.js';
import { GasPortalTestingHarnessFactory, IGasBridgingTestHarness } from './shared/gas_portal_test_harness.js';

const TOKEN_NAME = 'BananaCoin';
const TOKEN_SYMBOL = 'BC';
const TOKEN_DECIMALS = 18n;
const BRIDGED_FPC_GAS = 500n;

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

    const { accounts, logger, aztecNode, pxe, deployL1ContractsValues, wallets } = e2eContext;

    logFunctionSignatures(BananaCoin.artifact, logger);
    logFunctionSignatures(FPCContract.artifact, logger);
    logFunctionSignatures(GasTokenContract.artifact, logger);
    logFunctionSignatures(SchnorrAccountContract.artifact, logger);

    await aztecNode.setConfig({
      feeRecipient: accounts.at(-1)!.address,
    });

    aliceAddress = accounts.at(0)!.address;
    sequencerAddress = accounts.at(-1)!.address;

    gasBridgeTestHarness = await GasPortalTestingHarnessFactory.create({
      pxeService: pxe,
      publicClient: deployL1ContractsValues.publicClient,
      walletClient: deployL1ContractsValues.walletClient,
      wallet: wallets[0],
      logger,
      mockL1: false,
    });

    gasTokenContract = gasBridgeTestHarness.l2Token;

    bananaCoin = await BananaCoin.deploy(wallets[0], accounts[0], TOKEN_NAME, TOKEN_SYMBOL, TOKEN_DECIMALS)
      .send()
      .deployed();

    logger(`BananaCoin deployed at ${bananaCoin.address}`);

    bananaFPC = await FPCContract.deploy(wallets[0], bananaCoin.address, gasTokenContract.address).send().deployed();
    logger(`bananaPay deployed at ${bananaFPC.address}`);
    await publicDeployAccounts(wallets[0], accounts);

    await gasBridgeTestHarness.bridgeFromL1ToL2(BRIDGED_FPC_GAS, BRIDGED_FPC_GAS, bananaFPC.address);

    bananaPublicBalances = getBalancesFn('🍌.public', bananaCoin.methods.balance_of_public, logger);
    bananaPrivateBalances = getBalancesFn('🍌.private', bananaCoin.methods.balance_of_private, logger);
    gasBalances = getBalancesFn('⛽', gasTokenContract.methods.balance_of_public, logger);
    await expectMapping(bananaPrivateBalances, [aliceAddress, bananaFPC.address, sequencerAddress], [0n, 0n, 0n]);
    await expectMapping(bananaPublicBalances, [aliceAddress, bananaFPC.address, sequencerAddress], [0n, 0n, 0n]);
    await expectMapping(gasBalances, [aliceAddress, bananaFPC.address, sequencerAddress], [0n, BRIDGED_FPC_GAS, 0n]);
  });

  it('reverts transactions but still pays fees using PublicFeePaymentMethod', async () => {
    const OutrageousPublicAmountAliceDoesNotHave = 10000n;
    const PublicMintedAlicePublicBananas = 1000n;
    const FeeAmount = 1n;
    const RefundAmount = 2n;
    const MaxFee = FeeAmount + RefundAmount;
    const { wallets } = e2eContext;

    const [initialAlicePrivateBananas, initialFPCPrivateBananas] = await bananaPrivateBalances(
      aliceAddress,
      bananaFPC.address,
    );
    const [initialAlicePublicBananas, initialFPCPublicBananas] = await bananaPublicBalances(
      aliceAddress,
      bananaFPC.address,
    );
    const [initialAliceGas, initialFPCGas, initialSequencerGas] = await gasBalances(
      aliceAddress,
      bananaFPC.address,
      sequencerAddress,
    );

    await bananaCoin.methods.mint_public(aliceAddress, PublicMintedAlicePublicBananas).send().wait();
    // if we simulate locally, it throws an error
    await expect(
      bananaCoin.methods
        .transfer_public(aliceAddress, sequencerAddress, OutrageousPublicAmountAliceDoesNotHave, 0)
        .send({
          fee: {
            maxFee: MaxFee,
            paymentMethod: new PublicFeePaymentMethod(bananaCoin.address, bananaFPC.address, wallets[0]),
          },
        })
        .wait(),
    ).rejects.toThrow(/attempt to subtract with underflow 'hi == high'/);

    // we did not pay the fee, because we did not submit the TX
    await expectMapping(
      bananaPrivateBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [initialAlicePrivateBananas, initialFPCPrivateBananas, 0n],
    );
    await expectMapping(
      bananaPublicBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [initialAlicePublicBananas + PublicMintedAlicePublicBananas, initialFPCPublicBananas, 0n],
    );
    await expectMapping(
      gasBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [initialAliceGas, initialFPCGas, initialSequencerGas],
    );

    // if we skip simulation, it includes the failed TX
    await bananaCoin.methods
      .transfer_public(aliceAddress, sequencerAddress, OutrageousPublicAmountAliceDoesNotHave, 0)
      .send({
        skipPublicSimulation: true,
        fee: {
          maxFee: MaxFee,
          paymentMethod: new PublicFeePaymentMethod(bananaCoin.address, bananaFPC.address, wallets[0]),
        },
      })
      .wait();

    // and thus we paid the fee
    await expectMapping(
      bananaPrivateBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [initialAlicePrivateBananas, initialFPCPrivateBananas, 0n],
    );
    await expectMapping(
      bananaPublicBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [initialAlicePublicBananas + PublicMintedAlicePublicBananas - FeeAmount, initialFPCPublicBananas + FeeAmount, 0n],
    );
    await expectMapping(
      gasBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [initialAliceGas, initialFPCGas - FeeAmount, initialSequencerGas + FeeAmount],
    );

    // TODO(#4712) - demonstrate reverts with the PrivateFeePaymentMethod.
    // Can't do presently because all logs are "revertible" so we lose notes that get broadcasted during unshielding.
  });

  it('mint banana privately, pay privately with banana via FPC', async () => {
    const PrivateMintedBananasAmount = 100n;
    const AppLogicMintedBananasAmount = 1000n;
    const FeeAmount = 1n;
    const RefundAmount = 2n;
    const MaxFee = FeeAmount + RefundAmount;
    const { wallets } = e2eContext;

    const [initialAlicePrivateBananas, initialFPCPrivateBananas] = await bananaPrivateBalances(
      aliceAddress,
      bananaFPC.address,
    );
    const [initialAlicePublicBananas, initialFPCPublicBananas] = await bananaPublicBalances(
      aliceAddress,
      bananaFPC.address,
    );
    const [initialAliceGas, initialFPCGas, initialSequencerGas] = await gasBalances(
      aliceAddress,
      bananaFPC.address,
      sequencerAddress,
    );

    await mintPrivate(PrivateMintedBananasAmount, aliceAddress);

    await expectMapping(
      bananaPrivateBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [initialAlicePrivateBananas + PrivateMintedBananasAmount, initialFPCPrivateBananas, 0n],
    );
    await expectMapping(
      bananaPublicBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [initialAlicePublicBananas, initialFPCPublicBananas, 0n],
    );

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
      .mint_public(aliceAddress, AppLogicMintedBananasAmount)
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
      [initialAlicePrivateBananas + PrivateMintedBananasAmount - MaxFee, initialFPCPrivateBananas, 0n],
    );
    await expectMapping(
      bananaPublicBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [
        initialAlicePublicBananas + AppLogicMintedBananasAmount + RefundAmount,
        initialFPCPublicBananas + MaxFee - RefundAmount,
        0n,
      ],
    );
    await expectMapping(
      gasBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [initialAliceGas, initialFPCGas - FeeAmount, initialSequencerGas + FeeAmount],
    );
  });

  function logFunctionSignatures(artifact: ContractArtifact, logger: DebugLogger) {
    artifact.functions.forEach(fn => {
      const sig = decodeFunctionSignature(fn.name, fn.parameters);
      logger(`${FunctionSelector.fromNameAndParameters(fn.name, fn.parameters)} => ${artifact.name}.${sig} `);
    });
  }

  const mintPrivate = async (amount: bigint, address: AztecAddress) => {
    // Mint bananas privately
    const secret = Fr.random();
    const secretHash = computeMessageSecretHash(secret);
    const receipt = await bananaCoin.methods.mint_private(amount, secretHash).send().wait();

    // Setup auth wit
    await addPendingShieldNoteToPXE(0, amount, secretHash, receipt.txHash);
    const txClaim = bananaCoin.methods.redeem_shield(address, amount, secret).send();
    const receiptClaim = await txClaim.wait({ debug: true });
    const { visibleNotes } = receiptClaim.debugInfo!;
    expect(visibleNotes[0].note.items[0].toBigInt()).toBe(amount);
  };

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
