import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { Point } from '@aztec/foundation/fields';

import { type AvmContext } from '../avm_context.js';
import { Field } from '../avm_memory_types.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Addressing } from './addressing_mode.js';
import { Instruction } from './instruction.js';

export class EcAdd extends Instruction {
  static type: string = 'ECADD';
  static readonly opcode = Opcode.ECADD;

  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8, // reserved
    OperandType.UINT8, // indirect
    OperandType.UINT32, // p1X
    OperandType.UINT32, // p1Y
    OperandType.UINT32, // p1IsInfinite
    OperandType.UINT32, // p2X
    OperandType.UINT32, // p2Y
    OperandType.UINT32, // p2IsInfinite
    OperandType.UINT32, // dst
  ];

  constructor(
    private indirect: number,
    private p1XOffset: number,
    private p1YOffset: number,
    private p1IsInfiniteOffset: number,
    private p2XOffset: number,
    private p2YOffset: number,
    private p2IsInfiniteOffset: number,
    private dstOffset: number,
  ) {
    super();
  }

  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { reads: 6, writes: 3, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    const [p1XOffset, p1YOffset, p1IsInfiniteOffset, p2XOffset, p2YOffset, p2IsInfiniteOffset, dstOffset] =
      Addressing.fromWire(this.indirect).resolve(
        [
          this.p1XOffset,
          this.p1YOffset,
          this.p1IsInfiniteOffset,
          this.p2XOffset,
          this.p2YOffset,
          this.p2IsInfiniteOffset,
          this.dstOffset,
        ],
        memory,
      );

    const p1X = memory.get(p1XOffset);
    const p1Y = memory.get(p1YOffset);
    const p1IsInfinite = memory.get(p1IsInfiniteOffset).toNumber() === 1;
    const p1 = new Point(p1X.toFr(), p1Y.toFr(), p1IsInfinite);
    if (!p1.isOnGrumpkin()) {
      throw new Error(`Point1 is not on the curve`);
    }

    const p2X = memory.get(p2XOffset);
    const p2Y = memory.get(p2YOffset);
    // unused. Point doesn't store this information
    const p2IsInfinite = memory.get(p2IsInfiniteOffset).toNumber() === 1;
    const p2 = new Point(p2X.toFr(), p2Y.toFr(), p2IsInfinite);
    if (!p2.isOnGrumpkin()) {
      throw new Error(`Point1 is not on the curve`);
    }

    const grumpkin = new Grumpkin();
    let dest = grumpkin.add(p1, p2);
    // Temporary,
    if (p1IsInfinite) {
      dest = p2;
    } else if (p2IsInfinite) {
      dest = p1;
    }
    memory.set(dstOffset, new Field(dest.x));
    memory.set(dstOffset + 1, new Field(dest.y));
    // Check representation of infinity for grumpkin
    memory.set(dstOffset + 2, new Field(dest.equals(Point.ZERO) ? 1 : 0));

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }
}
