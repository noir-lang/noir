import type { AvmContext } from '../avm_context.js';
import { Uint8 } from '../avm_memory_types.js';
import { InstructionExecutionError } from '../errors.js';
import { NullifierCollisionError } from '../journal/nullifiers.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Addressing } from './addressing_mode.js';
import { Instruction } from './instruction.js';
import { StaticCallStorageAlterError } from './storage.js';

export class NoteHashExists extends Instruction {
  static type: string = 'NOTEHASHEXISTS';
  static readonly opcode: Opcode = Opcode.NOTEHASHEXISTS;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(
    private indirect: number,
    private noteHashOffset: number,
    private leafIndexOffset: number,
    private existsOffset: number,
  ) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    // Note that this instruction accepts any type in memory, and converts to Field.
    const noteHash = context.machineState.memory.get(this.noteHashOffset).toFr();
    const leafIndex = context.machineState.memory.get(this.leafIndexOffset).toFr();

    const exists = await context.persistableState.checkNoteHashExists(
      context.environment.storageAddress,
      noteHash,
      leafIndex,
    );
    context.machineState.memory.set(this.existsOffset, exists ? new Uint8(1) : new Uint8(0));

    context.machineState.incrementPc();
  }
}

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

export class L1ToL2MessageExists extends Instruction {
  static type: string = 'L1TOL2MSGEXISTS';
  static readonly opcode: Opcode = Opcode.L1TOL2MSGEXISTS;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(
    private indirect: number,
    private msgHashOffset: number,
    private msgLeafIndexOffset: number,
    private existsOffset: number,
  ) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    const msgHash = context.machineState.memory.get(this.msgHashOffset).toFr();
    const msgLeafIndex = context.machineState.memory.get(this.msgLeafIndexOffset).toFr();
    const exists = await context.persistableState.checkL1ToL2MessageExists(msgHash, msgLeafIndex);
    context.machineState.memory.set(this.existsOffset, exists ? new Uint8(1) : new Uint8(0));

    context.machineState.incrementPc();
  }
}

export class EmitUnencryptedLog extends Instruction {
  static type: string = 'EMITUNENCRYPTEDLOG';
  static readonly opcode: Opcode = Opcode.EMITUNENCRYPTEDLOG;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(
    private indirect: number,
    private eventSelectorOffset: number,
    private logOffset: number,
    private logSize: number,
  ) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    if (context.environment.isStaticCall) {
      throw new StaticCallStorageAlterError();
    }

    const [eventSelectorOffset, logOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.eventSelectorOffset, this.logOffset],
      context.machineState.memory,
    );

    const contractAddress = context.environment.address;
    const event = context.machineState.memory.get(eventSelectorOffset).toFr();
    const log = context.machineState.memory.getSlice(logOffset, this.logSize).map(f => f.toFr());
    context.persistableState.writeLog(contractAddress, event, log);

    context.machineState.incrementPc();
  }
}

export class SendL2ToL1Message extends Instruction {
  static type: string = 'SENDL2TOL1MSG';
  static readonly opcode: Opcode = Opcode.SENDL2TOL1MSG;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat = [OperandType.UINT8, OperandType.UINT8, OperandType.UINT32, OperandType.UINT32];

  constructor(private indirect: number, private recipientOffset: number, private contentOffset: number) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    if (context.environment.isStaticCall) {
      throw new StaticCallStorageAlterError();
    }

    const recipient = context.machineState.memory.get(this.recipientOffset).toFr();
    const content = context.machineState.memory.get(this.contentOffset).toFr();
    context.persistableState.writeL1Message(recipient, content);

    context.machineState.incrementPc();
  }
}
