import { Fr } from '@aztec/foundation/fields';

import { TypeTag } from '../avm_memory_types.js';
import { initContext, initExecutionEnvironment, initGlobalVariables } from '../fixtures/index.js';
import {
  Address,
  BlockNumber,
  ChainId,
  FeePerDAGas,
  FeePerL2Gas,
  Sender,
  StorageAddress,
  Timestamp,
  TransactionFee,
  Version,
} from './environment_getters.js';

type EnvInstruction =
  | typeof FeePerL2Gas
  | typeof FeePerDAGas
  | typeof Sender
  | typeof StorageAddress
  | typeof Address
  | typeof TransactionFee;

describe.each([
  [FeePerL2Gas, 'feePerL2Gas'],
  [FeePerDAGas, 'feePerDaGas'],
  [Sender, 'sender'],
  [StorageAddress, 'storageAddress'],
  [Address, 'address'],
  [TransactionFee, 'transactionFee'],
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

    expect(context.machineState.memory.getTag(0)).toBe(TypeTag.FIELD);
    const actual = context.machineState.memory.get(0).toFr();
    expect(actual).toEqual(value);
  });
});

type GlobalsInstruction = typeof ChainId | typeof Version | typeof BlockNumber | typeof Timestamp;
describe.each([
  [ChainId, 'chainId', TypeTag.FIELD],
  [Version, 'version', TypeTag.FIELD],
  [BlockNumber, 'blockNumber', TypeTag.FIELD],
  [Timestamp, 'timestamp', TypeTag.UINT64],
])('Global Variables', (clsValue: GlobalsInstruction, key: string, tag: TypeTag) => {
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

    expect(context.machineState.memory.getTag(0)).toBe(tag);
    const actual = context.machineState.memory.get(0).toFr();
    expect(actual).toEqual(value);
  });
});
