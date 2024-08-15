import {
  type AccountWallet,
  type AztecAddress,
  ExtendedNote,
  type FeePaymentMethod,
  type FunctionCall,
  Note,
  type Wallet,
} from '@aztec/aztec.js';
import { Fr, type GasSettings } from '@aztec/circuits.js';
import { deriveStorageSlotInMap, siloNullifier } from '@aztec/circuits.js/hash';
import { FunctionSelector, FunctionType } from '@aztec/foundation/abi';
import { type PrivateFPCContract, TokenWithRefundsContract } from '@aztec/noir-contracts.js';

import { expectMapping } from '../fixtures/utils.js';
import { FeesTest } from './fees_test.js';

describe('e2e_fees/private_refunds', () => {
  let aliceWallet: AccountWallet;
  let aliceAddress: AztecAddress;
  let bobAddress: AztecAddress;
  let tokenWithRefunds: TokenWithRefundsContract;
  let privateFPC: PrivateFPCContract;

  let initialAliceBalance: bigint;
  // Bob is the admin of the fee paying contract
  let initialBobBalance: bigint;
  let initialFPCGasBalance: bigint;

  const t = new FeesTest('private_refunds');

  beforeAll(async () => {
    await t.applyInitialAccountsSnapshot();
    await t.applyPublicDeployAccountsSnapshot();
    await t.applyDeployFeeJuiceSnapshot();
    await t.applyTokenWithRefundsAndFPC();
    await t.applyFundAliceWithTokens();
    ({ aliceWallet, aliceAddress, bobAddress, privateFPC, tokenWithRefunds } = await t.setup());
    t.logger.debug(`Alice address: ${aliceAddress}`);

    // We give Alice access to Bob's notes because Alice is used to check if balances are correct.
    aliceWallet.setScopes([aliceAddress, bobAddress]);
  });

  afterAll(async () => {
    await t.teardown();
  });

  beforeEach(async () => {
    [[initialAliceBalance, initialBobBalance], [initialFPCGasBalance]] = await Promise.all([
      t.getTokenWithRefundsBalanceFn(aliceAddress, t.bobAddress),
      t.getGasBalanceFn(privateFPC.address),
    ]);
  });

  it('can do private payments and refunds', async () => {
    // 1. We generate randomness for Alice and derive randomness for Bob.
    const aliceRandomness = Fr.random(); // Called user_randomness in contracts
    const bobRandomness = siloNullifier(privateFPC.address, aliceRandomness); // Called fee_payer_randomness in contracts

    // 2. We call arbitrary `private_get_name(...)` function to check that the fee refund flow works.
    const { txHash, transactionFee, debugInfo } = await tokenWithRefunds.methods
      .private_get_name()
      .send({
        fee: {
          gasSettings: t.gasSettings,
          paymentMethod: new PrivateRefundPaymentMethod(
            tokenWithRefunds.address,
            privateFPC.address,
            aliceWallet,
            aliceRandomness,
            bobRandomness,
            t.bobWallet.getAddress(), // Bob is the recipient of the fee notes.
          ),
        },
      })
      .wait({ debug: true });

    expect(transactionFee).toBeGreaterThan(0);

    // 3. We check that randomness for Bob was correctly emitted as a nullifier (Bobs needs it to reconstruct his note).
    const bobRandomnessFromLog = debugInfo?.nullifiers[1];
    expect(bobRandomnessFromLog).toEqual(bobRandomness);

    // 4. Now we compute the contents of the note containing the refund for Alice. The refund note value is simply
    // the fee limit minus the final transaction fee. The other 2 fields in the note are Alice's npk_m_hash and
    // the randomness.
    const refundNoteValue = t.gasSettings.getFeeLimit().sub(new Fr(transactionFee!));
    const aliceNpkMHash = t.aliceWallet.getCompleteAddress().publicKeys.masterNullifierPublicKey.hash();
    const aliceRefundNote = new Note([refundNoteValue, aliceNpkMHash, aliceRandomness]);

    // 5. If the refund flow worked it should have added emitted a note hash of the note we constructed above and we
    // should be able to add the note to our PXE. Just calling `pxe.addNote(...)` is enough of a check that the note
    // hash was emitted because the endpoint will compute the hash and then it will try to find it in the note hash
    // tree. If the note hash is not found in the tree, an error is thrown.
    await t.aliceWallet.addNote(
      new ExtendedNote(
        aliceRefundNote,
        t.aliceAddress,
        tokenWithRefunds.address,
        deriveStorageSlotInMap(TokenWithRefundsContract.storage.balances.slot, t.aliceAddress),
        TokenWithRefundsContract.notes.TokenNote.id,
        txHash,
      ),
    );

    // 6. Now we reconstruct the note for the final fee payment. It should contain the transaction fee, Bob's
    // npk_m_hash and the randomness.
    // Note that FPC emits randomness as unencrypted log and the tx fee is publicly know so Bob is able to reconstruct
    // his note just from on-chain data.
    const bobNpkMHash = t.bobWallet.getCompleteAddress().publicKeys.masterNullifierPublicKey.hash();
    const bobFeeNote = new Note([new Fr(transactionFee!), bobNpkMHash, bobRandomness]);

    // 7. Once again we add the note to PXE which computes the note hash and checks that it is in the note hash tree.
    await t.bobWallet.addNote(
      new ExtendedNote(
        bobFeeNote,
        t.bobAddress,
        tokenWithRefunds.address,
        deriveStorageSlotInMap(TokenWithRefundsContract.storage.balances.slot, t.bobAddress),
        TokenWithRefundsContract.notes.TokenNote.id,
        txHash,
      ),
    );

    // 8. At last we check that the gas balance of FPC has decreased exactly by the transaction fee ...
    await expectMapping(t.getGasBalanceFn, [privateFPC.address], [initialFPCGasBalance - transactionFee!]);
    // ... and that the transaction fee was correctly transferred from Alice to Bob.
    await expectMapping(
      t.getTokenWithRefundsBalanceFn,
      [aliceAddress, t.bobAddress],
      [initialAliceBalance - transactionFee!, initialBobBalance + transactionFee!],
    );
  });

  // TODO(#7694): Remove this test once the lacking feature in TXE is implemented.
  it('insufficient funded amount is correctly handled', async () => {
    // 1. We generate randomness for Alice and derive randomness for Bob.
    const aliceRandomness = Fr.random(); // Called user_randomness in contracts
    const bobRandomness = siloNullifier(privateFPC.address, aliceRandomness); // Called fee_payer_randomness in contracts

    // 2. We call arbitrary `private_get_name(...)` function to check that the fee refund flow works.
    await expect(
      tokenWithRefunds.methods.private_get_name().prove({
        fee: {
          gasSettings: t.gasSettings,
          paymentMethod: new PrivateRefundPaymentMethod(
            tokenWithRefunds.address,
            privateFPC.address,
            aliceWallet,
            aliceRandomness,
            bobRandomness,
            t.bobWallet.getAddress(), // Bob is the recipient of the fee notes.
            true, // We set max fee/funded amount to 1 to trigger the error.
          ),
        },
      }),
    ).rejects.toThrow('funded amount not enough to cover tx fee');
  });
});

class PrivateRefundPaymentMethod implements FeePaymentMethod {
  constructor(
    /**
     * The asset used to pay the fee.
     */
    private asset: AztecAddress,
    /**
     * Address which will hold the fee payment.
     */
    private paymentContract: AztecAddress,

    /**
     * An auth witness provider to authorize fee payments
     */
    private wallet: Wallet,

    /**
     * A randomness to mix in with the generated refund note for the sponsored user.
     * Use this to reconstruct note preimages for the PXE.
     */
    private userRandomness: Fr,

    /**
     * A randomness to mix in with the generated fee note for the fee payer.
     * Use this to reconstruct note preimages for the PXE.
     */
    private feePayerRandomness: Fr,

    /**
     * Address that the FPC sends notes it receives to.
     */
    private feeRecipient: AztecAddress,

    /**
     * If true, the max fee will be set to 1.
     * TODO(#7694): Remove this param once the lacking feature in TXE is implemented.
     */
    private setMaxFeeToOne = false,
  ) {}

  /**
   * The asset used to pay the fee.
   * @returns The asset used to pay the fee.
   */
  getAsset() {
    return this.asset;
  }

  getFeePayer(): Promise<AztecAddress> {
    return Promise.resolve(this.paymentContract);
  }

  /**
   * Creates a function call to pay the fee in the given asset.
   * @param gasSettings - The gas settings.
   * @returns The function call to pay the fee.
   */
  async getFunctionCalls(gasSettings: GasSettings): Promise<FunctionCall[]> {
    // We assume 1:1 exchange rate between fee juice and token. But in reality you would need to convert feeLimit
    // (maxFee) to be in token denomination.
    const maxFee = this.setMaxFeeToOne ? Fr.ONE : gasSettings.getFeeLimit();

    await this.wallet.createAuthWit({
      caller: this.paymentContract,
      action: {
        name: 'setup_refund',
        args: [
          this.feeRecipient,
          this.wallet.getCompleteAddress().address,
          maxFee,
          this.userRandomness,
          this.feePayerRandomness,
        ],
        selector: FunctionSelector.fromSignature('setup_refund((Field),(Field),Field,Field,Field)'),
        type: FunctionType.PRIVATE,
        isStatic: false,
        to: this.asset,
        returnTypes: [],
      },
    });

    return [
      {
        name: 'fund_transaction_privately',
        to: this.paymentContract,
        selector: FunctionSelector.fromSignature('fund_transaction_privately(Field,(Field),Field)'),
        type: FunctionType.PRIVATE,
        isStatic: false,
        args: [maxFee, this.asset, this.userRandomness],
        returnTypes: [],
      },
    ];
  }
}
