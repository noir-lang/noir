import { CombinedAccumulatedData, CombinedConstantData, Fr, Gas } from '@aztec/circuits.js';
import { mapValues } from '@aztec/foundation/collection';

import { EncryptedTxL2Logs, UnencryptedTxL2Logs } from '../logs/tx_l2_logs.js';
import { type SimulationError } from '../simulation_error.js';
import { type PublicKernelType } from './processed_tx.js';

/** Return values of simulating a circuit. */
export type ProcessReturnValues = Fr[] | undefined;

/** Return values of simulating complete callstack. */
export class NestedProcessReturnValues {
  values: ProcessReturnValues;
  nested: NestedProcessReturnValues[];

  constructor(values: ProcessReturnValues, nested?: NestedProcessReturnValues[]) {
    this.values = values;
    this.nested = nested ?? [];
  }

  toJSON(): any {
    return {
      values: this.values?.map(fr => fr.toString()),
      nested: this.nested.map(n => n.toJSON()),
    };
  }

  static fromJSON(json: any): NestedProcessReturnValues {
    return new NestedProcessReturnValues(
      json.values?.map(Fr.fromString),
      json.nested?.map((n: any) => NestedProcessReturnValues.fromJSON(n)),
    );
  }
}

/**
 * Outputs of processing the public component of a transaction.
 */
export class PublicSimulationOutput {
  constructor(
    public encryptedLogs: EncryptedTxL2Logs,
    public unencryptedLogs: UnencryptedTxL2Logs,
    public revertReason: SimulationError | undefined,
    public constants: CombinedConstantData,
    public end: CombinedAccumulatedData,
    public publicReturnValues: NestedProcessReturnValues[],
    public gasUsed: Partial<Record<PublicKernelType, Gas>>,
  ) {}

  toJSON() {
    return {
      encryptedLogs: this.encryptedLogs.toJSON(),
      unencryptedLogs: this.unencryptedLogs.toJSON(),
      revertReason: this.revertReason,
      constants: this.constants.toBuffer().toString('hex'),
      end: this.end.toBuffer().toString('hex'),
      publicReturnValues: this.publicReturnValues.map(returns => returns?.toJSON()),
      gasUsed: mapValues(this.gasUsed, gas => gas?.toJSON()),
    };
  }

  static fromJSON(json: any): PublicSimulationOutput {
    return new PublicSimulationOutput(
      EncryptedTxL2Logs.fromJSON(json.encryptedLogs),
      UnencryptedTxL2Logs.fromJSON(json.unencryptedLogs),
      json.revertReason,
      CombinedConstantData.fromBuffer(Buffer.from(json.constants, 'hex')),
      CombinedAccumulatedData.fromBuffer(Buffer.from(json.end, 'hex')),
      Array.isArray(json.publicReturnValues)
        ? json.publicReturnValues.map((returns: any) => NestedProcessReturnValues.fromJSON(returns))
        : [],
      mapValues(json.gasUsed, gas => (gas ? Gas.fromJSON(gas) : undefined)),
    );
  }
}
