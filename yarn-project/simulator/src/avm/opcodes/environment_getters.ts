import { type Fr } from '@aztec/circuits.js';

import type { AvmContext } from '../avm_context.js';
import type { AvmExecutionEnvironment } from '../avm_execution_environment.js';
import { Field } from '../avm_memory_types.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Instruction } from './instruction.js';

abstract class GetterInstruction extends Instruction {
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [OperandType.UINT8, OperandType.UINT8, OperandType.UINT32];

  constructor(protected indirect: number, protected dstOffset: number) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    const res = new Field(this.getIt(context.environment));
    context.machineState.memory.set(this.dstOffset, res);
    context.machineState.incrementPc();
  }

  protected abstract getIt(env: AvmExecutionEnvironment): Fr | number | bigint;
}

export class Address extends GetterInstruction {
  static type: string = 'ADDRESS';
  static readonly opcode: Opcode = Opcode.ADDRESS;

  protected getIt(env: AvmExecutionEnvironment) {
    return env.address;
  }
}

export class StorageAddress extends GetterInstruction {
  static type: string = 'STORAGEADDRESS';
  static readonly opcode: Opcode = Opcode.STORAGEADDRESS;

  protected getIt(env: AvmExecutionEnvironment) {
    return env.storageAddress;
  }
}

export class Sender extends GetterInstruction {
  static type: string = 'SENDER';
  static readonly opcode: Opcode = Opcode.SENDER;

  protected getIt(env: AvmExecutionEnvironment) {
    return env.sender;
  }
}

export class Origin extends GetterInstruction {
  static type: string = 'ORIGIN';
  static readonly opcode: Opcode = Opcode.ORIGIN;

  protected getIt(env: AvmExecutionEnvironment) {
    return env.origin;
  }
}

export class FeePerL1Gas extends GetterInstruction {
  static type: string = 'FEEPERL1GAS';
  static readonly opcode: Opcode = Opcode.FEEPERL1GAS;

  protected getIt(env: AvmExecutionEnvironment) {
    return env.feePerL1Gas;
  }
}

export class FeePerL2Gas extends GetterInstruction {
  static type: string = 'FEEPERL2GAS';
  static readonly opcode: Opcode = Opcode.FEEPERL2GAS;

  protected getIt(env: AvmExecutionEnvironment) {
    return env.feePerL2Gas;
  }
}

export class FeePerDAGas extends GetterInstruction {
  static type: string = 'FEEPERDAGAS';
  static readonly opcode: Opcode = Opcode.FEEPERDAGAS;

  protected getIt(env: AvmExecutionEnvironment) {
    return env.feePerDaGas;
  }
}

export class Portal extends GetterInstruction {
  static type: string = 'PORTAL';
  static readonly opcode: Opcode = Opcode.PORTAL;

  protected getIt(env: AvmExecutionEnvironment) {
    return env.portal.toField();
  }
}

export class ChainId extends GetterInstruction {
  static type: string = 'CHAINID';
  static readonly opcode: Opcode = Opcode.CHAINID;

  protected getIt(env: AvmExecutionEnvironment) {
    return env.globals.chainId;
  }
}

export class Version extends GetterInstruction {
  static type: string = 'VERSION';
  static readonly opcode: Opcode = Opcode.VERSION;

  protected getIt(env: AvmExecutionEnvironment) {
    return env.globals.version;
  }
}

export class BlockNumber extends GetterInstruction {
  static type: string = 'BLOCKNUMBER';
  static readonly opcode: Opcode = Opcode.BLOCKNUMBER;

  protected getIt(env: AvmExecutionEnvironment) {
    return env.globals.blockNumber;
  }
}

export class Timestamp extends GetterInstruction {
  static type: string = 'TIMESTAMP';
  static readonly opcode: Opcode = Opcode.TIMESTAMP;

  protected getIt(env: AvmExecutionEnvironment) {
    return env.globals.timestamp;
  }
}

// export class Coinbase extends GetterInstruction {
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

// // TODO: are these even needed within the block? (both block gas limit variables - why does the execution env care?)
// export class BlockL1GasLimit extends GetterInstruction {
//     static type: string = 'BLOCKL1GASLIMIT';
//     static numberOfOperands = 1;

//     constructor(private destOffset: number) {
//         super();
//     }

//     async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
//         const {blockL1GasLimit} = machineState.executionEnvironment.globals;

//         machineState.memory.set(this.destOffset, blockL1GasLimit);

//         this.incrementPc(machineState);
//     }
// }

// export class BlockL2GasLimit extends GetterInstruction {
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

// export class BlockDAGasLimit extends GetterInstruction {
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
