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
import { type PrivateFPCContract, PrivateTokenContract } from '@aztec/noir-contracts.js';

import { expectMapping } from '../fixtures/utils.js';
import { FeesTest } from './fees_test.js';

describe('e2e_fees/private_refunds', () => {
  let aliceWallet: Wallet;
  let aliceAddress: AztecAddress;
  let privateToken: PrivateTokenContract;
  let privateFPC: PrivateFPCContract;

  let InitialAlicePrivateTokens: bigint;
  let InitialBobPrivateTokens: bigint;
  let InitialPrivateFPCGas: bigint;

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
    [[InitialAlicePrivateTokens, InitialBobPrivateTokens], [InitialPrivateFPCGas]] = await Promise.all([
      t.privateTokenBalances(aliceAddress, t.bobAddress),
      t.gasBalances(privateFPC.address),
    ]);
  });

  it('can do private payments and refunds', async () => {
    const bobKeyHash = t.bobWallet.getCompleteAddress().publicKeys.masterNullifierPublicKey.hash();
    const rebateNonce = new Fr(42);
    const tx = await privateToken.methods
      .private_get_name()
      .send({
        fee: {
          gasSettings: t.gasSettings,
          paymentMethod: new PrivateRefundPaymentMethod(
            privateToken.address,
            privateFPC.address,
            aliceWallet,
            rebateNonce,
            bobKeyHash,
          ),
        },
      })
      .wait();

    expect(tx.transactionFee).toBeGreaterThan(0);

    const refundedNoteValue = t.gasSettings.getFeeLimit().sub(new Fr(tx.transactionFee!));
    const aliceKeyHash = t.aliceWallet.getCompleteAddress().publicKeys.masterNullifierPublicKey.hash();
    const aliceRefundNote = new Note([refundedNoteValue, aliceKeyHash, rebateNonce]);
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

    const bobFeeNote = new Note([new Fr(tx.transactionFee!), bobKeyHash, rebateNonce]);
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

    await expectMapping(t.gasBalances, [privateFPC.address], [InitialPrivateFPCGas - tx.transactionFee!]);
    await expectMapping(
      t.privateTokenBalances,
      [aliceAddress, t.bobAddress],
      [InitialAlicePrivateTokens - tx.transactionFee!, InitialBobPrivateTokens + tx.transactionFee!],
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
     * A nonce to mix in with the generated notes.
     * Use this to reconstruct note preimages for the PXE.
     */
    private rebateNonce: Fr,

    /**
     * The hash of the nullifier private key that the FPC sends notes it receives to.
     */
    private feeRecipientNPKMHash: Fr,
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
        args: [this.feeRecipientNPKMHash, this.wallet.getCompleteAddress().address, maxFee, this.rebateNonce],
        selector: FunctionSelector.fromSignature('setup_refund(Field,(Field),Field,Field)'),
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
        args: [maxFee, this.asset, this.rebateNonce],
        returnTypes: [],
      },
    ];
  }
}
