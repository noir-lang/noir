import {
  AztecAddress,
  BatchCall,
  DebugLogger,
  ExtendedNote,
  Fr,
  FunctionSelector,
  Note,
  PrivateFeePaymentMethod,
  PublicFeePaymentMethod,
  TxHash,
  Wallet,
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
  let aliceWallet: Wallet;
  let aliceAddress: AztecAddress;
  let bobAddress: AztecAddress;
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

    aliceWallet = wallets[0];
    aliceAddress = accounts[0].address;
    bobAddress = accounts[1].address;
    sequencerAddress = accounts[2].address;

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

  describe('private fees payments', () => {
    let InitialAlicePrivateBananas: bigint;
    let InitialAlicePublicBananas: bigint;
    let InitialAliceGas: bigint;

    let InitialBobPrivateBananas: bigint;

    let InitialFPCPrivateBananas: bigint;
    let InitialFPCPublicBananas: bigint;
    let InitialFPCGas: bigint;

    let InitialSequencerGas: bigint;

    let MaxFee: bigint;
    let FeeAmount: bigint;
    let RefundAmount: bigint;
    let RefundSecret: Fr;

    beforeAll(async () => {
      // fund Alice
      await mintPrivate(100n, aliceAddress);
    });

    beforeEach(async () => {
      FeeAmount = 1n;
      RefundAmount = 2n;
      MaxFee = FeeAmount + RefundAmount;
      RefundSecret = Fr.random();

      [
        [InitialAlicePrivateBananas, InitialBobPrivateBananas, InitialFPCPrivateBananas],
        [InitialAlicePublicBananas, InitialFPCPublicBananas],
        [InitialAliceGas, InitialFPCGas, InitialSequencerGas],
      ] = await Promise.all([
        bananaPrivateBalances(aliceAddress, bobAddress, bananaFPC.address),
        bananaPublicBalances(aliceAddress, bananaFPC.address),
        gasBalances(aliceAddress, bananaFPC.address, sequencerAddress),
      ]);
    });

    it("pays fees for tx that don't run public app logic", async () => {
      /**
       * PRIVATE SETUP
       * check authwit
       * reduce alice BC.private by MaxFee
       * enqueue public call to increase FPC BC.public by MaxFee
       * enqueue public call for fpc.pay_fee_with_shielded_rebate
       *
       * PRIVATE APP LOGIC
       * reduce Alice's BC.private by transferAmount
       * create note for Bob of transferAmount
       *
       * PUBLIC SETUP
       * increase FPC BC.public by MaxFee
       *
       * PUBLIC APP LOGIC
       * N/A
       *
       * PUBLIC TEARDOWN
       * call gas.pay_fee
       *   decrease FPC AZT by FeeAmount
       *   increase sequencer AZT by FeeAmount
       * call banana.shield
       *   decrease FPC BC.public by RefundAmount
       *   create transparent note with RefundAmount
       *
       * this is expected to squash notes and nullifiers
       */
      const transferAmount = 5n;
      const tx = await bananaCoin.methods
        .transfer(aliceAddress, bobAddress, transferAmount, 0n)
        .send({
          fee: {
            maxFee: MaxFee,
            paymentMethod: new PrivateFeePaymentMethod(
              bananaCoin.address,
              bananaFPC.address,
              aliceWallet,
              RefundSecret,
            ),
          },
        })
        .wait();

      await expectMapping(
        bananaPrivateBalances,
        [aliceAddress, bobAddress, bananaFPC.address, sequencerAddress],
        [InitialAlicePrivateBananas - MaxFee - transferAmount, transferAmount, InitialFPCPrivateBananas, 0n],
      );
      await expectMapping(
        bananaPublicBalances,
        [aliceAddress, bananaFPC.address, sequencerAddress],
        [InitialAlicePublicBananas, InitialFPCPublicBananas + MaxFee - RefundAmount, 0n],
      );
      await expectMapping(
        gasBalances,
        [aliceAddress, bananaFPC.address, sequencerAddress],
        [InitialAliceGas, InitialFPCGas - FeeAmount, InitialSequencerGas + FeeAmount],
      );

      await expect(
        // this rejects if note can't be added
        addPendingShieldNoteToPXE(0, RefundAmount, computeMessageSecretHash(RefundSecret), tx.txHash),
      ).resolves.toBeUndefined();
    });

    it('pays fees for tx that creates notes in private', async () => {
      /**
       * PRIVATE SETUP
       * check authwit
       * reduce alice BC.private by MaxFee
       * enqueue public call to increase FPC BC.public by MaxFee
       * enqueue public call for fpc.pay_fee_with_shielded_rebate
       *
       * PRIVATE APP LOGIC
       * increase alice BC.private by newlyMintedBananas
       *
       * PUBLIC SETUP
       * increase FPC BC.public by MaxFee
       *
       * PUBLIC APP LOGIC
       * BC increase total supply
       *
       * PUBLIC TEARDOWN
       * call gas.pay_fee
       *   decrease FPC AZT by FeeAmount
       *   increase sequencer AZT by FeeAmount
       * call banana.shield
       *   decrease FPC BC.public by RefundAmount
       *   create transparent note with RefundAmount
       */
      const newlyMintedBananas = 10n;
      const tx = await bananaCoin.methods
        .privately_mint_private_note(newlyMintedBananas)
        .send({
          fee: {
            maxFee: MaxFee,
            paymentMethod: new PrivateFeePaymentMethod(
              bananaCoin.address,
              bananaFPC.address,
              aliceWallet,
              RefundSecret,
            ),
          },
        })
        .wait();

      await expectMapping(
        bananaPrivateBalances,
        [aliceAddress, bananaFPC.address, sequencerAddress],
        [InitialAlicePrivateBananas - MaxFee + newlyMintedBananas, InitialFPCPrivateBananas, 0n],
      );
      await expectMapping(
        bananaPublicBalances,
        [aliceAddress, bananaFPC.address, sequencerAddress],
        [InitialAlicePublicBananas, InitialFPCPublicBananas + MaxFee - RefundAmount, 0n],
      );
      await expectMapping(
        gasBalances,
        [aliceAddress, bananaFPC.address, sequencerAddress],
        [InitialAliceGas, InitialFPCGas - FeeAmount, InitialSequencerGas + FeeAmount],
      );

      await expect(
        // this rejects if note can't be added
        addPendingShieldNoteToPXE(0, RefundAmount, computeMessageSecretHash(RefundSecret), tx.txHash),
      ).resolves.toBeUndefined();
    });

    it('pays fees for tx that creates notes in public', async () => {
      /**
       * PRIVATE SETUP
       * check authwit
       * reduce alice BC.private by MaxFee
       * enqueue public call to increase FPC BC.public by MaxFee
       * enqueue public call for fpc.pay_fee_with_shielded_rebate
       *
       * PRIVATE APP LOGIC
       * N/A
       *
       * PUBLIC SETUP
       * increase FPC BC.public by MaxFee
       *
       * PUBLIC APP LOGIC
       * BC decrease Alice public balance by shieldedBananas
       * BC create transparent note of shieldedBananas
       *
       * PUBLIC TEARDOWN
       * call gas.pay_fee
       *   decrease FPC AZT by FeeAmount
       *   increase sequencer AZT by FeeAmount
       * call banana.shield
       *   decrease FPC BC.public by RefundAmount
       *   create transparent note with RefundAmount
       */
      const shieldedBananas = 1n;
      const shieldSecret = Fr.random();
      const shieldSecretHash = computeMessageSecretHash(shieldSecret);
      const tx = await bananaCoin.methods
        .shield(aliceAddress, shieldedBananas, shieldSecretHash, 0n)
        .send({
          fee: {
            maxFee: MaxFee,
            paymentMethod: new PrivateFeePaymentMethod(
              bananaCoin.address,
              bananaFPC.address,
              aliceWallet,
              RefundSecret,
            ),
          },
        })
        .wait();

      await expectMapping(
        bananaPrivateBalances,
        [aliceAddress, bananaFPC.address, sequencerAddress],
        [InitialAlicePrivateBananas - MaxFee, InitialFPCPrivateBananas, 0n],
      );
      await expectMapping(
        bananaPublicBalances,
        [aliceAddress, bananaFPC.address, sequencerAddress],
        [InitialAlicePublicBananas - shieldedBananas, InitialFPCPublicBananas + MaxFee - RefundAmount, 0n],
      );
      await expectMapping(
        gasBalances,
        [aliceAddress, bananaFPC.address, sequencerAddress],
        [InitialAliceGas, InitialFPCGas - FeeAmount, InitialSequencerGas + FeeAmount],
      );

      await expect(addPendingShieldNoteToPXE(0, shieldedBananas, shieldSecretHash, tx.txHash)).resolves.toBeUndefined();

      await expect(
        addPendingShieldNoteToPXE(0, RefundAmount, computeMessageSecretHash(RefundSecret), tx.txHash),
      ).resolves.toBeUndefined();
    });

    it('pays fees for tx that creates notes in both private and public', async () => {
      const privateTransfer = 1n;
      const shieldedBananas = 1n;
      const shieldSecret = Fr.random();
      const shieldSecretHash = computeMessageSecretHash(shieldSecret);

      /**
       * PRIVATE SETUP
       * check authwit
       * reduce alice BC.private by MaxFee
       * enqueue public call to increase FPC BC.public by MaxFee
       * enqueue public call for fpc.pay_fee_with_shielded_rebate
       *
       * PRIVATE APP LOGIC
       * reduce Alice's private balance by privateTransfer
       * create note for Bob with privateTransfer amount of private BC
       *
       * PUBLIC SETUP
       * increase FPC BC.public by MaxFee
       *
       * PUBLIC APP LOGIC
       * BC decrease Alice public balance by shieldedBananas
       * BC create transparent note of shieldedBananas
       *
       * PUBLIC TEARDOWN
       * call gas.pay_fee
       *   decrease FPC AZT by FeeAmount
       *   increase sequencer AZT by FeeAmount
       * call banana.shield
       *   decrease FPC BC.public by RefundAmount
       *   create transparent note with RefundAmount
       */
      const tx = await new BatchCall(aliceWallet, [
        bananaCoin.methods.transfer(aliceAddress, bobAddress, privateTransfer, 0n).request(),
        bananaCoin.methods.shield(aliceAddress, shieldedBananas, shieldSecretHash, 0n).request(),
      ])
        .send({
          fee: {
            maxFee: MaxFee,
            paymentMethod: new PrivateFeePaymentMethod(
              bananaCoin.address,
              bananaFPC.address,
              aliceWallet,
              RefundSecret,
            ),
          },
        })
        .wait();

      await expectMapping(
        bananaPrivateBalances,
        [aliceAddress, bobAddress, bananaFPC.address, sequencerAddress],
        [
          InitialAlicePrivateBananas - MaxFee - privateTransfer,
          InitialBobPrivateBananas + privateTransfer,
          InitialFPCPrivateBananas,
          0n,
        ],
      );
      await expectMapping(
        bananaPublicBalances,
        [aliceAddress, bananaFPC.address, sequencerAddress],
        [InitialAlicePublicBananas - shieldedBananas, InitialFPCPublicBananas + MaxFee - RefundAmount, 0n],
      );
      await expectMapping(
        gasBalances,
        [aliceAddress, bananaFPC.address, sequencerAddress],
        [InitialAliceGas, InitialFPCGas - FeeAmount, InitialSequencerGas + FeeAmount],
      );

      await expect(addPendingShieldNoteToPXE(0, shieldedBananas, shieldSecretHash, tx.txHash)).resolves.toBeUndefined();

      await expect(
        addPendingShieldNoteToPXE(0, RefundAmount, computeMessageSecretHash(RefundSecret), tx.txHash),
      ).resolves.toBeUndefined();
    });
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
