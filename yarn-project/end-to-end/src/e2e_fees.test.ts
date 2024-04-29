import {
  type AccountWallet,
  type AztecAddress,
  BatchCall,
  type DebugLogger,
  ExtendedNote,
  Fr,
  type FunctionCall,
  FunctionSelector,
  Note,
  PrivateFeePaymentMethod,
  PublicFeePaymentMethod,
  type TxHash,
  TxStatus,
  type Wallet,
  computeAuthWitMessageHash,
  computeSecretHash,
} from '@aztec/aztec.js';
import { FunctionData, GasSettings } from '@aztec/circuits.js';
import { type ContractArtifact, decodeFunctionSignature } from '@aztec/foundation/abi';
import {
  TokenContract as BananaCoin,
  FPCContract,
  GasTokenContract,
  SchnorrAccountContract,
} from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import { type BalancesFn, expectMapping, getBalancesFn, publicDeployAccounts, setup } from './fixtures/utils.js';
import { GasPortalTestingHarnessFactory, type IGasBridgingTestHarness } from './shared/gas_portal_test_harness.js';

const TOKEN_NAME = 'BananaCoin';
const TOKEN_SYMBOL = 'BC';
const TOKEN_DECIMALS = 18n;
const BRIDGED_FPC_GAS = 500n;

jest.setTimeout(1_000_000_000);

describe('e2e_fees', () => {
  let wallets: AccountWallet[];
  let aliceWallet: Wallet;
  let aliceAddress: AztecAddress;
  let bobAddress: AztecAddress;
  let sequencerAddress: AztecAddress;
  let gasTokenContract: GasTokenContract;
  let bananaCoin: BananaCoin;
  let bananaFPC: FPCContract;
  let logger: DebugLogger;

  let gasBridgeTestHarness: IGasBridgingTestHarness;

  let gasBalances: BalancesFn;
  let bananaPublicBalances: BalancesFn;
  let bananaPrivateBalances: BalancesFn;

  const gasSettings = GasSettings.default();

  beforeAll(async () => {
    const ctx = await setup(3, {}, {}, true);
    const { aztecNode, deployL1ContractsValues, pxe } = ctx;
    ({ wallets, logger } = ctx);

    logFunctionSignatures(BananaCoin.artifact, logger);
    logFunctionSignatures(FPCContract.artifact, logger);
    logFunctionSignatures(GasTokenContract.artifact, logger);
    logFunctionSignatures(SchnorrAccountContract.artifact, logger);

    await aztecNode.setConfig({
      feeRecipient: wallets.at(-1)!.getAddress(),
    });

    aliceWallet = wallets[0];
    aliceAddress = wallets[0].getAddress();
    bobAddress = wallets[1].getAddress();
    sequencerAddress = wallets[2].getAddress();

    gasBridgeTestHarness = await GasPortalTestingHarnessFactory.create({
      aztecNode: aztecNode,
      pxeService: pxe,
      publicClient: deployL1ContractsValues.publicClient,
      walletClient: deployL1ContractsValues.walletClient,
      wallet: wallets[0],
      logger,
      mockL1: false,
    });

    gasTokenContract = gasBridgeTestHarness.l2Token;

    bananaCoin = await BananaCoin.deploy(wallets[0], wallets[0].getAddress(), TOKEN_NAME, TOKEN_SYMBOL, TOKEN_DECIMALS)
      .send()
      .deployed();

    logger.info(`BananaCoin deployed at ${bananaCoin.address}`);

    bananaFPC = await FPCContract.deploy(wallets[0], bananaCoin.address, gasTokenContract.address).send().deployed();
    logger.info(`BananaPay deployed at ${bananaFPC.address}`);
    await publicDeployAccounts(wallets[0], wallets);

    await gasBridgeTestHarness.bridgeFromL1ToL2(BRIDGED_FPC_GAS, BRIDGED_FPC_GAS, bananaFPC.address);

    bananaPublicBalances = getBalancesFn('🍌.public', bananaCoin.methods.balance_of_public, logger);
    bananaPrivateBalances = getBalancesFn('🍌.private', bananaCoin.methods.balance_of_private, logger);
    gasBalances = getBalancesFn('⛽', gasTokenContract.methods.balance_of_public, logger);
    await expectMapping(bananaPrivateBalances, [aliceAddress, bananaFPC.address, sequencerAddress], [0n, 0n, 0n]);
    await expectMapping(bananaPublicBalances, [aliceAddress, bananaFPC.address, sequencerAddress], [0n, 0n, 0n]);
    await expectMapping(gasBalances, [aliceAddress, bananaFPC.address, sequencerAddress], [0n, BRIDGED_FPC_GAS, 0n]);
  });

  it('reverts transactions but still pays fees using PublicFeePaymentMethod', async () => {
    const OutrageousPublicAmountAliceDoesNotHave = BigInt(1e15);
    const PublicMintedAlicePublicBananas = BigInt(1e12);
    const FeeAmount = 1n;

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
            gasSettings,
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
    const txReceipt = await bananaCoin.methods
      .transfer_public(aliceAddress, sequencerAddress, OutrageousPublicAmountAliceDoesNotHave, 0)
      .send({
        skipPublicSimulation: true,
        fee: {
          gasSettings,
          paymentMethod: new PublicFeePaymentMethod(bananaCoin.address, bananaFPC.address, wallets[0]),
        },
      })
      .wait({ dontThrowOnRevert: true });
    expect(txReceipt.status).toBe(TxStatus.REVERTED);

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
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    let InitialBobPublicBananas: bigint;

    let InitialFPCPrivateBananas: bigint;
    let InitialFPCPublicBananas: bigint;
    let InitialFPCGas: bigint;

    let InitialSequencerGas: bigint;

    let MaxFee: bigint;
    let FeeAmount: bigint;
    let RefundAmount: bigint;
    let RefundSecret: Fr;

    beforeAll(async () => {
      // Fund Alice private and publicly
      await mintPrivate(BigInt(1e12), aliceAddress);
      await bananaCoin.methods.mint_public(aliceAddress, 1e12).send().wait();
    });

    beforeEach(async () => {
      FeeAmount = 1n;
      MaxFee = BigInt(20e9);
      RefundAmount = MaxFee - FeeAmount;
      RefundSecret = Fr.random();

      expect(gasSettings.getFeeLimit().toBigInt()).toEqual(MaxFee);

      [
        [InitialAlicePrivateBananas, InitialBobPrivateBananas, InitialFPCPrivateBananas],
        [InitialAlicePublicBananas, InitialBobPublicBananas, InitialFPCPublicBananas],
        [InitialAliceGas, InitialFPCGas, InitialSequencerGas],
      ] = await Promise.all([
        bananaPrivateBalances(aliceAddress, bobAddress, bananaFPC.address),
        bananaPublicBalances(aliceAddress, bobAddress, bananaFPC.address),
        gasBalances(aliceAddress, bananaFPC.address, sequencerAddress),
      ]);
    });

    it('pays fees for tx that dont run public app logic', async () => {
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
            gasSettings,
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
        addPendingShieldNoteToPXE(0, RefundAmount, computeSecretHash(RefundSecret), tx.txHash),
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
            gasSettings,
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
        addPendingShieldNoteToPXE(0, RefundAmount, computeSecretHash(RefundSecret), tx.txHash),
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
      const shieldSecretHash = computeSecretHash(shieldSecret);
      const tx = await bananaCoin.methods
        .shield(aliceAddress, shieldedBananas, shieldSecretHash, 0n)
        .send({
          fee: {
            gasSettings,
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
        addPendingShieldNoteToPXE(0, RefundAmount, computeSecretHash(RefundSecret), tx.txHash),
      ).resolves.toBeUndefined();
    });

    it('pays fees for tx that creates notes in both private and public', async () => {
      const privateTransfer = 1n;
      const shieldedBananas = 1n;
      const shieldSecret = Fr.random();
      const shieldSecretHash = computeSecretHash(shieldSecret);

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
            gasSettings,
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
        addPendingShieldNoteToPXE(0, RefundAmount, computeSecretHash(RefundSecret), tx.txHash),
      ).resolves.toBeUndefined();
    });

    it('rejects txs that dont have enough balance to cover gas costs', async () => {
      // deploy a copy of bananaFPC but don't fund it!
      const bankruptFPC = await FPCContract.deploy(aliceWallet, bananaCoin.address, gasTokenContract.address)
        .send()
        .deployed();

      await expectMapping(gasBalances, [bankruptFPC.address], [0n]);

      await expect(
        bananaCoin.methods
          .privately_mint_private_note(10)
          .send({
            // we need to skip public simulation otherwise the PXE refuses to accept the TX
            skipPublicSimulation: true,
            fee: {
              gasSettings,
              paymentMethod: new PrivateFeePaymentMethod(
                bananaCoin.address,
                bankruptFPC.address,
                aliceWallet,
                RefundSecret,
              ),
            },
          })
          .wait(),
      ).rejects.toThrow('Tx dropped by P2P node.');
    });
  });

  it('fails transaction that error in setup', async () => {
    const OutrageousPublicAmountAliceDoesNotHave = BigInt(100e12);

    // simulation throws an error when setup fails
    await expect(
      bananaCoin.methods
        .transfer_public(aliceAddress, sequencerAddress, OutrageousPublicAmountAliceDoesNotHave, 0)
        .send({
          fee: {
            gasSettings,
            paymentMethod: new BuggedSetupFeePaymentMethod(bananaCoin.address, bananaFPC.address, wallets[0]),
          },
        })
        .wait(),
    ).rejects.toThrow(/Message not authorized by account 'is_valid == true'/);

    // so does the sequencer
    await expect(
      bananaCoin.methods
        .transfer_public(aliceAddress, sequencerAddress, OutrageousPublicAmountAliceDoesNotHave, 0)
        .send({
          skipPublicSimulation: true,
          fee: {
            gasSettings,
            paymentMethod: new BuggedSetupFeePaymentMethod(bananaCoin.address, bananaFPC.address, wallets[0]),
          },
        })
        .wait(),
    ).rejects.toThrow(/Transaction [0-9a-f]{64} was dropped\. Reason: Tx dropped by P2P node\./);
  });

  it('fails transaction that error in teardown', async () => {
    /**
     * We trigger an error in teardown by having the FPC authorize a transfer of its entire balance to Alice
     * as part of app logic. This will cause the FPC to not have enough funds to pay the refund back to Alice.
     */
    const PublicMintedAlicePublicBananas = 100_000_000_000n;

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

    await expect(
      bananaCoin.methods
        .mint_public(aliceAddress, 1n) // random operation
        .send({
          fee: {
            gasSettings,
            paymentMethod: new BuggedTeardownFeePaymentMethod(bananaCoin.address, bananaFPC.address, wallets[0]),
          },
        })
        .wait(),
    ).rejects.toThrow(/invalid nonce/);

    // node also drops
    await expect(
      bananaCoin.methods
        .mint_public(aliceAddress, 1n) // random operation
        .send({
          skipPublicSimulation: true,
          fee: {
            gasSettings,
            paymentMethod: new BuggedTeardownFeePaymentMethod(bananaCoin.address, bananaFPC.address, wallets[0]),
          },
        })
        .wait(),
    ).rejects.toThrow(/Transaction [0-9a-f]{64} was dropped\. Reason: Tx dropped by P2P node\./);

    // nothing happened
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
  });

  function logFunctionSignatures(artifact: ContractArtifact, logger: DebugLogger) {
    artifact.functions.forEach(fn => {
      const sig = decodeFunctionSignature(fn.name, fn.parameters);
      logger.verbose(`${FunctionSelector.fromNameAndParameters(fn.name, fn.parameters)} => ${artifact.name}.${sig} `);
    });
  }

  const mintPrivate = async (amount: bigint, address: AztecAddress) => {
    // Mint bananas privately
    const secret = Fr.random();
    const secretHash = computeSecretHash(secret);
    logger.debug(`Minting ${amount} bananas privately for ${address} with secret ${secretHash.toString()}`);
    const receipt = await bananaCoin.methods.mint_private(amount, secretHash).send().wait();

    // Setup auth wit
    await addPendingShieldNoteToPXE(0, amount, secretHash, receipt.txHash);
    const txClaim = bananaCoin.methods.redeem_shield(address, amount, secret).send();
    const receiptClaim = await txClaim.wait({ debug: true });
    const { visibleNotes } = receiptClaim.debugInfo!;
    expect(visibleNotes[0].note.items[0].toBigInt()).toBe(amount);
  };

  const addPendingShieldNoteToPXE = async (accountIndex: number, amount: bigint, secretHash: Fr, txHash: TxHash) => {
    const note = new Note([new Fr(amount), secretHash]);
    const extendedNote = new ExtendedNote(
      note,
      wallets[accountIndex].getAddress(),
      bananaCoin.address,
      BananaCoin.storage.pending_shields.slot,
      BananaCoin.notes.TransparentNote.id,
      txHash,
    );
    await wallets[accountIndex].addNote(extendedNote);
  };
});

class BuggedSetupFeePaymentMethod extends PublicFeePaymentMethod {
  override getFunctionCalls(gasSettings: GasSettings): Promise<FunctionCall[]> {
    const maxFee = gasSettings.getFeeLimit();
    const nonce = Fr.random();
    const messageHash = computeAuthWitMessageHash(
      this.paymentContract,
      this.wallet.getChainId(),
      this.wallet.getVersion(),
      {
        args: [this.wallet.getAddress(), this.paymentContract, maxFee, nonce],
        functionData: new FunctionData(
          FunctionSelector.fromSignature('transfer_public((Field),(Field),Field,Field)'),
          false,
        ),
        to: this.asset,
      },
    );

    const tooMuchFee = new Fr(maxFee.toBigInt() * 2n);

    return Promise.resolve([
      this.wallet.setPublicAuthWit(messageHash, true).request(),
      {
        to: this.getPaymentContract(),
        functionData: new FunctionData(
          FunctionSelector.fromSignature('fee_entrypoint_public(Field,(Field),Field)'),
          true,
        ),
        args: [tooMuchFee, this.asset, nonce],
      },
    ]);
  }
}

class BuggedTeardownFeePaymentMethod extends PublicFeePaymentMethod {
  override async getFunctionCalls(gasSettings: GasSettings): Promise<FunctionCall[]> {
    // authorize the FPC to take the max fee from Alice
    const nonce = Fr.random();
    const maxFee = gasSettings.getFeeLimit();
    const messageHash1 = computeAuthWitMessageHash(
      this.paymentContract,
      this.wallet.getChainId(),
      this.wallet.getVersion(),
      {
        args: [this.wallet.getAddress(), this.paymentContract, maxFee, nonce],
        functionData: new FunctionData(
          FunctionSelector.fromSignature('transfer_public((Field),(Field),Field,Field)'),
          false,
        ),
        to: this.asset,
      },
    );

    // authorize the FPC to take the maxFee
    // do this first because we only get 2 feepayload calls
    await this.wallet.setPublicAuthWit(messageHash1, true).send().wait();

    return Promise.resolve([
      // in this, we're actually paying the fee in setup
      {
        to: this.getPaymentContract(),
        functionData: new FunctionData(
          FunctionSelector.fromSignature('fee_entrypoint_public(Field,(Field),Field)'),
          true,
        ),
        args: [maxFee, this.asset, nonce],
      },
      // and trying to take a little extra in teardown, but specify a bad nonce
      {
        to: this.asset,
        functionData: new FunctionData(
          FunctionSelector.fromSignature('transfer_public((Field),(Field),Field,Field)'),
          false,
        ),
        args: [this.wallet.getAddress(), this.paymentContract, new Fr(1), Fr.random()],
      },
    ]);
  }
}
