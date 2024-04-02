import { type AvmContext } from '../avm_context.js';
import { TypeTag, Uint16, Uint32 } from '../avm_memory_types.js';
import { initContext } from '../fixtures/index.js';
import { And, Not, Or, Shl, Shr, Xor } from './bitwise.js';

describe('Bitwise instructions', () => {
  let context: AvmContext;

  beforeEach(() => {
    context = initContext();
  });

  describe('AND', () => {
    it('Should deserialize correctly', () => {
      const buf = Buffer.from([
        And.opcode, // opcode
        0x01, // indirect
        TypeTag.UINT64, // inTag
        ...Buffer.from('12345678', 'hex'), // aOffset
        ...Buffer.from('23456789', 'hex'), // bOffset
        ...Buffer.from('3456789a', 'hex'), // dstOffset
      ]);
      const inst = new And(
        /*indirect=*/ 0x01,
        /*inTag=*/ TypeTag.UINT64,
        /*aOffset=*/ 0x12345678,
        /*bOffset=*/ 0x23456789,
        /*dstOffset=*/ 0x3456789a,
      );

      expect(And.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should AND correctly over integral types', async () => {
      context.machineState.memory.set(0, new Uint32(0b11111110010011100100n));
      context.machineState.memory.set(1, new Uint32(0b11100100111001001111n));

      await new And(
        /*indirect=*/ 0,
        /*inTag=*/ TypeTag.UINT32,
        /*aOffset=*/ 0,
        /*bOffset=*/ 1,
        /*dstOffset=*/ 2,
      ).execute(context);

      const actual = context.machineState.memory.get(2);
      expect(actual).toEqual(new Uint32(0b11100100010001000100n));
    });
  });

  describe('OR', () => {
    it('Should deserialize correctly', () => {
      const buf = Buffer.from([
        Or.opcode, // opcode
        0x01, // indirect
        TypeTag.UINT64, // inTag
        ...Buffer.from('12345678', 'hex'), // aOffset
        ...Buffer.from('23456789', 'hex'), // bOffset
        ...Buffer.from('3456789a', 'hex'), // dstOffset
      ]);
      const inst = new Or(
        /*indirect=*/ 0x01,
        /*inTag=*/ TypeTag.UINT64,
        /*aOffset=*/ 0x12345678,
        /*bOffset=*/ 0x23456789,
        /*dstOffset=*/ 0x3456789a,
      );

      expect(Or.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should OR correctly over integral types', async () => {
      const a = new Uint32(0b11111110010011100100n);
      const b = new Uint32(0b11100100111001001111n);

      context.machineState.memory.set(0, a);
      context.machineState.memory.set(1, b);

      await new Or(
        /*indirect=*/ 0,
        /*inTag=*/ TypeTag.UINT32,
        /*aOffset=*/ 0,
        /*bOffset=*/ 1,
        /*dstOffset=*/ 2,
      ).execute(context);

      const expected = new Uint32(0b11111110111011101111n);
      const actual = context.machineState.memory.get(2);
      expect(actual).toEqual(expected);
    });
  });

  describe('XOR', () => {
    it('Should deserialize correctly', () => {
      const buf = Buffer.from([
        Xor.opcode, // opcode
        0x01, // indirect
        TypeTag.UINT64, // inTag
        ...Buffer.from('12345678', 'hex'), // aOffset
        ...Buffer.from('23456789', 'hex'), // bOffset
        ...Buffer.from('3456789a', 'hex'), // dstOffset
      ]);
      const inst = new Xor(
        /*indirect=*/ 0x01,
        /*inTag=*/ TypeTag.UINT64,
        /*aOffset=*/ 0x12345678,
        /*bOffset=*/ 0x23456789,
        /*dstOffset=*/ 0x3456789a,
      );

      expect(Xor.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should XOR correctly over integral types', async () => {
      const a = new Uint32(0b11111110010011100100n);
      const b = new Uint32(0b11100100111001001111n);

      context.machineState.memory.set(0, a);
      context.machineState.memory.set(1, b);

      await new Xor(
        /*indirect=*/ 0,
        /*inTag=*/ TypeTag.UINT32,
        /*aOffset=*/ 0,
        /*bOffset=*/ 1,
        /*dstOffset=*/ 2,
      ).execute(context);

      const expected = new Uint32(0b00011010101010101011n);
      const actual = context.machineState.memory.get(2);
      expect(actual).toEqual(expected);
    });
  });

  describe('SHR', () => {
    it('Should deserialize correctly', () => {
      const buf = Buffer.from([
        Shr.opcode, // opcode
        0x01, // indirect
        TypeTag.UINT64, // inTag
        ...Buffer.from('12345678', 'hex'), // aOffset
        ...Buffer.from('23456789', 'hex'), // bOffset
        ...Buffer.from('3456789a', 'hex'), // dstOffset
      ]);
      const inst = new Shr(
        /*indirect=*/ 0x01,
        /*inTag=*/ TypeTag.UINT64,
        /*aOffset=*/ 0x12345678,
        /*bOffset=*/ 0x23456789,
        /*dstOffset=*/ 0x3456789a,
      );

      expect(Shr.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should shift correctly 0 positions over integral types', async () => {
      const a = new Uint32(0b11111110010011100100n);
      const b = new Uint32(0n);

      context.machineState.memory.set(0, a);
      context.machineState.memory.set(1, b);

      await new Shr(
        /*indirect=*/ 0,
        /*inTag=*/ TypeTag.UINT32,
        /*aOffset=*/ 0,
        /*bOffset=*/ 1,
        /*dstOffset=*/ 2,
      ).execute(context);

      const expected = a;
      const actual = context.machineState.memory.get(2);
      expect(actual).toEqual(expected);
    });

    it('Should shift correctly 2 positions over integral types', async () => {
      const a = new Uint32(0b11111110010011100100n);
      const b = new Uint32(2n);

      context.machineState.memory.set(0, a);
      context.machineState.memory.set(1, b);

      await new Shr(
        /*indirect=*/ 0,
        /*inTag=*/ TypeTag.UINT32,
        /*aOffset=*/ 0,
        /*bOffset=*/ 1,
        /*dstOffset=*/ 2,
      ).execute(context);

      const expected = new Uint32(0b00111111100100111001n);
      const actual = context.machineState.memory.get(2);
      expect(actual).toEqual(expected);
    });

    it('Should shift correctly 19 positions over integral types', async () => {
      const a = new Uint32(0b11111110010011100100n);
      const b = new Uint32(19n);

      context.machineState.memory.set(0, a);
      context.machineState.memory.set(1, b);

      await new Shr(
        /*indirect=*/ 0,
        /*inTag=*/ TypeTag.UINT32,
        /*aOffset=*/ 0,
        /*bOffset=*/ 1,
        /*dstOffset=*/ 2,
      ).execute(context);

      const expected = new Uint32(0b01n);
      const actual = context.machineState.memory.get(2);
      expect(actual).toEqual(expected);
    });
  });

  describe('SHL', () => {
    it('Should deserialize correctly', () => {
      const buf = Buffer.from([
        Shl.opcode, // opcode
        0x01, // indirect
        TypeTag.UINT64, // inTag
        ...Buffer.from('12345678', 'hex'), // aOffset
        ...Buffer.from('23456789', 'hex'), // bOffset
        ...Buffer.from('3456789a', 'hex'), // dstOffset
      ]);
      const inst = new Shl(
        /*indirect=*/ 0x01,
        /*inTag=*/ TypeTag.UINT64,
        /*aOffset=*/ 0x12345678,
        /*bOffset=*/ 0x23456789,
        /*dstOffset=*/ 0x3456789a,
      );

      expect(Shl.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should shift correctly 0 positions over integral types', async () => {
      const a = new Uint32(0b11111110010011100100n);
      const b = new Uint32(0n);

      context.machineState.memory.set(0, a);
      context.machineState.memory.set(1, b);

      await new Shl(
        /*indirect=*/ 0,
        /*inTag=*/ TypeTag.UINT32,
        /*aOffset=*/ 0,
        /*bOffset=*/ 1,
        /*dstOffset=*/ 2,
      ).execute(context);

      const expected = a;
      const actual = context.machineState.memory.get(2);
      expect(actual).toEqual(expected);
    });

    it('Should shift correctly 2 positions over integral types', async () => {
      const a = new Uint32(0b11111110010011100100n);
      const b = new Uint32(2n);

      context.machineState.memory.set(0, a);
      context.machineState.memory.set(1, b);

      await new Shl(
        /*indirect=*/ 0,
        /*inTag=*/ TypeTag.UINT32,
        /*aOffset=*/ 0,
        /*bOffset=*/ 1,
        /*dstOffset=*/ 2,
      ).execute(context);

      const expected = new Uint32(0b1111111001001110010000n);
      const actual = context.machineState.memory.get(2);
      expect(actual).toEqual(expected);
    });

    it('Should shift correctly over bit limit over integral types', async () => {
      const a = new Uint16(0b1110010011100111n);
      const b = new Uint16(17n);

      context.machineState.memory.set(0, a);
      context.machineState.memory.set(1, b);

      await new Shl(
        /*indirect=*/ 0,
        /*inTag=*/ TypeTag.UINT16,
        /*aOffset=*/ 0,
        /*bOffset=*/ 1,
        /*dstOffset=*/ 2,
      ).execute(context);

      const expected = new Uint16(0n);
      const actual = context.machineState.memory.get(2);
      expect(actual).toEqual(expected);
    });

    it('Should truncate when shifting over bit size over integral types', async () => {
      const a = new Uint16(0b1110010011100111n);
      const b = new Uint16(2n);

      context.machineState.memory.set(0, a);
      context.machineState.memory.set(1, b);

      await new Shl(
        /*indirect=*/ 0,
        /*inTag=*/ TypeTag.UINT16,
        /*aOffset=*/ 0,
        /*bOffset=*/ 1,
        /*dstOffset=*/ 2,
      ).execute(context);

      const expected = new Uint16(0b1001001110011100n);
      const actual = context.machineState.memory.get(2);
      expect(actual).toEqual(expected);
    });
  });

  describe('NOT', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        Not.opcode, // opcode
        0x01, // indirect
        TypeTag.UINT64, // inTag
        ...Buffer.from('12345678', 'hex'), // aOffset
        ...Buffer.from('3456789a', 'hex'), // dstOffset
      ]);
      const inst = new Not(
        /*indirect=*/ 0x01,
        /*inTag=*/ TypeTag.UINT64,
        /*aOffset=*/ 0x12345678,
        /*dstOffset=*/ 0x3456789a,
      );

      expect(Not.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should NOT correctly over integral types', async () => {
      const a = new Uint16(0b0110010011100100n);

      context.machineState.memory.set(0, a);

      await new Not(/*indirect=*/ 0, /*inTag=*/ TypeTag.UINT16, /*aOffset=*/ 0, /*dstOffset=*/ 1).execute(context);

      const expected = new Uint16(0b1001101100011011n); // high bits!
      const actual = context.machineState.memory.get(1);
      expect(actual).toEqual(expected);
    });
  });
});
