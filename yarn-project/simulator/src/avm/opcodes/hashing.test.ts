import { keccak, pedersenHash, poseidonHash, sha256 } from '@aztec/foundation/crypto';

import { type AvmContext } from '../avm_context.js';
import { Field, Uint32 } from '../avm_memory_types.js';
import { initContext } from '../fixtures/index.js';
import { Addressing, AddressingMode } from './addressing_mode.js';
import { Keccak, Pedersen, Poseidon2, Sha256 } from './hashing.js';

describe('Hashing Opcodes', () => {
  let context: AvmContext;

  beforeEach(async () => {
    context = initContext();
  });

  describe('Poseidon2', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        Poseidon2.opcode, // opcode
        1, // indirect
        ...Buffer.from('12345678', 'hex'), // dstOffset
        ...Buffer.from('23456789', 'hex'), // messageOffset
        ...Buffer.from('3456789a', 'hex'), // hashSize
      ]);
      const inst = new Poseidon2(
        /*indirect=*/ 1,
        /*dstOffset=*/ 0x12345678,
        /*messageOffset=*/ 0x23456789,
        /*hashSize=*/ 0x3456789a,
      );

      expect(Poseidon2.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should hash correctly - direct', async () => {
      const indirect = 0;
      const args = [new Field(1n), new Field(2n), new Field(3n)];
      const messageOffset = 0;
      context.machineState.memory.setSlice(messageOffset, args);

      const dstOffset = 3;

      const expectedHash = poseidonHash(args.map(field => field.toBuffer()));
      await new Poseidon2(indirect, dstOffset, messageOffset, args.length).execute(context);

      const result = context.machineState.memory.get(dstOffset);
      expect(result).toEqual(new Field(expectedHash));
    });

    it('Should hash correctly - indirect', async () => {
      const args = [new Field(1n), new Field(2n), new Field(3n)];
      const indirect = new Addressing([
        /*dstOffset=*/ AddressingMode.DIRECT,
        /*messageOffset*/ AddressingMode.INDIRECT,
      ]).toWire();
      const messageOffset = 0;
      const realLocation = 4;

      context.machineState.memory.set(messageOffset, new Uint32(realLocation));
      context.machineState.memory.setSlice(realLocation, args);

      const dstOffset = 3;

      const expectedHash = poseidonHash(args.map(field => field.toBuffer()));
      await new Poseidon2(indirect, dstOffset, messageOffset, args.length).execute(context);

      const result = context.machineState.memory.get(dstOffset);
      expect(result).toEqual(new Field(expectedHash));
    });
  });

  describe('Keccak', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        Keccak.opcode, // opcode
        1, // indirect
        ...Buffer.from('12345678', 'hex'), // dstOffset
        ...Buffer.from('23456789', 'hex'), // messageOffset
        ...Buffer.from('3456789a', 'hex'), // hashSize
      ]);
      const inst = new Keccak(
        /*indirect=*/ 1,
        /*dstOffset=*/ 0x12345678,
        /*messageOffset=*/ 0x23456789,
        /*hashSize=*/ 0x3456789a,
      );

      expect(Keccak.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should hash correctly - direct', async () => {
      const args = [new Field(1n), new Field(2n), new Field(3n)];
      const indirect = 0;
      const messageOffset = 0;
      context.machineState.memory.setSlice(messageOffset, args);

      const dstOffset = 3;

      const inputBuffer = Buffer.concat(args.map(field => field.toBuffer()));
      const expectedHash = keccak(inputBuffer);
      await new Keccak(indirect, dstOffset, messageOffset, args.length).execute(context);

      const result = context.machineState.memory.getSliceAs<Field>(dstOffset, 2);
      const combined = Buffer.concat([result[0].toBuffer().subarray(16, 32), result[1].toBuffer().subarray(16, 32)]);

      expect(combined).toEqual(expectedHash);
    });

    it('Should hash correctly - indirect', async () => {
      const args = [new Field(1n), new Field(2n), new Field(3n)];
      const indirect = new Addressing([
        /*dstOffset=*/ AddressingMode.INDIRECT,
        /*messageOffset*/ AddressingMode.INDIRECT,
      ]).toWire();
      const messageOffset = 0;
      const argsLocation = 4;

      const dstOffset = 2;
      const readLocation = 6;

      context.machineState.memory.set(messageOffset, new Uint32(argsLocation));
      context.machineState.memory.set(dstOffset, new Uint32(readLocation));
      context.machineState.memory.setSlice(argsLocation, args);

      const inputBuffer = Buffer.concat(args.map(field => field.toBuffer()));
      const expectedHash = keccak(inputBuffer);
      await new Keccak(indirect, dstOffset, messageOffset, args.length).execute(context);

      const result = context.machineState.memory.getSliceAs<Field>(readLocation, 2);
      const combined = Buffer.concat([result[0].toBuffer().subarray(16, 32), result[1].toBuffer().subarray(16, 32)]);

      expect(combined).toEqual(expectedHash);
    });
  });

  describe('Sha256', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        Sha256.opcode, // opcode
        1, // indirect
        ...Buffer.from('12345678', 'hex'), // dstOffset
        ...Buffer.from('23456789', 'hex'), // messageOffset
        ...Buffer.from('3456789a', 'hex'), // hashSize
      ]);
      const inst = new Sha256(
        /*indirect=*/ 1,
        /*dstOffset=*/ 0x12345678,
        /*messageOffset=*/ 0x23456789,
        /*hashSize=*/ 0x3456789a,
      );

      expect(Sha256.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should hash correctly - direct', async () => {
      const args = [new Field(1n), new Field(2n), new Field(3n)];
      const messageOffset = 0;
      const indirect = 0;
      context.machineState.memory.setSlice(messageOffset, args);

      const dstOffset = 3;

      const inputBuffer = Buffer.concat(args.map(field => field.toBuffer()));
      const expectedHash = sha256(inputBuffer);
      await new Sha256(indirect, dstOffset, messageOffset, args.length).execute(context);

      const result = context.machineState.memory.getSliceAs<Field>(dstOffset, 2);
      const combined = Buffer.concat([result[0].toBuffer().subarray(16, 32), result[1].toBuffer().subarray(16, 32)]);

      expect(combined).toEqual(expectedHash);
    });

    it('Should hash correctly - indirect', async () => {
      const args = [new Field(1n), new Field(2n), new Field(3n)];
      const indirect = new Addressing([
        /*dstOffset=*/ AddressingMode.INDIRECT,
        /*messageOffset*/ AddressingMode.INDIRECT,
      ]).toWire();
      const messageOffset = 0;
      const argsLocation = 4;

      const dstOffset = 2;
      const readLocation = 6;

      context.machineState.memory.set(messageOffset, new Uint32(argsLocation));
      context.machineState.memory.set(dstOffset, new Uint32(readLocation));
      context.machineState.memory.setSlice(argsLocation, args);

      const inputBuffer = Buffer.concat(args.map(field => field.toBuffer()));
      const expectedHash = sha256(inputBuffer);
      await new Sha256(indirect, dstOffset, messageOffset, args.length).execute(context);

      const result = context.machineState.memory.getSliceAs<Field>(readLocation, 2);
      const combined = Buffer.concat([result[0].toBuffer().subarray(16, 32), result[1].toBuffer().subarray(16, 32)]);

      expect(combined).toEqual(expectedHash);
    });
  });

  describe('Pedersen', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        Pedersen.opcode, // opcode
        1, // indirect
        ...Buffer.from('02345678', 'hex'), // genIndexOffset
        ...Buffer.from('12345678', 'hex'), // dstOffset
        ...Buffer.from('23456789', 'hex'), // messageOffset
        ...Buffer.from('3456789a', 'hex'), // hashSize
      ]);
      const inst = new Pedersen(
        /*indirect=*/ 1,
        /*genIndexOffset=*/ 0x02345678,
        /*dstOffset=*/ 0x12345678,
        /*messageOffset=*/ 0x23456789,
        /*hashSizeOffset=*/ 0x3456789a,
      );

      expect(Pedersen.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should hash correctly - direct', async () => {
      const args = [new Field(1n), new Field(2n), new Field(3n)];
      const messageOffset = 0;
      const sizeOffset = 10;
      const genIndexOffset = 20;
      const indirect = 0;
      const genIndex = 20;

      context.machineState.memory.setSlice(messageOffset, args);
      context.machineState.memory.set(sizeOffset, new Uint32(args.length));
      context.machineState.memory.set(genIndexOffset, new Uint32(genIndex));

      const dstOffset = 3;

      const expectedHash = pedersenHash(args, genIndex);
      await new Pedersen(indirect, genIndexOffset, dstOffset, messageOffset, sizeOffset).execute(context);

      const result = context.machineState.memory.get(dstOffset);
      expect(result).toEqual(new Field(expectedHash));
    });

    it('Should hash correctly - indirect', async () => {
      const args = [new Field(1n), new Field(2n), new Field(3n)];
      const indirect = new Addressing([
        /*genIndexOffset=*/ AddressingMode.DIRECT,
        /*dstOffset=*/ AddressingMode.DIRECT,
        /*messageOffset*/ AddressingMode.INDIRECT,
        /*messageSizeOffset*/ AddressingMode.INDIRECT,
      ]).toWire();
      const messageOffset = 0;
      const sizeOffset = 10;
      const realLocation = 4;
      const realSizeLocation = 20;
      const genIndexOffset = 50;

      context.machineState.memory.set(messageOffset, new Uint32(realLocation));
      context.machineState.memory.set(sizeOffset, new Uint32(realSizeLocation));
      context.machineState.memory.setSlice(realLocation, args);
      context.machineState.memory.set(realSizeLocation, new Uint32(args.length));
      context.machineState.memory.set(genIndexOffset, new Uint32(0));

      const dstOffset = 300;

      const expectedHash = pedersenHash(args);
      await new Pedersen(indirect, genIndexOffset, dstOffset, messageOffset, sizeOffset).execute(context);

      const result = context.machineState.memory.get(dstOffset);
      expect(result).toEqual(new Field(expectedHash));
    });
  });
});
