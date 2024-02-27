import type { AvmContext } from '../avm_context.js';
import { Uint8 } from '../avm_memory_types.js';
import { InstructionExecutionError } from '../errors.js';
import { NullifierCollisionError } from '../journal/nullifiers.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Instruction } from './instruction.js';
import { StaticCallStorageAlterError } from './storage.js';

export class EmitNoteHash extends Instruction {
  static type: string = 'EMITNOTEHASH';
  static readonly opcode: Opcode = Opcode.EMITNOTEHASH;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat = [OperandType.UINT8, OperandType.UINT8, OperandType.UINT32];

  constructor(private indirect: number, private noteHashOffset: number) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    if (context.environment.isStaticCall) {
      throw new StaticCallStorageAlterError();
    }

    const noteHash = context.machineState.memory.get(this.noteHashOffset).toFr();
    context.persistableState.writeNoteHash(noteHash);

    context.machineState.incrementPc();
  }
}

export class NullifierExists extends Instruction {
  static type: string = 'NULLIFIEREXISTS';
  static readonly opcode: Opcode = Opcode.NULLIFIEREXISTS;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat = [OperandType.UINT8, OperandType.UINT8, OperandType.UINT32, OperandType.UINT32];

  constructor(private indirect: number, private nullifierOffset: number, private existsOffset: number) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    const nullifier = context.machineState.memory.get(this.nullifierOffset).toFr();
    const exists = await context.persistableState.checkNullifierExists(context.environment.storageAddress, nullifier);

    context.machineState.memory.set(this.existsOffset, exists ? new Uint8(1) : new Uint8(0));

    context.machineState.incrementPc();
  }
}

export class EmitNullifier extends Instruction {
  static type: string = 'EMITNULLIFIER';
  static readonly opcode: Opcode = Opcode.EMITNULLIFIER;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat = [OperandType.UINT8, OperandType.UINT8, OperandType.UINT32];

  constructor(private indirect: number, private nullifierOffset: number) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    if (context.environment.isStaticCall) {
      throw new StaticCallStorageAlterError();
    }

    const nullifier = context.machineState.memory.get(this.nullifierOffset).toFr();
    try {
      await context.persistableState.writeNullifier(context.environment.storageAddress, nullifier);
    } catch (e) {
      if (e instanceof NullifierCollisionError) {
        // Error is known/expected, raise as InstructionExecutionError that the will lead the simulator to revert this call
        throw new InstructionExecutionError(
          `Attempted to emit duplicate nullifier ${nullifier} (storage address: ${context.environment.storageAddress}).`,
        );
      } else {
        throw e;
      }
    }

    context.machineState.incrementPc();
  }
}

export class EmitUnencryptedLog extends Instruction {
  static type: string = 'EMITUNENCRYPTEDLOG';
  static readonly opcode: Opcode = Opcode.EMITUNENCRYPTEDLOG;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat = [OperandType.UINT8, OperandType.UINT8, OperandType.UINT32, OperandType.UINT32];

  constructor(private indirect: number, private logOffset: number, private logSize: number) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    if (context.environment.isStaticCall) {
      throw new StaticCallStorageAlterError();
    }

    const log = context.machineState.memory.getSlice(this.logOffset, this.logSize).map(f => f.toFr());
    context.persistableState.writeLog(log);

    context.machineState.incrementPc();
  }
}

export class SendL2ToL1Message extends Instruction {
  static type: string = 'SENDL2TOL1MSG';
  static readonly opcode: Opcode = Opcode.SENDL2TOL1MSG;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat = [OperandType.UINT8, OperandType.UINT8, OperandType.UINT32, OperandType.UINT32];

  constructor(private indirect: number, private msgOffset: number, private msgSize: number) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    if (context.environment.isStaticCall) {
      throw new StaticCallStorageAlterError();
    }

    const msg = context.machineState.memory.getSlice(this.msgOffset, this.msgSize).map(f => f.toFr());
    context.persistableState.writeL1Message(msg);

    context.machineState.incrementPc();
  }
}
