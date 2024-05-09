import { Fr, Gas } from '@aztec/circuits.js';

import { PublicKernelType } from './processed_tx.js';
import { type ProcessReturnValues, PublicSimulationOutput } from './public_simulation_output.js';
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
    public privateReturnValues?: ProcessReturnValues,
    public publicOutput?: PublicSimulationOutput,
  ) {}

  /**
   * Returns suggested total and teardown gas limits for the simulated tx.
   * Note that public gas usage is only accounted for if the publicOutput is present.
   * @param pad - Percentage to pad the suggested gas limits by, defaults to 10%.
   */
  public getGasLimits(pad = 0.1) {
    const privateGasUsed = this.tx.data.publicInputs.end.gasUsed;
    if (this.publicOutput) {
      const publicGasUsed = Object.values(this.publicOutput.gasUsed).reduce(
        (total, current) => total.add(current),
        Gas.empty(),
      );
      const teardownGas = this.publicOutput.gasUsed[PublicKernelType.TEARDOWN] ?? Gas.empty();

      return {
        totalGas: privateGasUsed.add(publicGasUsed).mul(1 + pad),
        teardownGas: teardownGas.mul(1 + pad),
      };
    }

    return { totalGas: privateGasUsed.mul(1 + pad), teardownGas: Gas.empty() };
  }

  /**
   * Convert a SimulatedTx class object to a plain JSON object.
   * @returns A plain object with SimulatedTx properties.
   */
  public toJSON() {
    return {
      tx: this.tx.toJSON(),
      privateReturnValues: this.privateReturnValues?.map(fr => fr.toString()),
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
    const privateReturnValues = obj.privateReturnValues?.map(Fr.fromString);

    return new SimulatedTx(tx, privateReturnValues, publicOutput);
  }
}
