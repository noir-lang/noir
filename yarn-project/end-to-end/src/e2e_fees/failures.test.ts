import {
  type AccountWallet,
  type AztecAddress,
  Fr,
  type FunctionCall,
  FunctionSelector,
  PublicFeePaymentMethod,
  TxStatus,
  computeAuthWitMessageHash,
} from '@aztec/aztec.js';
import { FunctionData, type GasSettings } from '@aztec/circuits.js';
import { type TokenContract as BananaCoin, type FPCContract } from '@aztec/noir-contracts.js';

import { expectMapping } from '../fixtures/utils.js';
import { FeesTest } from './fees_test.js';

describe('e2e_fees failures', () => {
  let aliceWallet: AccountWallet;
  let aliceAddress: AztecAddress;
  let sequencerAddress: AztecAddress;
  let bananaCoin: BananaCoin;
  let bananaFPC: FPCContract;
  let gasSettings: GasSettings;

  const t = new FeesTest('failures');

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    ({ aliceWallet, aliceAddress, sequencerAddress, bananaCoin, bananaFPC, gasSettings } = await t.setup());
  });

  afterAll(async () => {
    await t.teardown();
  });

  it('reverts transactions but still pays fees using PublicFeePaymentMethod', async () => {
    const OutrageousPublicAmountAliceDoesNotHave = BigInt(1e15);
    const PublicMintedAlicePublicBananas = BigInt(1e12);

    const [initialAlicePrivateBananas, initialFPCPrivateBananas] = await t.bananaPrivateBalances(
      aliceAddress,
      bananaFPC.address,
    );
    const [initialAlicePublicBananas, initialFPCPublicBananas] = await t.bananaPublicBalances(
      aliceAddress,
      bananaFPC.address,
    );
    const [initialAliceGas, initialFPCGas, initialSequencerGas] = await t.gasBalances(
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
            paymentMethod: new PublicFeePaymentMethod(bananaCoin.address, bananaFPC.address, aliceWallet),
          },
        })
        .wait(),
    ).rejects.toThrow(/attempt to subtract with underflow 'hi == high'/);

    // we did not pay the fee, because we did not submit the TX
    await expectMapping(
      t.bananaPrivateBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [initialAlicePrivateBananas, initialFPCPrivateBananas, 0n],
    );
    await expectMapping(
      t.bananaPublicBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [initialAlicePublicBananas + PublicMintedAlicePublicBananas, initialFPCPublicBananas, 0n],
    );
    await expectMapping(
      t.gasBalances,
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
          paymentMethod: new PublicFeePaymentMethod(bananaCoin.address, bananaFPC.address, aliceWallet),
        },
      })
      .wait({ dontThrowOnRevert: true });

    expect(txReceipt.status).toBe(TxStatus.REVERTED);
    const feeAmount = txReceipt.transactionFee!;

    // and thus we paid the fee
    await expectMapping(
      t.bananaPrivateBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [initialAlicePrivateBananas, initialFPCPrivateBananas, 0n],
    );
    await expectMapping(
      t.bananaPublicBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [initialAlicePublicBananas + PublicMintedAlicePublicBananas - feeAmount, initialFPCPublicBananas + feeAmount, 0n],
    );
    await expectMapping(
      t.gasBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [initialAliceGas, initialFPCGas - feeAmount, initialSequencerGas + feeAmount],
    );

    // TODO(#4712) - demonstrate reverts with the PrivateFeePaymentMethod.
    // Can't do presently because all logs are "revertible" so we lose notes that get broadcasted during unshielding.
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
            paymentMethod: new BuggedSetupFeePaymentMethod(bananaCoin.address, bananaFPC.address, aliceWallet),
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
            paymentMethod: new BuggedSetupFeePaymentMethod(bananaCoin.address, bananaFPC.address, aliceWallet),
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

    const [initialAlicePrivateBananas, initialFPCPrivateBananas] = await t.bananaPrivateBalances(
      aliceAddress,
      bananaFPC.address,
    );
    const [initialAlicePublicBananas, initialFPCPublicBananas] = await t.bananaPublicBalances(
      aliceAddress,
      bananaFPC.address,
    );
    const [initialAliceGas, initialFPCGas, initialSequencerGas] = await t.gasBalances(
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
            paymentMethod: new BuggedTeardownFeePaymentMethod(bananaCoin.address, bananaFPC.address, aliceWallet),
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
            paymentMethod: new BuggedTeardownFeePaymentMethod(bananaCoin.address, bananaFPC.address, aliceWallet),
          },
        })
        .wait(),
    ).rejects.toThrow(/Transaction [0-9a-f]{64} was dropped\. Reason: Tx dropped by P2P node\./);

    // nothing happened
    await expectMapping(
      t.bananaPrivateBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [initialAlicePrivateBananas, initialFPCPrivateBananas, 0n],
    );
    await expectMapping(
      t.bananaPublicBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [initialAlicePublicBananas + PublicMintedAlicePublicBananas, initialFPCPublicBananas, 0n],
    );
    await expectMapping(
      t.gasBalances,
      [aliceAddress, bananaFPC.address, sequencerAddress],
      [initialAliceGas, initialFPCGas, initialSequencerGas],
    );
  });
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
