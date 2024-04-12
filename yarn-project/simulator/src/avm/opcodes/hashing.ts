import { toBigIntBE } from '@aztec/foundation/bigint-buffer';
import { keccak256, pedersenHash, poseidon2Permutation, sha256 } from '@aztec/foundation/crypto';

import { type AvmContext } from '../avm_context.js';
import { Field, Uint8 } from '../avm_memory_types.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Addressing } from './addressing_mode.js';
import { Instruction } from './instruction.js';

export class Poseidon2 extends Instruction {
  static type: string = 'POSEIDON2';
  static readonly opcode: Opcode = Opcode.POSEIDON2;
  static readonly stateSize = 4;

  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(private indirect: number, private inputStateOffset: number, private outputStateOffset: number) {
    super();
  }

  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { reads: Poseidon2.stateSize, writes: Poseidon2.stateSize, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    const [inputOffset, outputOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.inputStateOffset, this.outputStateOffset],
      memory,
    );

    const inputState = memory.getSlice(inputOffset, Poseidon2.stateSize);
    const outputState = poseidon2Permutation(inputState);
    memory.setSlice(
      outputOffset,
      outputState.map(word => new Field(word)),
    );

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
    private messageSizeOffset: number,
  ) {
    super();
  }

  // pub fn keccak256(input: [u8], message_size: u32) -> [u8; 32]
  public async execute(context: AvmContext): Promise<void> {
    const memory = context.machineState.memory.track(this.type);
    const [dstOffset, messageOffset, messageSizeOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.dstOffset, this.messageOffset, this.messageSizeOffset],
      memory,
    );
    const messageSize = memory.get(messageSizeOffset).toNumber();
    const memoryOperations = { reads: messageSize + 1, writes: 32, indirect: this.indirect };
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    const messageData = Buffer.concat(memory.getSlice(messageOffset, messageSize).map(word => word.toBuffer()));
    const hashBuffer = keccak256(messageData);

    // We need to convert the hashBuffer because map doesn't work as expected on an Uint8Array (Buffer).
    const res = [...hashBuffer].map(byte => new Uint8(byte));
    memory.setSlice(dstOffset, res);

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
