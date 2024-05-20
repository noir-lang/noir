import { NestedProcessReturnValues, PublicSimulationOutput } from './public_simulation_output.js';
import { Tx } from './tx.js';

// REFACTOR: Review what we need to expose to the user when running a simulation.
// Eg tx already has encrypted and unencrypted logs, but those cover only the ones
// emitted during private. We need the ones from ProcessOutput to include the public
// ones as well. However, those would only be present if the user chooses to simulate
// the public side of things. This also points at this class needing to be split into
// two: one with just private simulation, and one that also includes public simulation.
export class SimulatedTx {
  constructor(
    public tx: Tx,
    public privateReturnValues?: NestedProcessReturnValues,
    public publicOutput?: PublicSimulationOutput,
  ) {}

  /**
   * Convert a SimulatedTx class object to a plain JSON object.
   * @returns A plain object with SimulatedTx properties.
   */
  public toJSON() {
    return {
      tx: this.tx.toJSON(),
      privateReturnValues: this.privateReturnValues && this.privateReturnValues.toJSON(),
      publicOutput: this.publicOutput && this.publicOutput.toJSON(),
    };
  }

  /**
   * Convert a plain JSON object to a Tx class object.
   * @param obj - A plain Tx JSON object.
   * @returns A Tx class object.
   */
  public static fromJSON(obj: any) {
    const tx = Tx.fromJSON(obj.tx);
    const publicOutput = obj.publicOutput ? PublicSimulationOutput.fromJSON(obj.publicOutput) : undefined;
    const privateReturnValues = obj.privateReturnValues
      ? NestedProcessReturnValues.fromJSON(obj.privateReturnValues)
      : undefined;

    return new SimulatedTx(tx, privateReturnValues, publicOutput);
  }
}
