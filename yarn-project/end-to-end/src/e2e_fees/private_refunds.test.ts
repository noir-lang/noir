import {
  type AztecAddress,
  ExtendedNote,
  type FeePaymentMethod,
  type FunctionCall,
  Note,
  type Wallet,
} from '@aztec/aztec.js';
import { Fr, type GasSettings } from '@aztec/circuits.js';
import { FunctionSelector, FunctionType } from '@aztec/foundation/abi';
import { poseidon2Hash } from '@aztec/foundation/crypto';
import { type PrivateFPCContract, PrivateTokenContract } from '@aztec/noir-contracts.js';

import { expectMapping } from '../fixtures/utils.js';
import { FeesTest } from './fees_test.js';

describe('e2e_fees/private_refunds', () => {
  let aliceWallet: Wallet;
  let aliceAddress: AztecAddress;
  let privateToken: PrivateTokenContract;
  let privateFPC: PrivateFPCContract;

  let initialAliceBalance: bigint;
  // Bob is the admin of the fee paying contract
  let initialBobBalance: bigint;
  let initialFPCGasBalance: bigint;

  const t = new FeesTest('private_refunds');

  beforeAll(async () => {
    await t.applyInitialAccountsSnapshot();
    await t.applyPublicDeployAccountsSnapshot();
    await t.applyDeployGasTokenSnapshot();
    await t.applyPrivateTokenAndFPC();
    await t.applyFundAliceWithPrivateTokens();
    ({ aliceWallet, aliceAddress, privateFPC, privateToken } = await t.setup());
    t.logger.debug(`Alice address: ${aliceAddress}`);
  });

  afterAll(async () => {
    await t.teardown();
  });

  beforeEach(async () => {
    [[initialAliceBalance, initialBobBalance], [initialFPCGasBalance]] = await Promise.all([
      t.getPrivateTokenBalanceFn(aliceAddress, t.bobAddress),
      t.getGasBalanceFn(privateFPC.address),
    ]);
  });

  it('can do private payments and refunds', async () => {
    // 1. We get the hash of Bob's master nullifier public key. The corresponding nullifier secret key can later on
    // be used to nullify/spend the note that contains the npk_m_hash.
    // TODO(#7324): The values in complete address are currently not updated after the keys are rotated so this does
    // not work with key rotation as the key might be the old one and then we would fetch a new one in the contract.
    const bobNpkMHash = t.bobWallet.getCompleteAddress().publicKeys.masterNullifierPublicKey.hash();
    const aliceRandomness = Fr.random(); // Called user_randomness in contracts
    const bobRandomness = poseidon2Hash([aliceRandomness]); // Called fee_payer_randomness in contracts

    // 2. We call arbitrary `private_get_name(...)` function to check that the fee refund flow works.
    const tx = await privateToken.methods
      .private_get_name()
      .send({
        fee: {
          gasSettings: t.gasSettings,
          paymentMethod: new PrivateRefundPaymentMethod(
            privateToken.address,
            privateFPC.address,
            aliceWallet,
            aliceRandomness,
            bobRandomness,
            bobNpkMHash, // We use Bob's npk_m_hash in the notes that contain the transaction fee.
          ),
        },
      })
      .wait();

    expect(tx.transactionFee).toBeGreaterThan(0);

    // 3. We check that randomness for Bob was correctly emitted as an unencrypted log (Bobs needs it to reconstruct his note).
    const resp = await aliceWallet.getUnencryptedLogs({ txHash: tx.txHash });
    const bobRandomnessFromLog = Fr.fromBuffer(resp.logs[0].log.data);
    expect(bobRandomnessFromLog).toEqual(bobRandomness);

    // 4. Now we compute the contents of the note containing the refund for Alice. The refund note value is simply
    // the fee limit minus the final transaction fee. The other 2 fields in the note are Alice's npk_m_hash and
    // the randomness.
    const refundNoteValue = t.gasSettings.getFeeLimit().sub(new Fr(tx.transactionFee!));
    // TODO(#7324): The values in complete address are currently not updated after the keys are rotated so this does
    // not work with key rotation as the key might be the old one and then we would fetch a new one in the contract.
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
        privateToken.address,
        PrivateTokenContract.storage.balances.slot,
        PrivateTokenContract.notes.TokenNote.id,
        tx.txHash,
      ),
    );

    // 6. Now we reconstruct the note for the final fee payment. It should contain the transaction fee, Bob's
    // npk_m_hash (set in the paymentMethod above) and the randomness.
    // Note that FPC emits randomness as unencrypted log and the tx fee is publicly know so Bob is able to reconstruct
    // his note just from on-chain data.
    const bobFeeNote = new Note([new Fr(tx.transactionFee!), bobNpkMHash, bobRandomness]);

    // 7. Once again we add the note to PXE which computes the note hash and checks that it is in the note hash tree.
    await t.bobWallet.addNote(
      new ExtendedNote(
        bobFeeNote,
        t.bobAddress,
        privateToken.address,
        PrivateTokenContract.storage.balances.slot,
        PrivateTokenContract.notes.TokenNote.id,
        tx.txHash,
      ),
    );

    // 8. At last we check that the gas balance of FPC has decreased exactly by the transaction fee ...
    await expectMapping(t.getGasBalanceFn, [privateFPC.address], [initialFPCGasBalance - tx.transactionFee!]);
    // ... and that the transaction fee was correctly transferred from Alice to Bob.
    await expectMapping(
      t.getPrivateTokenBalanceFn,
      [aliceAddress, t.bobAddress],
      [initialAliceBalance - tx.transactionFee!, initialBobBalance + tx.transactionFee!],
    );
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
     * The hash of the master nullifier public key that the FPC sends notes it receives to.
     */
    private feeRecipientNpkMHash: Fr,
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
    const maxFee = gasSettings.getFeeLimit();

    await this.wallet.createAuthWit({
      caller: this.paymentContract,
      action: {
        name: 'setup_refund',
        args: [
          this.feeRecipientNpkMHash,
          this.wallet.getCompleteAddress().address,
          maxFee,
          this.userRandomness,
          this.feePayerRandomness,
        ],
        selector: FunctionSelector.fromSignature('setup_refund(Field,(Field),Field,Field,Field)'),
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
