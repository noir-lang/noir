import { type Fr } from '@aztec/circuits.js';

import type { AvmContext } from '../avm_context.js';
import type { AvmExecutionEnvironment } from '../avm_execution_environment.js';
import { Field, type MemoryValue } from '../avm_memory_types.js';
import { Opcode } from '../serialization/instruction_serialization.js';
import { GetterInstruction } from './instruction_impl.js';

abstract class EnvironmentGetterInstruction extends GetterInstruction {
  protected getValue(context: AvmContext): MemoryValue {
    return new Field(this.getEnvironmentValue(context.environment));
  }

  protected abstract getEnvironmentValue(env: AvmExecutionEnvironment): Fr | number | bigint;
}

export class Address extends EnvironmentGetterInstruction {
  static type: string = 'ADDRESS';
  static readonly opcode: Opcode = Opcode.ADDRESS;

  protected getEnvironmentValue(env: AvmExecutionEnvironment) {
    return env.address;
  }
}

export class StorageAddress extends EnvironmentGetterInstruction {
  static type: string = 'STORAGEADDRESS';
  static readonly opcode: Opcode = Opcode.STORAGEADDRESS;

  protected getEnvironmentValue(env: AvmExecutionEnvironment) {
    return env.storageAddress;
  }
}

export class Sender extends EnvironmentGetterInstruction {
  static type: string = 'SENDER';
  static readonly opcode: Opcode = Opcode.SENDER;

  protected getEnvironmentValue(env: AvmExecutionEnvironment) {
    return env.sender;
  }
}

export class FeePerL2Gas extends EnvironmentGetterInstruction {
  static type: string = 'FEEPERL2GAS';
  static readonly opcode: Opcode = Opcode.FEEPERL2GAS;

  protected getEnvironmentValue(env: AvmExecutionEnvironment) {
    return env.feePerL2Gas;
  }
}

export class FeePerDAGas extends EnvironmentGetterInstruction {
  static type: string = 'FEEPERDAGAS';
  static readonly opcode: Opcode = Opcode.FEEPERDAGAS;

  protected getEnvironmentValue(env: AvmExecutionEnvironment) {
    return env.feePerDaGas;
  }
}

export class ChainId extends EnvironmentGetterInstruction {
  static type: string = 'CHAINID';
  static readonly opcode: Opcode = Opcode.CHAINID;

  protected getEnvironmentValue(env: AvmExecutionEnvironment) {
    return env.globals.chainId;
  }
}

export class Version extends EnvironmentGetterInstruction {
  static type: string = 'VERSION';
  static readonly opcode: Opcode = Opcode.VERSION;

  protected getEnvironmentValue(env: AvmExecutionEnvironment) {
    return env.globals.version;
  }
}

export class BlockNumber extends EnvironmentGetterInstruction {
  static type: string = 'BLOCKNUMBER';
  static readonly opcode: Opcode = Opcode.BLOCKNUMBER;

  protected getEnvironmentValue(env: AvmExecutionEnvironment) {
    return env.globals.blockNumber;
  }
}

export class Timestamp extends EnvironmentGetterInstruction {
  static type: string = 'TIMESTAMP';
  static readonly opcode: Opcode = Opcode.TIMESTAMP;

  protected getEnvironmentValue(env: AvmExecutionEnvironment) {
    return env.globals.timestamp;
  }
}

// export class Coinbase extends EnvironmentGetterInstruction {
//     static type: string = 'COINBASE';
//     static numberOfOperands = 1;

//     constructor(private destOffset: number) {
//         super();
//     }

//     async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
//         const {coinbase} = machineState.executionEnvironment.globals;

//         machineState.memory.set(this.destOffset, coinbase);

//         this.incrementPc(machineState);
//     }
// }

// export class BlockL2GasLimit extends EnvironmentGetterInstruction {
//     static type: string = 'BLOCKL2GASLIMIT';
//     static numberOfOperands = 1;

//     constructor(private destOffset: number) {
//         super();
//     }

//     async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
//         const {blockL2GasLimit} = machineState.executionEnvironment.globals;

//         machineState.memory.set(this.destOffset, blockL2GasLimit);

//         this.incrementPc(machineState);
//     }
// }

// export class BlockDAGasLimit extends EnvironmentGetterInstruction {
//     static type: string = 'BLOCKDAGASLIMIT';
//     static numberOfOperands = 1;

//     constructor(private destOffset: number) {
//         super();
//     }

//     async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
//         const {blockDAGasLimit} = machineState.executionEnvironment.globals;

//         machineState.memory.set(this.destOffset, blockDAGasLimit);

//         this.incrementPc(machineState);
//     }
// }
