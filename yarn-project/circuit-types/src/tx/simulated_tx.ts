import { CombinedAccumulatedData, CombinedConstantData, Fr } from '@aztec/circuits.js';

import { EncryptedTxL2Logs, UnencryptedTxL2Logs } from '../logs/index.js';
import { type ProcessedTx } from './processed_tx.js';
import { Tx } from './tx.js';

/** Return values of simulating a circuit. */
export type ProcessReturnValues = Fr[] | undefined;

/**
 * Outputs of processing the public component of a transaction.
 * REFACTOR: Rename.
 */
export type ProcessOutput = Pick<ProcessedTx, 'encryptedLogs' | 'unencryptedLogs' | 'revertReason'> &
  Pick<ProcessedTx['data'], 'constants' | 'end'> & { publicReturnValues: ProcessReturnValues };

function processOutputToJSON(output: ProcessOutput) {
  return {
    encryptedLogs: output.encryptedLogs.toJSON(),
    unencryptedLogs: output.unencryptedLogs.toJSON(),
    revertReason: output.revertReason,
    constants: output.constants.toBuffer().toString('hex'),
    end: output.end.toBuffer().toString('hex'),
    publicReturnValues: output.publicReturnValues?.map(fr => fr.toString()),
  };
}

function processOutputFromJSON(json: any): ProcessOutput {
  return {
    encryptedLogs: EncryptedTxL2Logs.fromJSON(json.encryptedLogs),
    unencryptedLogs: UnencryptedTxL2Logs.fromJSON(json.unencryptedLogs),
    revertReason: json.revertReason,
    constants: CombinedConstantData.fromBuffer(Buffer.from(json.constants, 'hex')),
    end: CombinedAccumulatedData.fromBuffer(Buffer.from(json.end, 'hex')),
    publicReturnValues: json.publicReturnValues?.map(Fr.fromString),
  };
}

// REFACTOR: Review what we need to expose to the user when running a simulation.
// Eg tx already has encrypted and unencrypted logs, but those cover only the ones
// emitted during private. We need the ones from ProcessOutput to include the public
// ones as well. However, those would only be present if the user chooses to simulate
// the public side of things. This also points at this class needing to be split into
// two: one with just private simulation, and one that also includes public simulation.
export class SimulatedTx {
  constructor(public tx: Tx, public privateReturnValues?: ProcessReturnValues, public publicOutput?: ProcessOutput) {}

  /**
   * Convert a SimulatedTx class object to a plain JSON object.
   * @returns A plain object with SimulatedTx properties.
   */
  public toJSON() {
    return {
      tx: this.tx.toJSON(),
      privateReturnValues: this.privateReturnValues?.map(fr => fr.toString()),
      publicOutput: this.publicOutput && processOutputToJSON(this.publicOutput),
    };
  }

  /**
   * Convert a plain JSON object to a Tx class object.
   * @param obj - A plain Tx JSON object.
   * @returns A Tx class object.
   */
  public static fromJSON(obj: any) {
    const tx = Tx.fromJSON(obj.tx);
    const publicOutput = obj.publicOutput ? processOutputFromJSON(obj.publicOutput) : undefined;
    const privateReturnValues = obj.privateReturnValues?.map(Fr.fromString);

    return new SimulatedTx(tx, privateReturnValues, publicOutput);
  }
}
