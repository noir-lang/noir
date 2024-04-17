import { mockSimulatedTx } from '../mocks.js';
import { SimulatedTx } from './simulated_tx.js';

describe('simulated_tx', () => {
  it('convert to and from json', () => {
    const simulatedTx = mockSimulatedTx();
    expect(SimulatedTx.fromJSON(simulatedTx.toJSON())).toEqual(simulatedTx);
  });

  it('convert undefined effects to and from json', () => {
    const simulatedTx = mockSimulatedTx();
    simulatedTx.privateReturnValues = undefined;
    simulatedTx.publicReturnValues = undefined;
    expect(SimulatedTx.fromJSON(simulatedTx.toJSON())).toEqual(simulatedTx);
  });
});
