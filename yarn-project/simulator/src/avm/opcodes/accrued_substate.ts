import type { AvmContext } from '../avm_context.js';
import { TypeTag, Uint8 } from '../avm_memory_types.js';
import { InstructionExecutionError, StaticCallAlterationError } from '../errors.js';
import { NullifierCollisionError } from '../journal/nullifiers.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Addressing } from './addressing_mode.js';
import { Instruction } from './instruction.js';

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
    const [noteHashOffset, leafIndexOffset, existsOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.noteHashOffset, this.leafIndexOffset, this.existsOffset],
      memory,
    );
    memory.checkTags(TypeTag.FIELD, noteHashOffset, leafIndexOffset);

    // Note that this instruction accepts any type in memory, and converts to Field.
    const noteHash = memory.get(noteHashOffset).toFr();
    const leafIndex = memory.get(leafIndexOffset).toFr();

    const exists = await context.persistableState.checkNoteHashExists(
      context.environment.storageAddress,
      noteHash,
      leafIndex,
    );
    memory.set(existsOffset, exists ? new Uint8(1) : new Uint8(0));

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

    const [noteHashOffset] = Addressing.fromWire(this.indirect).resolve([this.noteHashOffset], memory);
    memory.checkTag(TypeTag.FIELD, noteHashOffset);

    if (context.environment.isStaticCall) {
      throw new StaticCallAlterationError();
    }

    const noteHash = memory.get(noteHashOffset).toFr();
    context.persistableState.writeNoteHash(context.environment.storageAddress, noteHash);

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }
}

export class NullifierExists extends Instruction {
  static type: string = 'NULLIFIEREXISTS';
  static readonly opcode: Opcode = Opcode.NULLIFIEREXISTS;
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
    private nullifierOffset: number,
    private addressOffset: number,
    private existsOffset: number,
  ) {
    super();
  }

  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { reads: 2, writes: 1, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    const [nullifierOffset, addressOffset, existsOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.nullifierOffset, this.addressOffset, this.existsOffset],
      memory,
    );
    memory.checkTags(TypeTag.FIELD, nullifierOffset, addressOffset);

    const nullifier = memory.get(nullifierOffset).toFr();
    const address = memory.get(addressOffset).toFr();
    const exists = await context.persistableState.checkNullifierExists(address, nullifier);

    memory.set(existsOffset, exists ? new Uint8(1) : new Uint8(0));

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
      throw new StaticCallAlterationError();
    }

    const memoryOperations = { reads: 1, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    const [nullifierOffset] = Addressing.fromWire(this.indirect).resolve([this.nullifierOffset], memory);
    memory.checkTag(TypeTag.FIELD, nullifierOffset);

    const nullifier = memory.get(nullifierOffset).toFr();
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

    const [msgHashOffset, msgLeafIndexOffset, existsOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.msgHashOffset, this.msgLeafIndexOffset, this.existsOffset],
      memory,
    );
    memory.checkTags(TypeTag.FIELD, msgHashOffset, msgLeafIndexOffset);

    const msgHash = memory.get(msgHashOffset).toFr();
    const msgLeafIndex = memory.get(msgLeafIndexOffset).toFr();
    const exists = await context.persistableState.checkL1ToL2MessageExists(
      context.environment.address,
      msgHash,
      msgLeafIndex,
    );
    memory.set(existsOffset, exists ? new Uint8(1) : new Uint8(0));

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
    private logSizeOffset: number,
  ) {
    super();
  }

  public async execute(context: AvmContext): Promise<void> {
    if (context.environment.isStaticCall) {
      throw new StaticCallAlterationError();
    }

    const memory = context.machineState.memory.track(this.type);

    const [eventSelectorOffset, logOffset, logSizeOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.eventSelectorOffset, this.logOffset, this.logSizeOffset],
      memory,
    );
    memory.checkTag(TypeTag.FIELD, eventSelectorOffset);
    memory.checkTag(TypeTag.UINT32, logSizeOffset);
    const logSize = memory.get(logSizeOffset).toNumber();
    memory.checkTagsRange(TypeTag.FIELD, logOffset, logSize);

    const contractAddress = context.environment.address;
    const event = memory.get(eventSelectorOffset).toFr();

    const memoryOperations = { reads: 2 + logSize, indirect: this.indirect };
    context.machineState.consumeGas(this.gasCost(memoryOperations));
    const log = memory.getSlice(logOffset, logSize).map(f => f.toFr());
    context.persistableState.writeUnencryptedLog(contractAddress, event, log);

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
      throw new StaticCallAlterationError();
    }

    const memoryOperations = { reads: 2, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    const [recipientOffset, contentOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.recipientOffset, this.contentOffset],
      memory,
    );

    const recipient = memory.get(recipientOffset).toFr();
    const content = memory.get(contentOffset).toFr();
    context.persistableState.writeL2ToL1Message(recipient, content);

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }
}
