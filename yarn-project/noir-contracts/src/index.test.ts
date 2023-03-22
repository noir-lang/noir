import { compileCircuit } from './index.js';

test('it should compile the constructor circuit', () => {
  const { circuit, abi } = compileCircuit('constructor');

  expect(circuit.length).toBeGreaterThan(0);
  expect(abi.parameters.length).toBe(4);
  expect(abi.return_type).toBe(null);
});
