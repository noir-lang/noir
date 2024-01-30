import { AvmMachineState } from '../avm_machine_state.js';
import { Field } from '../avm_memory_types.js';
import { AvmJournal } from '../journal/journal.js';
import { Instruction } from './instruction.js';

export class Address extends Instruction {
  static type: string = 'ADDRESS';
  static numberOfOperands = 1;

  constructor(private destOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const { address } = machineState.executionEnvironment;

    machineState.memory.set(this.destOffset, new Field(address));
    this.incrementPc(machineState);
  }
}

export class StorageAddress extends Instruction {
  static type: string = 'STORAGEADDRESS';
  static numberOfOperands = 1;

  constructor(private destOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const { storageAddress } = machineState.executionEnvironment;

    machineState.memory.set(this.destOffset, new Field(storageAddress));
    this.incrementPc(machineState);
  }
}

export class Sender extends Instruction {
  static type: string = 'SENDER';
  static numberOfOperands = 1;

  constructor(private destOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const { sender } = machineState.executionEnvironment;

    machineState.memory.set(this.destOffset, new Field(sender));

    this.incrementPc(machineState);
  }
}

export class Origin extends Instruction {
  static type: string = 'ORIGIN';
  static numberOfOperands = 1;

  constructor(private destOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const { origin } = machineState.executionEnvironment;

    machineState.memory.set(this.destOffset, new Field(origin));

    this.incrementPc(machineState);
  }
}

export class FeePerL1Gas extends Instruction {
  static type: string = 'FEEPERL1GAS';
  static numberOfOperands = 1;

  constructor(private destOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const { feePerL1Gas } = machineState.executionEnvironment;

    machineState.memory.set(this.destOffset, new Field(feePerL1Gas));

    this.incrementPc(machineState);
  }
}

export class FeePerL2Gas extends Instruction {
  static type: string = 'FEEPERL2GAS';
  static numberOfOperands = 1;

  constructor(private destOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const { feePerL2Gas } = machineState.executionEnvironment;

    machineState.memory.set(this.destOffset, new Field(feePerL2Gas));

    this.incrementPc(machineState);
  }
}

export class FeePerDAGas extends Instruction {
  static type: string = 'FEEPERDAGAS';
  static numberOfOperands = 1;

  constructor(private destOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const { feePerDaGas } = machineState.executionEnvironment;

    machineState.memory.set(this.destOffset, new Field(feePerDaGas));

    this.incrementPc(machineState);
  }
}

export class Portal extends Instruction {
  static type: string = 'PORTAL';
  static numberOfOperands = 1;

  constructor(private destOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const { portal } = machineState.executionEnvironment;

    machineState.memory.set(this.destOffset, new Field(portal.toField()));

    this.incrementPc(machineState);
  }
}

export class ChainId extends Instruction {
  static type: string = 'CHAINID';
  static numberOfOperands = 1;

  constructor(private destOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const { chainId } = machineState.executionEnvironment.globals;

    machineState.memory.set(this.destOffset, new Field(chainId));

    this.incrementPc(machineState);
  }
}

export class Version extends Instruction {
  static type: string = 'VERSION';
  static numberOfOperands = 1;

  constructor(private destOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const { version } = machineState.executionEnvironment.globals;

    machineState.memory.set(this.destOffset, new Field(version));

    this.incrementPc(machineState);
  }
}

export class BlockNumber extends Instruction {
  static type: string = 'BLOCKNUMBER';
  static numberOfOperands = 1;

  constructor(private destOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const { blockNumber } = machineState.executionEnvironment.globals;

    machineState.memory.set(this.destOffset, new Field(blockNumber));

    this.incrementPc(machineState);
  }
}

export class Timestamp extends Instruction {
  static type: string = 'TIMESTAMP';
  static numberOfOperands = 1;

  constructor(private destOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const { timestamp } = machineState.executionEnvironment.globals;

    machineState.memory.set(this.destOffset, new Field(timestamp));

    this.incrementPc(machineState);
  }
}

// export class Coinbase extends Instruction {
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
// export class BlockL1GasLimit extends Instruction {
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

// export class BlockL2GasLimit extends Instruction {
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

// export class BlockDAGasLimit extends Instruction {
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
