import { PublicKernelType, type SimulatedTx, mockSimulatedTx } from '@aztec/circuit-types';
import { type CombinedAccumulatedData, Gas } from '@aztec/circuits.js';

import { getGasLimits } from './get_gas_limits.js';

describe('getGasLimits', () => {
  let simulatedTx: SimulatedTx;
  let simulationTeardownGasLimits: Gas;

  beforeEach(() => {
    simulatedTx = mockSimulatedTx();

    const data: CombinedAccumulatedData = simulatedTx.tx.data.publicInputs.end as CombinedAccumulatedData;
    data.gasUsed = Gas.from({ daGas: 100, l2Gas: 200 });
    simulatedTx.tx.data.publicInputs.end = data;
    simulatedTx.publicOutput!.gasUsed = {
      [PublicKernelType.SETUP]: Gas.from({ daGas: 10, l2Gas: 20 }),
      [PublicKernelType.APP_LOGIC]: Gas.from({ daGas: 20, l2Gas: 40 }),
      [PublicKernelType.TEARDOWN]: Gas.from({ daGas: 10, l2Gas: 20 }),
    };
    simulationTeardownGasLimits = Gas.empty();
  });

  it('returns gas limits from private gas usage only', () => {
    simulatedTx.publicOutput = undefined;
    // Should be 110 and 220 but oh floating point
    expect(getGasLimits(simulatedTx, simulationTeardownGasLimits)).toEqual({
      totalGas: Gas.from({ daGas: 111, l2Gas: 221 }),
      teardownGas: Gas.empty(),
    });
  });

  it('returns gas limits for private and public', () => {
    expect(getGasLimits(simulatedTx, simulationTeardownGasLimits)).toEqual({
      totalGas: Gas.from({ daGas: 154, l2Gas: 308 }),
      teardownGas: Gas.from({ daGas: 11, l2Gas: 22 }),
    });
  });

  it('pads gas limits', () => {
    expect(getGasLimits(simulatedTx, simulationTeardownGasLimits, 1)).toEqual({
      totalGas: Gas.from({ daGas: 280, l2Gas: 560 }),
      teardownGas: Gas.from({ daGas: 20, l2Gas: 40 }),
    });
  });

  it('subtracts input teardown gas limits', () => {
    simulationTeardownGasLimits = Gas.from({ daGas: 80, l2Gas: 200 });
    expect(getGasLimits(simulatedTx, simulationTeardownGasLimits)).toEqual({
      totalGas: Gas.from({ daGas: 66, l2Gas: 88 }),
      teardownGas: Gas.from({ daGas: 11, l2Gas: 22 }),
    });
  });
});
