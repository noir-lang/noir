import { GasFees } from '@aztec/circuits.js';
import { FunctionSelector as FunctionSelectorType } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';

import { randomInt } from 'crypto';

import { type AvmContext } from '../avm_context.js';
import { TypeTag } from '../avm_memory_types.js';
import { initContext, initExecutionEnvironment, initGlobalVariables } from '../fixtures/index.js';
import {
  Address,
  BlockNumber,
  ChainId,
  FeePerDAGas,
  FeePerL2Gas,
  FunctionSelector,
  Sender,
  StorageAddress,
  Timestamp,
  TransactionFee,
  Version,
} from './environment_getters.js';

type GetterInstruction =
  | typeof Sender
  | typeof StorageAddress
  | typeof Address
  | typeof FunctionSelector
  | typeof TransactionFee
  | typeof ChainId
  | typeof Version
  | typeof BlockNumber
  | typeof Timestamp
  | typeof FeePerDAGas
  | typeof FeePerL2Gas;

describe('Environment getters', () => {
  const address = AztecAddress.random();
  const storageAddress = AztecAddress.random();
  const sender = AztecAddress.random();
  const functionSelector = FunctionSelectorType.random();
  const transactionFee = Fr.random();
  const chainId = Fr.random();
  const version = Fr.random();
  const blockNumber = Fr.random();
  const timestamp = new Fr(randomInt(100000)); // cap timestamp since must fit in u64
  const feePerDaGas = Fr.random();
  const feePerL2Gas = Fr.random();
  const gasFees = new GasFees(feePerDaGas, feePerL2Gas);
  const globals = initGlobalVariables({
    chainId,
    version,
    blockNumber,
    timestamp,
    gasFees,
  });
  const env = initExecutionEnvironment({
    address,
    storageAddress,
    sender,
    functionSelector,
    transactionFee,
    globals,
  });
  let context: AvmContext;
  beforeEach(() => {
    context = initContext({ env });
  });

  describe.each([
    [Address, address.toField()],
    [StorageAddress, storageAddress.toField()],
    [Sender, sender.toField()],
    [FunctionSelector, functionSelector.toField(), TypeTag.UINT32],
    [TransactionFee, transactionFee.toField()],
    [ChainId, chainId.toField()],
    [Version, version.toField()],
    [BlockNumber, blockNumber.toField()],
    [Timestamp, timestamp.toField(), TypeTag.UINT64],
    [FeePerDAGas, feePerDaGas.toField()],
    [FeePerL2Gas, feePerL2Gas.toField()],
  ])('Environment getters instructions', (instrClass: GetterInstruction, value: Fr, tag: TypeTag = TypeTag.FIELD) => {
    it(`${instrClass.name} should (de)serialize correctly`, () => {
      const buf = Buffer.from([
        instrClass.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // dstOffset
      ]);
      const instr = new instrClass(/*indirect=*/ 0x01, /*dstOffset=*/ 0x12345678);

      expect(instrClass.deserialize(buf)).toEqual(instr);
      expect(instr.serialize()).toEqual(buf);
    });
    it(`${instrClass.name} should read '${instrClass.type}' correctly`, async () => {
      const instruction = new instrClass(/*indirect=*/ 0, /*dstOffset=*/ 0);

      await instruction.execute(context);

      expect(context.machineState.memory.getTag(0)).toBe(tag);
      const actual = context.machineState.memory.get(0).toFr();
      expect(actual).toEqual(value);
    });
  });
});
