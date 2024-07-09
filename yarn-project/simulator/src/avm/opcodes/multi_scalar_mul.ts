import { Fq, Point } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';

import { strict as assert } from 'assert';

import { type AvmContext } from '../avm_context.js';
import { Field, TypeTag } from '../avm_memory_types.js';
import { InstructionExecutionError } from '../errors.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Addressing } from './addressing_mode.js';
import { Instruction } from './instruction.js';

export class MultiScalarMul extends Instruction {
  static type: string = 'MultiScalarMul';
  static readonly opcode: Opcode = Opcode.MSM;

  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8 /* opcode */,
    OperandType.UINT8 /* indirect */,
    OperandType.UINT32 /* points vector offset */,
    OperandType.UINT32 /* scalars vector offset */,
    OperandType.UINT32 /* output offset (fixed triplet) */,
    OperandType.UINT32 /* points length offset */,
  ];

  constructor(
    private indirect: number,
    private pointsOffset: number,
    private scalarsOffset: number,
    private outputOffset: number,
    private pointsLengthOffset: number,
  ) {
    super();
  }

  public async execute(context: AvmContext): Promise<void> {
    const memory = context.machineState.memory.track(this.type);
    // Resolve indirects
    const [pointsOffset, scalarsOffset, outputOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.pointsOffset, this.scalarsOffset, this.outputOffset],
      memory,
    );

    // Length of the points vector should be U32
    memory.checkTag(TypeTag.UINT32, this.pointsLengthOffset);
    // Get the size of the unrolled (x, y , inf) points vector
    const pointsReadLength = memory.get(this.pointsLengthOffset).toNumber();
    assert(pointsReadLength % 3 === 0, 'Points vector offset should be a multiple of 3');
    // Divide by 3 since each point is represented as a triplet to get the number of points
    const numPoints = pointsReadLength / 3;
    // The tag for each triplet will be (Field, Field, Uint8)
    for (let i = 0; i < numPoints; i++) {
      const offset = pointsOffset + i * 3;
      // Check (Field, Field)
      memory.checkTagsRange(TypeTag.FIELD, offset, 2);
      // Check Uint8 (inf flag)
      memory.checkTag(TypeTag.UINT8, offset + 2);
    }
    // Get the unrolled (x, y, inf) representing the points
    const pointsVector = memory.getSlice(pointsOffset, pointsReadLength);

    // The size of the scalars vector is twice the NUMBER of points because of the scalar limb decomposition
    const scalarReadLength = numPoints * 2;
    // Consume gas prior to performing work
    const memoryOperations = {
      reads: 1 + pointsReadLength + scalarReadLength /* points and scalars */,
      writes: 3 /* output triplet */,
      indirect: this.indirect,
    };
    context.machineState.consumeGas(this.gasCost(memoryOperations));
    // Get the unrolled scalar (lo & hi) representing the scalars
    const scalarsVector = memory.getSlice(scalarsOffset, scalarReadLength);
    memory.checkTagsRange(TypeTag.FIELD, scalarsOffset, scalarReadLength);

    // Now we need to reconstruct the points and scalars into something we can operate on.
    const grumpkinPoints: Point[] = [];
    for (let i = 0; i < numPoints; i++) {
      const p: Point = new Point(pointsVector[3 * i].toFr(), pointsVector[3 * i + 1].toFr(), false);
      // Include this later when we have a standard for representing infinity
      // const isInf = pointsVector[i + 2].toBoolean();

      if (!p.isOnGrumpkin()) {
        throw new InstructionExecutionError(`Point ${p.toString()} is not on the curve.`);
      }
      grumpkinPoints.push(p);
    }
    // The scalars are read from memory as Fr elements, which are limbs of Fq elements
    // So we need to reconstruct them before performing the scalar multiplications
    const scalarFqVector: Fq[] = [];
    for (let i = 0; i < numPoints; i++) {
      const scalarLo = scalarsVector[2 * i].toFr();
      const scalarHi = scalarsVector[2 * i + 1].toFr();
      const fqScalar = Fq.fromHighLow(scalarHi, scalarLo);
      scalarFqVector.push(fqScalar);
    }
    // TODO: Is there an efficient MSM implementation in ts that we can replace this by?
    const grumpkin = new Grumpkin();
    // Zip the points and scalars into pairs
    const [firstBaseScalarPair, ...rest]: Array<[Point, Fq]> = grumpkinPoints.map((p, idx) => [p, scalarFqVector[idx]]);
    // Fold the points and scalars into a single point
    // We have to ensure get the first point, since the identity element (point at infinity) isn't quite working in ts
    const outputPoint = rest.reduce(
      (acc, curr) => grumpkin.add(acc, grumpkin.mul(curr[0], curr[1])),
      grumpkin.mul(firstBaseScalarPair[0], firstBaseScalarPair[1]),
    );
    const output = outputPoint.toFields().map(f => new Field(f));

    memory.setSlice(outputOffset, output);

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }
}
