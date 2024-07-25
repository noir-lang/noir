import { PublicKernelType, type Tx, type TxValidator } from '@aztec/circuit-types';
import { type AztecAddress, type Fr } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { GasTokenArtifact } from '@aztec/protocol-contracts/gas-token';
import { AbstractPhaseManager, computeFeePayerBalanceStorageSlot } from '@aztec/simulator';

/** Provides a view into public contract state */
export interface PublicStateSource {
  storageRead: (contractAddress: AztecAddress, slot: Fr) => Promise<Fr>;
}

export class GasTxValidator implements TxValidator<Tx> {
  #log = createDebugLogger('aztec:sequencer:tx_validator:tx_gas');
  #publicDataSource: PublicStateSource;
  #gasTokenAddress: AztecAddress;

  constructor(publicDataSource: PublicStateSource, gasTokenAddress: AztecAddress, public enforceFees: boolean) {
    this.#publicDataSource = publicDataSource;
    this.#gasTokenAddress = gasTokenAddress;
  }

  async validateTxs(txs: Tx[]): Promise<[validTxs: Tx[], invalidTxs: Tx[]]> {
    const validTxs: Tx[] = [];
    const invalidTxs: Tx[] = [];

    for (const tx of txs) {
      if (await this.#validateTxFee(tx)) {
        validTxs.push(tx);
      } else {
        invalidTxs.push(tx);
      }
    }

    return [validTxs, invalidTxs];
  }

  async #validateTxFee(tx: Tx): Promise<boolean> {
    const feePayer = tx.data.feePayer;
    // TODO(@spalladino) Eventually remove the is_zero condition as we should always charge fees to every tx
    if (feePayer.isZero()) {
      if (this.enforceFees) {
        this.#log.warn(`Rejecting transaction ${tx.getTxHash()} due to missing fee payer`);
      } else {
        return true;
      }
    }

    // Compute the maximum fee that this tx may pay, based on its gasLimits and maxFeePerGas
    const feeLimit = tx.data.constants.txContext.gasSettings.getFeeLimit();

    // Read current balance of the feePayer
    const initialBalance = await this.#publicDataSource.storageRead(
      this.#gasTokenAddress,
      computeFeePayerBalanceStorageSlot(feePayer),
    );

    // If there is a claim in this tx that increases the fee payer balance in gas token, add it to balance
    const { [PublicKernelType.SETUP]: setupFns } = AbstractPhaseManager.extractEnqueuedPublicCallsByPhase(tx);
    const claimFunctionCall = setupFns.find(
      fn =>
        fn.contractAddress.equals(this.#gasTokenAddress) &&
        fn.callContext.msgSender.equals(this.#gasTokenAddress) &&
        fn.callContext.functionSelector.equals(
          GasTokenArtifact.functions.find(f => f.name === '_increase_public_balance')!,
        ) &&
        fn.args[0].equals(feePayer) &&
        !fn.callContext.isStaticCall &&
        !fn.callContext.isDelegateCall,
    );

    const balance = claimFunctionCall ? initialBalance.add(claimFunctionCall.args[1]) : initialBalance;
    if (balance.lt(feeLimit)) {
      this.#log.info(`Rejecting transaction due to not enough fee payer balance`, { feePayer, balance, feeLimit });
      return false;
    }
    return true;
  }
}
