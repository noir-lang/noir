import { initContext, initMachineState } from '../fixtures/index.js';
import { DAGasLeft, L2GasLeft } from './context_getters.js';

describe.each([
  [L2GasLeft, 'l2GasLeft'],
  [DAGasLeft, 'daGasLeft'],
] as const)('Context getter instructions for machine state', (clsValue, key) => {
  it(`${clsValue.name} should (de)serialize correctly`, () => {
    const buf = Buffer.from([
      clsValue.opcode, // opcode
      0x01, // indirect
      ...Buffer.from('12345678', 'hex'), // dstOffset
    ]);
    const inst = new clsValue(/*indirect=*/ 0x01, /*dstOffset=*/ 0x12345678);

    expect(clsValue.deserialize(buf)).toEqual(inst);
    expect(inst.serialize()).toEqual(buf);
  });

  it(`${clsValue.name} should read '${key}' correctly`, async () => {
    const value = 123456;
    const instruction = new clsValue(/*indirect=*/ 0, /*dstOffset=*/ 0);
    const context = initContext({ machineState: initMachineState({ [key]: value }) });

    await instruction.execute(context);

    const actual = context.machineState.memory.get(0).toNumber();
    const expected = key === 'l2GasLeft' ? value - 110 : value; // l2gascost decreases when it's executed
    expect(actual).toEqual(expected);
  });
});
