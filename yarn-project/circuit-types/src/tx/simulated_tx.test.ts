import { Gas } from '@aztec/circuits.js';

import { mockSimulatedTx } from '../mocks.js';
import { PublicKernelType } from './processed_tx.js';
import { SimulatedTx } from './simulated_tx.js';

describe('simulated_tx', () => {
  let simulatedTx: SimulatedTx;

  beforeEach(() => {
    simulatedTx = mockSimulatedTx();
  });

  describe('json', () => {
    it('convert to and from json', () => {
      expect(SimulatedTx.fromJSON(simulatedTx.toJSON())).toEqual(simulatedTx);
    });

    it('convert undefined effects to and from json', () => {
      simulatedTx.privateReturnValues = undefined;
      simulatedTx.publicOutput = undefined;
      expect(SimulatedTx.fromJSON(simulatedTx.toJSON())).toEqual(simulatedTx);
    });
  });

  describe('getGasLimits', () => {
    beforeEach(() => {
      simulatedTx.tx.data.publicInputs.end.gasUsed = Gas.from({ daGas: 100, l2Gas: 200 });
      simulatedTx.publicOutput!.gasUsed = {
        [PublicKernelType.SETUP]: Gas.from({ daGas: 10, l2Gas: 20 }),
        [PublicKernelType.APP_LOGIC]: Gas.from({ daGas: 20, l2Gas: 40 }),
        [PublicKernelType.TEARDOWN]: Gas.from({ daGas: 10, l2Gas: 20 }),
      };
    });

    it('returns gas limits from private gas usage only', () => {
      simulatedTx.publicOutput = undefined;
      // Should be 110 and 220 but oh floating point
      expect(simulatedTx.getGasLimits()).toEqual({
        totalGas: Gas.from({ daGas: 111, l2Gas: 221 }),
        teardownGas: Gas.empty(),
      });
    });

    it('returns gas limits for private and public', () => {
      expect(simulatedTx.getGasLimits()).toEqual({
        totalGas: Gas.from({ daGas: 154, l2Gas: 308 }),
        teardownGas: Gas.from({ daGas: 11, l2Gas: 22 }),
      });
    });

    it('pads gas limits', () => {
      expect(simulatedTx.getGasLimits(1)).toEqual({
        totalGas: Gas.from({ daGas: 280, l2Gas: 560 }),
        teardownGas: Gas.from({ daGas: 20, l2Gas: 40 }),
      });
    });
  });
});
