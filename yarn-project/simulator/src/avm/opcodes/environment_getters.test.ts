import { Fr } from '@aztec/foundation/fields';

import { initContext, initExecutionEnvironment, initGlobalVariables } from '../fixtures/index.js';
import {
  Address,
  BlockNumber,
  ChainId,
  FeePerDAGas,
  FeePerL1Gas,
  FeePerL2Gas,
  Portal,
  Sender,
  StorageAddress,
  Timestamp,
  Version,
} from './environment_getters.js';

type EnvInstruction =
  | typeof Portal
  | typeof FeePerL1Gas
  | typeof FeePerL2Gas
  | typeof FeePerDAGas
  | typeof Sender
  | typeof StorageAddress
  | typeof Address;
describe.each([
  [Portal, 'portal'],
  [FeePerL1Gas, 'feePerL1Gas'],
  [FeePerL2Gas, 'feePerL2Gas'],
  [FeePerDAGas, 'feePerDaGas'],
  [Sender, 'sender'],
  [StorageAddress, 'storageAddress'],
  [Address, 'address'],
])('Environment getters instructions', (clsValue: EnvInstruction, key: string) => {
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
    const value = new Fr(123456n);
    const instruction = new clsValue(/*indirect=*/ 0, /*dstOffset=*/ 0);
    const context = initContext({ env: initExecutionEnvironment({ [key]: value }) });

    await instruction.execute(context);

    const actual = context.machineState.memory.get(0).toFr();
    expect(actual).toEqual(value);
  });
});

type GlobalsInstruction = typeof ChainId | typeof Version | typeof BlockNumber | typeof Timestamp;
describe.each([
  [ChainId, 'chainId'],
  [Version, 'version'],
  [BlockNumber, 'blockNumber'],
  [Timestamp, 'timestamp'],
])('Global Variables', (clsValue: GlobalsInstruction, key: string) => {
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
    const value = new Fr(123456n);
    const instruction = new clsValue(/*indirect=*/ 0, /*dstOffset=*/ 0);
    const globals = initGlobalVariables({ [key]: value });
    const context = initContext({ env: initExecutionEnvironment({ globals }) });

    await instruction.execute(context);

    const actual = context.machineState.memory.get(0).toFr();
    expect(actual).toEqual(value);
  });
});
