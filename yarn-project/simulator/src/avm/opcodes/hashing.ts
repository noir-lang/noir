import { toBigIntBE } from '@aztec/foundation/bigint-buffer';
import { keccak, pedersenHash, poseidonHash, sha256 } from '@aztec/foundation/crypto';

import { type AvmContext } from '../avm_context.js';
import { Field } from '../avm_memory_types.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Addressing } from './addressing_mode.js';
import { Instruction } from './instruction.js';

export class Poseidon2 extends Instruction {
  static type: string = 'POSEIDON2';
  static readonly opcode: Opcode = Opcode.POSEIDON;

  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(
    private indirect: number,
    private dstOffset: number,
    private messageOffset: number,
    private messageSize: number,
  ) {
    super();
  }

  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { reads: this.messageSize, writes: 1, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    // We hash a set of field elements
    const [dstOffset, messageOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.dstOffset, this.messageOffset],
      memory,
    );

    // Memory pointer will be indirect
    const hashData = memory.getSlice(messageOffset, this.messageSize).map(word => word.toBuffer());

    const hash = poseidonHash(hashData);
    memory.set(dstOffset, new Field(hash));

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }
}

export class Keccak extends Instruction {
  static type: string = 'KECCAK';
  static readonly opcode: Opcode = Opcode.KECCAK;

  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(
    private indirect: number,
    private dstOffset: number,
    private messageOffset: number,
    private messageSize: number,
  ) {
    super();
  }

  // Note hash output is 32 bytes, so takes up two fields
  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { reads: this.messageSize, writes: 2, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    // We hash a set of field elements
    const [dstOffset, messageOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.dstOffset, this.messageOffset],
      memory,
    );

    const hashData = memory.getSlice(messageOffset, this.messageSize).map(word => word.toBuffer());

    const hash = keccak(Buffer.concat(hashData));

    // Split output into two fields
    const high = new Field(toBigIntBE(hash.subarray(0, 16)));
    const low = new Field(toBigIntBE(hash.subarray(16, 32)));

    memory.set(dstOffset, high);
    memory.set(dstOffset + 1, low);

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }
}

export class Sha256 extends Instruction {
  static type: string = 'SHA256';
  static readonly opcode: Opcode = Opcode.SHA256;

  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(
    private indirect: number,
    private dstOffset: number,
    private messageOffset: number,
    private messageSize: number,
  ) {
    super();
  }

  // Note hash output is 32 bytes, so takes up two fields
  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { reads: this.messageSize, writes: 2, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    const [dstOffset, messageOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.dstOffset, this.messageOffset],
      memory,
    );

    // We hash a set of field elements
    const hashData = memory.getSlice(messageOffset, this.messageSize).map(word => word.toBuffer());

    const hash = sha256(Buffer.concat(hashData));

    // Split output into two fields
    const high = new Field(toBigIntBE(hash.subarray(0, 16)));
    const low = new Field(toBigIntBE(hash.subarray(16, 32)));

    memory.set(dstOffset, high);
    memory.set(dstOffset + 1, low);

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }
}

export class Pedersen extends Instruction {
  static type: string = 'PEDERSEN';
  static readonly opcode: Opcode = Opcode.PEDERSEN;

  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(
    private indirect: number,
    private genIndexOffset: number,
    private dstOffset: number,
    private messageOffset: number,
    private messageSizeOffset: number,
  ) {
    super();
  }

  public async execute(context: AvmContext): Promise<void> {
    const memory = context.machineState.memory.track(this.type);
    const [genIndexOffset, dstOffset, messageOffset, messageSizeOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.genIndexOffset, this.dstOffset, this.messageOffset, this.messageSizeOffset],
      memory,
    );

    // We hash a set of field elements
    const genIndex = Number(memory.get(genIndexOffset).toBigInt());
    const messageSize = Number(memory.get(messageSizeOffset).toBigInt());
    const hashData = memory.getSlice(messageOffset, messageSize);

    const memoryOperations = { reads: messageSize + 2, writes: 1, indirect: this.indirect };
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    // No domain sep for now
    const hash = pedersenHash(hashData, genIndex);
    memory.set(dstOffset, new Field(hash));

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }
}
