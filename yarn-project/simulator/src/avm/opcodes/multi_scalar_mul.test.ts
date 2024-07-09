import { Fq, Fr } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';

import { type AvmContext } from '../avm_context.js';
import { Field, type MemoryValue, Uint8, Uint32 } from '../avm_memory_types.js';
import { initContext } from '../fixtures/index.js';
import { MultiScalarMul } from './multi_scalar_mul.js';

describe('MultiScalarMul Opcode', () => {
  let context: AvmContext;

  beforeEach(async () => {
    context = initContext();
  });
  it('Should (de)serialize correctly', () => {
    const buf = Buffer.from([
      MultiScalarMul.opcode, // opcode
      7, // indirect
      ...Buffer.from('12345678', 'hex'), // pointsOffset
      ...Buffer.from('23456789', 'hex'), // scalars Offset
      ...Buffer.from('3456789a', 'hex'), // outputOffset
      ...Buffer.from('456789ab', 'hex'), // pointsLengthOffset
    ]);
    const inst = new MultiScalarMul(
      /*indirect=*/ 7,
      /*pointsOffset=*/ 0x12345678,
      /*scalarsOffset=*/ 0x23456789,
      /*outputOffset=*/ 0x3456789a,
      /*pointsLengthOffset=*/ 0x456789ab,
    );

    expect(MultiScalarMul.deserialize(buf)).toEqual(inst);
    expect(inst.serialize()).toEqual(buf);
  });

  it('Should perform msm correctly - direct', async () => {
    const indirect = 0;
    const grumpkin = new Grumpkin();
    // We need to ensure points are actually on curve, so we just use the generator
    // In future we could use a random point, for now we create an array of [G, 2G, 3G]
    const points = Array.from({ length: 3 }, (_, i) => grumpkin.mul(grumpkin.generator(), new Fq(i + 1)));

    // Pick some big scalars to test the edge cases
    const scalars = [new Fq(Fq.MODULUS - 1n), new Fq(Fq.MODULUS - 2n), new Fq(1n)];
    const pointsReadLength = points.length * 3; // multiplied by 3 since we will store them as triplet in avm memory
    const scalarsLength = scalars.length * 2; // multiplied by 2 since we will store them as lo and hi limbs in avm memory
    // Transform the points and scalars into the format that we will write to memory
    // We just store the x and y coordinates here, and handle the infinities when we write to memory
    const storedScalars: Field[] = scalars.flatMap(s => [new Field(s.lo), new Field(s.hi)]);
    // Points are stored as [x1, y1, inf1, x2, y2, inf2, ...] where the types are [Field, Field, Uint8, Field, Field, Uint8, ...]
    const storedPoints: MemoryValue[] = points
      .map(p => p.toFields())
      .flatMap(([x, y, inf]) => [new Field(x), new Field(y), new Uint8(inf.toNumber())]);
    const pointsOffset = 0;
    context.machineState.memory.setSlice(pointsOffset, storedPoints);
    // Store scalars
    const scalarsOffset = pointsOffset + pointsReadLength;
    context.machineState.memory.setSlice(scalarsOffset, storedScalars);
    // Store length of points to read
    const pointsLengthOffset = scalarsOffset + scalarsLength;
    context.machineState.memory.set(pointsLengthOffset, new Uint32(pointsReadLength));
    const outputOffset = pointsLengthOffset + 1;

    await new MultiScalarMul(indirect, pointsOffset, scalarsOffset, outputOffset, pointsLengthOffset).execute(context);

    const result = context.machineState.memory.getSlice(outputOffset, 3).map(r => r.toFr());

    // We write it out explicitly here
    let expectedResult = grumpkin.mul(points[0], scalars[0]);
    expectedResult = grumpkin.add(expectedResult, grumpkin.mul(points[1], scalars[1]));
    expectedResult = grumpkin.add(expectedResult, grumpkin.mul(points[2], scalars[2]));

    expect(result).toEqual([expectedResult.x, expectedResult.y, new Fr(0n)]);
  });

  it('Should perform msm correctly - indirect', async () => {
    const indirect = 7;
    const grumpkin = new Grumpkin();
    // We need to ensure points are actually on curve, so we just use the generator
    // In future we could use a random point, for now we create an array of [G, 2G, 3G]
    const points = Array.from({ length: 3 }, (_, i) => grumpkin.mul(grumpkin.generator(), new Fq(i + 1)));

    // Pick some big scalars to test the edge cases
    const scalars = [new Fq(Fq.MODULUS - 1n), new Fq(Fq.MODULUS - 2n), new Fq(1n)];
    const pointsReadLength = points.length * 3; // multiplied by 3 since we will store them as triplet in avm memory
    const scalarsLength = scalars.length * 2; // multiplied by 2 since we will store them as lo and hi limbs in avm memory
    // Transform the points and scalars into the format that we will write to memory
    // We just store the x and y coordinates here, and handle the infinities when we write to memory
    const storedScalars: Field[] = scalars.flatMap(s => [new Field(s.lo), new Field(s.hi)]);
    // Points are stored as [x1, y1, inf1, x2, y2, inf2, ...] where the types are [Field, Field, Uint8, Field, Field, Uint8, ...]
    const storedPoints: MemoryValue[] = points
      .map(p => p.toFields())
      .flatMap(([x, y, inf]) => [new Field(x), new Field(y), new Uint8(inf.toNumber())]);
    const pointsOffset = 0;
    context.machineState.memory.setSlice(pointsOffset, storedPoints);
    // Store scalars
    const scalarsOffset = pointsOffset + pointsReadLength;
    context.machineState.memory.setSlice(scalarsOffset, storedScalars);
    // Store length of points to read
    const pointsLengthOffset = scalarsOffset + scalarsLength;
    context.machineState.memory.set(pointsLengthOffset, new Uint32(pointsReadLength));
    const outputOffset = pointsLengthOffset + 1;

    // Set up the indirect pointers
    const pointsIndirectOffset = outputOffset + 3; /* 3 since the output is a triplet */
    const scalarsIndirectOffset = pointsIndirectOffset + 1;
    const outputIndirectOffset = scalarsIndirectOffset + 1;

    context.machineState.memory.set(pointsIndirectOffset, new Uint32(pointsOffset));
    context.machineState.memory.set(scalarsIndirectOffset, new Uint32(scalarsOffset));
    context.machineState.memory.set(outputIndirectOffset, new Uint32(outputOffset));

    await new MultiScalarMul(
      indirect,
      pointsIndirectOffset,
      scalarsIndirectOffset,
      outputIndirectOffset,
      pointsLengthOffset,
    ).execute(context);

    const result = context.machineState.memory.getSlice(outputOffset, 3).map(r => r.toFr());

    // We write it out explicitly here
    let expectedResult = grumpkin.mul(points[0], scalars[0]);
    expectedResult = grumpkin.add(expectedResult, grumpkin.mul(points[1], scalars[1]));
    expectedResult = grumpkin.add(expectedResult, grumpkin.mul(points[2], scalars[2]));

    expect(result).toEqual([expectedResult.x, expectedResult.y, new Fr(0n)]);
  });
});
