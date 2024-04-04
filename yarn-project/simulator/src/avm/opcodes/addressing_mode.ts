import { strict as assert } from 'assert';

import { type TaggedMemoryInterface } from '../avm_memory_types.js';

export enum AddressingMode {
  DIRECT,
  INDIRECT,
  INDIRECT_PLUS_CONSTANT, // Not implemented yet.
}

/** A class to represent the addressing mode of an instruction. */
export class Addressing {
  public constructor(
    /** The addressing mode for each operand. The length of this array is the number of operands of the instruction. */
    private readonly modePerOperand: AddressingMode[],
  ) {
    assert(modePerOperand.length <= 8, 'At most 8 operands are supported');
  }

  public static fromWire(wireModes: number): Addressing {
    // The modes are stored in the wire format as a byte, with each bit representing the mode for an operand.
    // The least significant bit represents the zeroth operand, and the most significant bit represents the last operand.
    const modes = new Array<AddressingMode>(8);
    for (let i = 0; i < 8; i++) {
      modes[i] = (wireModes & (1 << i)) === 0 ? AddressingMode.DIRECT : AddressingMode.INDIRECT;
    }
    return new Addressing(modes);
  }

  public toWire(): number {
    // The modes are stored in the wire format as a byte, with each bit representing the mode for an operand.
    // The least significant bit represents the zeroth operand, and the least significant bit represents the last operand.
    let wire: number = 0;
    for (let i = 0; i < 8; i++) {
      if (this.modePerOperand[i] === AddressingMode.INDIRECT) {
        wire |= 1 << i;
      }
    }
    return wire;
  }

  /** Returns how many operands use the given addressing mode. */
  public count(mode: AddressingMode): number {
    return this.modePerOperand.filter(m => m === mode).length;
  }

  /**
   * Resolves the offsets using the addressing mode.
   * @param offsets The offsets to resolve.
   * @param mem The memory to use for resolution.
   * @returns The resolved offsets. The length of the returned array is the same as the length of the input array.
   */
  public resolve(offsets: number[], mem: TaggedMemoryInterface): number[] {
    assert(offsets.length <= this.modePerOperand.length);
    const resolved = new Array(offsets.length);
    for (const [i, offset] of offsets.entries()) {
      switch (this.modePerOperand[i]) {
        case AddressingMode.INDIRECT:
          // NOTE(reviewer): less than equal is a deviation from the spec - i dont see why this shouldnt be possible!
          mem.checkIsValidMemoryOffsetTag(offset);
          resolved[i] = Number(mem.get(offset).toBigInt());
          break;
        case AddressingMode.DIRECT:
          resolved[i] = offset;
          break;
        default:
          throw new Error(`Unimplemented addressing mode: ${AddressingMode[this.modePerOperand[i]]}`);
      }
    }
    return resolved;
  }
}
