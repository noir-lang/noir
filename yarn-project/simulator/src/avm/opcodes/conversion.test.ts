import { type AvmContext } from '../avm_context.js';
import { Field, type Uint8, Uint32 } from '../avm_memory_types.js';
import { initContext } from '../fixtures/index.js';
import { Addressing, AddressingMode } from './addressing_mode.js';
import { ToRadixLE } from './conversion.js';

describe('Conversion Opcodes', () => {
  let context: AvmContext;

  beforeEach(async () => {
    context = initContext();
  });

  describe('To Radix LE', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        ToRadixLE.opcode, // opcode
        1, // indirect
        ...Buffer.from('12345678', 'hex'), // inputStateOffset
        ...Buffer.from('23456789', 'hex'), // outputStateOffset
        ...Buffer.from('00000002', 'hex'), // radix
        ...Buffer.from('00000100', 'hex'), // numLimbs
      ]);
      const inst = new ToRadixLE(
        /*indirect=*/ 1,
        /*srcOffset=*/ 0x12345678,
        /*dstOffset=*/ 0x23456789,
        /*radix=*/ 2,
        /*numLimbs=*/ 256,
      );

      expect(ToRadixLE.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should decompose correctly - direct', async () => {
      const arg = new Field(0b1011101010100n);
      const indirect = 0;
      const srcOffset = 0;
      const radix = 2; // Bit decomposition
      const numLimbs = 10; // only the first 10 bits
      const dstOffset = 20;
      context.machineState.memory.set(srcOffset, arg);

      await new ToRadixLE(indirect, srcOffset, dstOffset, radix, numLimbs).execute(context);

      const resultBuffer: Buffer = Buffer.concat(
        context.machineState.memory.getSliceAs<Uint8>(dstOffset, numLimbs).map(byte => byte.toBuffer()),
      );
      // The expected result is the first 10 bits of the input, reversed
      const expectedResults = '1011101010100'.split('').reverse().slice(0, numLimbs).map(Number);
      for (let i = 0; i < numLimbs; i++) {
        expect(resultBuffer.readUInt8(i)).toEqual(expectedResults[i]);
      }
    });

    it('Should decompose correctly - indirect', async () => {
      const arg = new Field(Buffer.from('1234567890abcdef', 'hex'));
      const indirect = new Addressing([
        /*srcOffset=*/ AddressingMode.INDIRECT,
        /*dstOffset*/ AddressingMode.INDIRECT,
      ]).toWire();
      const srcOffset = 0;
      const srcOffsetReal = 10;
      const dstOffset = 2;
      const dstOffsetReal = 30;
      context.machineState.memory.set(srcOffset, new Uint32(srcOffsetReal));
      context.machineState.memory.set(dstOffset, new Uint32(dstOffsetReal));
      context.machineState.memory.set(srcOffsetReal, arg);

      const radix = 1 << 8; // Byte decomposition
      const numLimbs = 32; // 256-bit decomposition
      await new ToRadixLE(indirect, srcOffset, dstOffset, radix, numLimbs).execute(context);

      const resultBuffer: Buffer = Buffer.concat(
        context.machineState.memory.getSliceAs<Uint8>(dstOffsetReal, numLimbs).map(byte => byte.toBuffer()),
      );
      // The expected result is the input (padded to 256 bits),and reversed
      const expectedResults = '1234567890abcdef'
        .padStart(64, '0')
        .split('')
        .reverse()
        .map(a => parseInt(a, 16));
      // Checking the value in each byte of the buffer is correct
      for (let i = 0; i < numLimbs; i++) {
        expect(resultBuffer.readUInt8(i)).toEqual(expectedResults[2 * i] + expectedResults[2 * i + 1] * 16);
      }
    });
  });
});
