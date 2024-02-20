import { toBigIntBE } from '@aztec/foundation/bigint-buffer';
import { keccak, pedersenHash, poseidonHash, sha256 } from '@aztec/foundation/crypto';

import { AvmContext } from '../avm_context.js';
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
    private hashOffset: number,
    private hashSize: number,
  ) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    // We hash a set of field elements
    const [hashOffset] = Addressing.fromWire(this.indirect).resolve([this.hashOffset], context.machineState.memory);

    // Memory pointer will be indirect
    const hashData = context.machineState.memory.getSlice(hashOffset, this.hashSize).map(word => word.toBuffer());

    const hash = poseidonHash(hashData);
    context.machineState.memory.set(this.dstOffset, new Field(hash));

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
    private hashOffset: number,
    private hashSize: number,
  ) {
    super();
  }

  // Note hash output is 32 bytes, so takes up two fields
  async execute(context: AvmContext): Promise<void> {
    // We hash a set of field elements
    const [hashOffset, dstOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.hashOffset, this.dstOffset],
      context.machineState.memory,
    );

    const hashData = context.machineState.memory.getSlice(hashOffset, this.hashSize).map(word => word.toBuffer());

    const hash = keccak(Buffer.concat(hashData));

    // Split output into two fields
    const high = new Field(toBigIntBE(hash.subarray(0, 16)));
    const low = new Field(toBigIntBE(hash.subarray(16, 32)));

    context.machineState.memory.set(dstOffset, high);
    context.machineState.memory.set(dstOffset + 1, low);

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
    private hashOffset: number,
    private hashSize: number,
  ) {
    super();
  }

  // Note hash output is 32 bytes, so takes up two fields
  async execute(context: AvmContext): Promise<void> {
    const [hashOffset, dstOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.hashOffset, this.dstOffset],
      context.machineState.memory,
    );

    // We hash a set of field elements
    const hashData = context.machineState.memory.getSlice(hashOffset, this.hashSize).map(word => word.toBuffer());

    const hash = sha256(Buffer.concat(hashData));

    // Split output into two fields
    const high = new Field(toBigIntBE(hash.subarray(0, 16)));
    const low = new Field(toBigIntBE(hash.subarray(16, 32)));

    context.machineState.memory.set(dstOffset, high);
    context.machineState.memory.set(dstOffset + 1, low);

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
  ];

  constructor(
    private indirect: number,
    private dstOffset: number,
    private hashOffset: number,
    private hashSize: number,
  ) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    const [hashOffset] = Addressing.fromWire(this.indirect).resolve([this.hashOffset], context.machineState.memory);

    // We hash a set of field elements
    const hashData = context.machineState.memory.getSlice(hashOffset, this.hashSize).map(word => word.toBuffer());

    // No domain sep for now
    const hash = pedersenHash(hashData);
    context.machineState.memory.set(this.dstOffset, new Field(hash));

    context.machineState.incrementPc();
  }
}
