import type { AvmContext } from '../avm_context.js';
import type { AvmExecutionEnvironment } from '../avm_execution_environment.js';
import { Field, type MemoryValue, Uint64 } from '../avm_memory_types.js';
import { Opcode } from '../serialization/instruction_serialization.js';
import { GetterInstruction } from './instruction_impl.js';

abstract class EnvironmentGetterInstruction extends GetterInstruction {
  protected getValue(context: AvmContext): MemoryValue {
    return this.getEnvironmentValue(context.environment);
  }

  protected abstract getEnvironmentValue(env: AvmExecutionEnvironment): MemoryValue;
}

export class Address extends EnvironmentGetterInstruction {
  static type: string = 'ADDRESS';
  static readonly opcode: Opcode = Opcode.ADDRESS;

  protected getEnvironmentValue(env: AvmExecutionEnvironment) {
    return new Field(env.address.toField());
  }
}

export class StorageAddress extends EnvironmentGetterInstruction {
  static type: string = 'STORAGEADDRESS';
  static readonly opcode: Opcode = Opcode.STORAGEADDRESS;

  protected getEnvironmentValue(env: AvmExecutionEnvironment) {
    return new Field(env.storageAddress.toField());
  }
}

export class Sender extends EnvironmentGetterInstruction {
  static type: string = 'SENDER';
  static readonly opcode: Opcode = Opcode.SENDER;

  protected getEnvironmentValue(env: AvmExecutionEnvironment) {
    return new Field(env.sender.toField());
  }
}

export class FeePerL2Gas extends EnvironmentGetterInstruction {
  static type: string = 'FEEPERL2GAS';
  static readonly opcode: Opcode = Opcode.FEEPERL2GAS;

  protected getEnvironmentValue(env: AvmExecutionEnvironment) {
    return new Field(env.feePerL2Gas);
  }
}

export class FeePerDAGas extends EnvironmentGetterInstruction {
  static type: string = 'FEEPERDAGAS';
  static readonly opcode: Opcode = Opcode.FEEPERDAGAS;

  protected getEnvironmentValue(env: AvmExecutionEnvironment) {
    return new Field(env.feePerDaGas);
  }
}

export class TransactionFee extends EnvironmentGetterInstruction {
  static type: string = 'TRANSACTIONFEE';
  static readonly opcode: Opcode = Opcode.TRANSACTIONFEE;

  protected getEnvironmentValue(env: AvmExecutionEnvironment) {
    return new Field(env.transactionFee);
  }
}

export class ChainId extends EnvironmentGetterInstruction {
  static type: string = 'CHAINID';
  static readonly opcode: Opcode = Opcode.CHAINID;

  protected getEnvironmentValue(env: AvmExecutionEnvironment) {
    return new Field(env.globals.chainId);
  }
}

export class Version extends EnvironmentGetterInstruction {
  static type: string = 'VERSION';
  static readonly opcode: Opcode = Opcode.VERSION;

  protected getEnvironmentValue(env: AvmExecutionEnvironment) {
    return new Field(env.globals.version);
  }
}

export class BlockNumber extends EnvironmentGetterInstruction {
  static type: string = 'BLOCKNUMBER';
  static readonly opcode: Opcode = Opcode.BLOCKNUMBER;

  protected getEnvironmentValue(env: AvmExecutionEnvironment) {
    return new Field(env.globals.blockNumber);
  }
}

export class Timestamp extends EnvironmentGetterInstruction {
  static type: string = 'TIMESTAMP';
  static readonly opcode: Opcode = Opcode.TIMESTAMP;

  protected getEnvironmentValue(env: AvmExecutionEnvironment) {
    return new Uint64(env.globals.timestamp.toBigInt());
  }
}
