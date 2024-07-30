import { pedersenCommit } from '@aztec/foundation/crypto';

import { type AvmContext } from '../avm_context.js';
import { Field, Uint32 } from '../avm_memory_types.js';
import { initContext, randomMemoryFields } from '../fixtures/index.js';
import { Addressing, AddressingMode } from './addressing_mode.js';
import { PedersenCommitment } from './commitment.js';

describe('Commitment Opcode', () => {
  let context: AvmContext;

  beforeEach(async () => {
    context = initContext();
  });

  describe('Pedersen Commitment', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        PedersenCommitment.opcode, // opcode
        1, // indirect
        ...Buffer.from('23456789', 'hex'), // inputOffset
        ...Buffer.from('3456789a', 'hex'), // inputSizeOffset
        ...Buffer.from('12345678', 'hex'), // outputOffset
        ...Buffer.from('00000000', 'hex'), // genIndexOffset
      ]);
      const inst = new PedersenCommitment(
        /*indirect=*/ 1,
        /*inputOffset=*/ 0x23456789,
        /*inputSizeOffset=*/ 0x3456789a,
        /*outputOffset=*/ 0x12345678,
        /*genIndexOffset=*/ 0,
      );

      expect(PedersenCommitment.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should commit correctly - direct', async () => {
      const args = randomMemoryFields(10);
      const inputOffset = 0;
      const inputSizeOffset = 20;
      const outputOffset = 50;
      const indirect = 0;
      const generatorIndexOffset = 10;

      context.machineState.memory.setSlice(inputOffset, args);
      context.machineState.memory.set(inputSizeOffset, new Uint32(args.length));
      context.machineState.memory.set(generatorIndexOffset, new Uint32(0));

      const expectedCommitment = pedersenCommit(args.map(f => f.toBuffer())).map(f => new Field(f));
      await new PedersenCommitment(indirect, inputOffset, outputOffset, inputSizeOffset, generatorIndexOffset).execute(
        context,
      );

      const result = context.machineState.memory.getSlice(outputOffset, 2);
      expect(result).toEqual(expectedCommitment);
      // Check Inf
      expect(0).toEqual(context.machineState.memory.get(outputOffset + 2).toNumber());
    });

    it('Should commit correctly - indirect', async () => {
      const args = randomMemoryFields(10);
      const indirect = new Addressing([
        /*inputOffset=*/ AddressingMode.INDIRECT,
        /*outputOffset*/ AddressingMode.INDIRECT,
        /*inputSizeOffset=*/ AddressingMode.DIRECT,
        /*generatorIndexOffset=*/ AddressingMode.DIRECT,
      ]).toWire();
      const inputOffset = 0;
      const inputSizeOffset = 20;
      const outputOffset = 50;
      const realOutputOffset = 100;
      const realInputOffset = 200;
      const generatorIndexOffset = 51;

      context.machineState.memory.set(outputOffset, new Uint32(realOutputOffset));
      context.machineState.memory.set(inputOffset, new Uint32(realInputOffset));
      context.machineState.memory.setSlice(realInputOffset, args);
      context.machineState.memory.set(inputSizeOffset, new Uint32(args.length));
      context.machineState.memory.set(generatorIndexOffset, new Uint32(0));

      const expectedCommitment = pedersenCommit(args.map(f => f.toBuffer())).map(f => new Field(f));
      await new PedersenCommitment(indirect, inputOffset, outputOffset, inputSizeOffset, generatorIndexOffset).execute(
        context,
      );

      const result = context.machineState.memory.getSlice(realOutputOffset, 2);
      expect(result).toEqual(expectedCommitment);
      // Check Inf
      expect(0).toEqual(context.machineState.memory.get(realOutputOffset + 2).toNumber());
    });
  });
});
