import type { AvmContext } from '../avm_context.js';
import { type MemoryValue, Uint8 } from '../avm_memory_types.js';
import { Opcode } from '../serialization/instruction_serialization.js';
import { ThreeOperandInstruction } from './instruction_impl.js';

abstract class ComparatorInstruction extends ThreeOperandInstruction {
  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { reads: 2, writes: 1, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    memory.checkTags(this.inTag, this.aOffset, this.bOffset);

    const a = memory.get(this.aOffset);
    const b = memory.get(this.bOffset);

    const dest = new Uint8(this.compare(a, b) ? 1 : 0);
    memory.set(this.dstOffset, dest);

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }

  protected abstract compare(a: MemoryValue, b: MemoryValue): boolean;
}

export class Eq extends ComparatorInstruction {
  static readonly type: string = 'EQ';
  static readonly opcode = Opcode.EQ;

  protected compare(a: MemoryValue, b: MemoryValue): boolean {
    return a.equals(b);
  }
}

export class Lt extends ComparatorInstruction {
  static readonly type: string = 'LT';
  static readonly opcode = Opcode.LT;

  protected compare(a: MemoryValue, b: MemoryValue): boolean {
    return a.lt(b);
  }
}

export class Lte extends ComparatorInstruction {
  static readonly type: string = 'LTE';
  static readonly opcode = Opcode.LTE;

  protected compare(a: MemoryValue, b: MemoryValue): boolean {
    return a.lt(b) || a.equals(b);
  }
}
