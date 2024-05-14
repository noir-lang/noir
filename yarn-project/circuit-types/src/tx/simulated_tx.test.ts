import { mockSimulatedTx } from '../mocks.js';
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
});
