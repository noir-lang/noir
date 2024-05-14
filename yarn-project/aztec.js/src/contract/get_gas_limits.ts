import { PublicKernelType, type SimulatedTx } from '@aztec/circuit-types';
import { Gas } from '@aztec/circuits.js';

/**
 * Returns suggested total and teardown gas limits for a simulated tx.
 * Note that public gas usage is only accounted for if the publicOutput is present.
 * @param pad - Percentage to pad the suggested gas limits by, defaults to 10%.
 */
export function getGasLimits(simulatedTx: SimulatedTx, pad = 0.1) {
  const privateGasUsed = simulatedTx.tx.data.publicInputs.end.gasUsed;
  if (simulatedTx.publicOutput) {
    const publicGasUsed = Object.values(simulatedTx.publicOutput.gasUsed)
      .filter(Boolean)
      .reduce((total, current) => total.add(current), Gas.empty());
    const teardownGas = simulatedTx.publicOutput.gasUsed[PublicKernelType.TEARDOWN] ?? Gas.empty();

    return {
      totalGas: privateGasUsed.add(publicGasUsed).mul(1 + pad),
      teardownGas: teardownGas.mul(1 + pad),
    };
  }

  return { totalGas: privateGasUsed.mul(1 + pad), teardownGas: Gas.empty() };
}
