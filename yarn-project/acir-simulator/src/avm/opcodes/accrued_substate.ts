import { AvmMachineState } from '../avm_machine_state.js';
import { AvmJournal } from '../journal/journal.js';
import { Instruction } from './instruction.js';
import { StaticCallStorageAlterError } from './storage.js';

export class EmitNoteHash extends Instruction {
  static type: string = 'EMITNOTEHASH';
  static numberOfOperands = 1;

  constructor(private noteHashOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, journal: AvmJournal): Promise<void> {
    if (machineState.executionEnvironment.isStaticCall) {
      throw new StaticCallStorageAlterError();
    }

    const noteHash = machineState.memory.get(this.noteHashOffset).toFr();
    journal.writeNoteHash(noteHash);

    this.incrementPc(machineState);
  }
}

export class EmitNullifier extends Instruction {
  static type: string = 'EMITNULLIFIER';
  static numberOfOperands = 1;

  constructor(private nullifierOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, journal: AvmJournal): Promise<void> {
    if (machineState.executionEnvironment.isStaticCall) {
      throw new StaticCallStorageAlterError();
    }

    const nullifier = machineState.memory.get(this.nullifierOffset).toFr();
    journal.writeNullifier(nullifier);

    this.incrementPc(machineState);
  }
}

export class EmitUnencryptedLog extends Instruction {
  static type: string = 'EMITUNENCRYPTEDLOG';
  static numberOfOperands = 2;

  constructor(private logOffset: number, private logSize: number) {
    super();
  }

  async execute(machineState: AvmMachineState, journal: AvmJournal): Promise<void> {
    if (machineState.executionEnvironment.isStaticCall) {
      throw new StaticCallStorageAlterError();
    }

    const log = machineState.memory.getSlice(this.logOffset, this.logSize).map(f => f.toFr());
    journal.writeLog(log);

    this.incrementPc(machineState);
  }
}

export class SendL2ToL1Message extends Instruction {
  static type: string = 'EMITUNENCRYPTEDLOG';
  static numberOfOperands = 2;

  constructor(private msgOffset: number, private msgSize: number) {
    super();
  }

  async execute(machineState: AvmMachineState, journal: AvmJournal): Promise<void> {
    if (machineState.executionEnvironment.isStaticCall) {
      throw new StaticCallStorageAlterError();
    }

    const msg = machineState.memory.getSlice(this.msgOffset, this.msgSize).map(f => f.toFr());
    journal.writeLog(msg);

    this.incrementPc(machineState);
  }
}
