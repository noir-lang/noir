import { assert } from '../../../../foundation/src/json-rpc/js_utils.js';
import { type AvmContext } from '../avm_context.js';
import { TypeTag, Uint8 } from '../avm_memory_types.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Addressing } from './addressing_mode.js';
import { Instruction } from './instruction.js';

export class ToRadixLE extends Instruction {
  static type: string = 'TORADIXLE';
  static readonly opcode: Opcode = Opcode.TORADIXLE;

  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8, // Opcode
    OperandType.UINT8, // Indirect
    OperandType.UINT32, // src memory address
    OperandType.UINT32, // dst memory address
    OperandType.UINT32, // radix (immediate)
    OperandType.UINT32, // number of limbs (Immediate)
  ];

  constructor(
    private indirect: number,
    private srcOffset: number,
    private dstOffset: number,
    private radix: number,
    private numLimbs: number,
  ) {
    assert(radix <= 256, 'Radix cannot be greater than 256');
    super();
  }

  public async execute(context: AvmContext): Promise<void> {
    const memory = context.machineState.memory.track(this.type);
    const [srcOffset, dstOffset] = Addressing.fromWire(this.indirect).resolve([this.srcOffset, this.dstOffset], memory);
    const memoryOperations = { reads: 1, writes: this.numLimbs, indirect: this.indirect };
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    // The radix gadget only takes in a Field
    memory.checkTag(TypeTag.FIELD, srcOffset);

    let value: bigint = memory.get(srcOffset).toBigInt();
    const radixBN: bigint = BigInt(this.radix);
    const limbArray = [];

    for (let i = 0; i < this.numLimbs; i++) {
      const limb = value % radixBN;
      limbArray.push(limb);
      value /= radixBN;
    }

    const res = [...limbArray].map(byte => new Uint8(byte));
    memory.setSlice(dstOffset, res);

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }
}
