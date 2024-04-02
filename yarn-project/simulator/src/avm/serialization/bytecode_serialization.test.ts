import { strict as assert } from 'assert';

import { Add, Address, Call, StaticCall, Sub } from '../opcodes/index.js';
import { type BufferCursor } from './buffer_cursor.js';
import { type InstructionSet, decodeFromBytecode, encodeToBytecode } from './bytecode_serialization.js';
import { type Opcode } from './instruction_serialization.js';

class InstA {
  constructor(private n: number) {}
  static readonly opcode: number = 1;

  // Expects opcode.
  public static deserialize(buf: BufferCursor): InstA {
    const opcode: number = buf.readUint8();
    assert(opcode == InstA.opcode);
    return new InstA(buf.readUint16BE());
  }

  // Includes opcode.
  public serialize(): Buffer {
    const buf = Buffer.alloc(1 + 2);
    buf.writeUint8(InstA.opcode);
    buf.writeUint16BE(this.n, 1);
    return buf;
  }
}

class InstB {
  constructor(private n: bigint) {}
  static readonly opcode: number = 2;

  // Expects opcode.
  public static deserialize(buf: BufferCursor): InstB {
    const opcode: number = buf.readUint8();
    assert(opcode == InstB.opcode);
    return new InstB(buf.readBigInt64BE());
  }

  // Includes opcode.
  public serialize(): Buffer {
    const buf = Buffer.alloc(1 + 8);
    buf.writeUint8(InstB.opcode);
    buf.writeBigInt64BE(this.n, 1);
    return buf;
  }
}

describe('Bytecode Serialization', () => {
  it('Should deserialize using instruction set', () => {
    const instructionSet: InstructionSet = new Map<Opcode, any>([
      [InstA.opcode, InstA],
      [InstB.opcode, InstB],
    ]);
    const a = new InstA(0x1234);
    const b = new InstB(0x5678n);
    const bytecode = Buffer.concat([a.serialize(), b.serialize()]);

    const actual = decodeFromBytecode(bytecode, instructionSet);

    expect(actual).toEqual([a, b]);
  });

  it('Should serialize using instruction.serialize()', () => {
    const a = new InstA(1234);
    const b = new InstB(5678n);

    const actual = encodeToBytecode([a, b]);

    const expected = Buffer.concat([a.serialize(), b.serialize()]);
    expect(actual).toEqual(expected);
  });

  it('Should deserialize real instructions', () => {
    const instructions = [
      new Add(/*indirect=*/ 0, /*inTag=*/ 0, /*aOffset=*/ 0, /*bOffset=*/ 1, /*dstOffset=*/ 2),
      new Sub(/*indirect=*/ 0, /*inTag=*/ 0, /*aOffset=*/ 0, /*bOffset=*/ 1, /*dstOffset=*/ 2),
      new Address(/*indirect=*/ 0, /*dstOffset=*/ 1),
      new Call(
        /*indirect=*/ 0x01,
        /*gasOffset=*/ 0x12345678,
        /*addrOffset=*/ 0xa2345678,
        /*argsOffset=*/ 0xb2345678,
        /*argsSize=*/ 0xc2345678,
        /*retOffset=*/ 0xd2345678,
        /*retSize=*/ 0xe2345678,
        /*successOffset=*/ 0xf2345678,
        /*temporaryFunctionSelectorOffset=*/ 0xf3345678,
      ),
      new StaticCall(
        /*indirect=*/ 0x01,
        /*gasOffset=*/ 0x12345678,
        /*addrOffset=*/ 0xa2345678,
        /*argsOffset=*/ 0xb2345678,
        /*argsSize=*/ 0xc2345678,
        /*retOffset=*/ 0xd2345678,
        /*retSize=*/ 0xe2345678,
        /*successOffset=*/ 0xf2345678,
        /*temporaryFunctionSelectorOffset=*/ 0xf3345678,
      ),
    ];
    const bytecode = Buffer.concat(instructions.map(i => i.serialize()));

    const actual = decodeFromBytecode(bytecode);

    expect(actual).toEqual(instructions);
  });

  it('Should serialize real instructions', () => {
    const instructions = [
      new Add(/*indirect=*/ 0, /*inTag=*/ 0, /*aOffset=*/ 0, /*bOffset=*/ 1, /*dstOffset=*/ 2),
      new Sub(/*indirect=*/ 0, /*inTag=*/ 0, /*aOffset=*/ 0, /*bOffset=*/ 1, /*dstOffset=*/ 2),
      new Address(/*indirect=*/ 0, /*dstOffset=*/ 1),
      new Call(
        /*indirect=*/ 0x01,
        /*gasOffset=*/ 0x12345678,
        /*addrOffset=*/ 0xa2345678,
        /*argsOffset=*/ 0xb2345678,
        /*argsSize=*/ 0xc2345678,
        /*retOffset=*/ 0xd2345678,
        /*retSize=*/ 0xe2345678,
        /*successOffset=*/ 0xf2345678,
        /*temporaryFunctionSelectorOffset=*/ 0xf3345678,
      ),
      new StaticCall(
        /*indirect=*/ 0x01,
        /*gasOffset=*/ 0x12345678,
        /*addrOffset=*/ 0xa2345678,
        /*argsOffset=*/ 0xb2345678,
        /*argsSize=*/ 0xc2345678,
        /*retOffset=*/ 0xd2345678,
        /*retSize=*/ 0xe2345678,
        /*successOffset=*/ 0xf2345678,
        /*temporaryFunctionSelectorOffset=*/ 0xf3345678,
      ),
    ];

    const actual = encodeToBytecode(instructions);

    const expected = Buffer.concat(instructions.map(i => i.serialize()));
    expect(actual).toEqual(expected);
  });
});
