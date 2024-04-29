import type { AvmContext } from '../avm_context.js';
import { Field, type MemoryValue } from '../avm_memory_types.js';
import { Opcode } from '../serialization/instruction_serialization.js';
import { GetterInstruction } from './instruction_impl.js';

export class L2GasLeft extends GetterInstruction {
  static type: string = 'L2GASLEFT';
  static readonly opcode: Opcode = Opcode.L2GASLEFT;

  // TODO(@spalladino) Protocol specs specifies that the value should be an Uint32, not a Field.
  protected getValue(context: AvmContext): MemoryValue {
    return new Field(context.machineState.l2GasLeft);
  }
}

export class DAGasLeft extends GetterInstruction {
  static type: string = 'DAGASLEFT';
  static readonly opcode: Opcode = Opcode.DAGASLEFT;

  protected getValue(context: AvmContext): MemoryValue {
    return new Field(context.machineState.daGasLeft);
  }
}
