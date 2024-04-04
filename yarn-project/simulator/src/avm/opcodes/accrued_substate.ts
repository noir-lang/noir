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

  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { reads: 2, writes: 1, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    // Note that this instruction accepts any type in memory, and converts to Field.
    const noteHash = memory.get(this.noteHashOffset).toFr();
    const leafIndex = memory.get(this.leafIndexOffset).toFr();

    const exists = await context.persistableState.checkNoteHashExists(
      context.environment.storageAddress,
      noteHash,
      leafIndex,
    );
    memory.set(this.existsOffset, exists ? new Uint8(1) : new Uint8(0));

    memory.assert(memoryOperations);
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

  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { reads: 1, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    if (context.environment.isStaticCall) {
      throw new StaticCallStorageAlterError();
    }

    const noteHash = memory.get(this.noteHashOffset).toFr();
    context.persistableState.writeNoteHash(noteHash);

    memory.assert(memoryOperations);
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

  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { reads: 1, writes: 1, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    const nullifier = memory.get(this.nullifierOffset).toFr();
    const exists = await context.persistableState.checkNullifierExists(context.environment.storageAddress, nullifier);

    memory.set(this.existsOffset, exists ? new Uint8(1) : new Uint8(0));

    memory.assert(memoryOperations);
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

  public async execute(context: AvmContext): Promise<void> {
    if (context.environment.isStaticCall) {
      throw new StaticCallStorageAlterError();
    }

    const memoryOperations = { reads: 1, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    const nullifier = memory.get(this.nullifierOffset).toFr();
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

    memory.assert(memoryOperations);
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

  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { reads: 2, writes: 1, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    const msgHash = memory.get(this.msgHashOffset).toFr();
    const msgLeafIndex = memory.get(this.msgLeafIndexOffset).toFr();
    const exists = await context.persistableState.checkL1ToL2MessageExists(msgHash, msgLeafIndex);
    memory.set(this.existsOffset, exists ? new Uint8(1) : new Uint8(0));

    memory.assert(memoryOperations);
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

  public async execute(context: AvmContext): Promise<void> {
    if (context.environment.isStaticCall) {
      throw new StaticCallStorageAlterError();
    }

    const memoryOperations = { reads: 1 + this.logSize, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    const [eventSelectorOffset, logOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.eventSelectorOffset, this.logOffset],
      memory,
    );

    const contractAddress = context.environment.address;
    const event = memory.get(eventSelectorOffset).toFr();
    const log = memory.getSlice(logOffset, this.logSize).map(f => f.toFr());
    context.persistableState.writeLog(contractAddress, event, log);

    memory.assert(memoryOperations);
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

  public async execute(context: AvmContext): Promise<void> {
    if (context.environment.isStaticCall) {
      throw new StaticCallStorageAlterError();
    }

    const memoryOperations = { reads: 2, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    const recipient = memory.get(this.recipientOffset).toFr();
    const content = memory.get(this.contentOffset).toFr();
    context.persistableState.writeL1Message(recipient, content);

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }
}
