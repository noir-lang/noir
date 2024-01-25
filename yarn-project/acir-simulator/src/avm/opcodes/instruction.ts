import { AvmMachineState } from '../avm_machine_state.js';
import { TypeTag } from '../avm_memory_types.js';
import { AvmJournal } from '../journal/index.js';

export const AVM_OPERAND_BYTE_LENGTH = 4; // Keep in sync with cpp code
export const AVM_OPCODE_BYTE_LENGTH = 1; // Keep in sync with cpp code

/**
 * Opcode base class
 */
export abstract class Instruction {
  abstract execute(machineState: AvmMachineState, journal: AvmJournal): Promise<void>;

  incrementPc(machineState: AvmMachineState): void {
    machineState.pc++;
  }

  halt(machineState: AvmMachineState): void {
    machineState.halted = true;
  }

  static checkTags(machineState: AvmMachineState, tag: TypeTag, ...offsets: number[]) {
    for (const off of offsets) {
      if (machineState.memory.getTag(off) !== tag) {
        const error = `Offset ${off} has tag ${TypeTag[machineState.memory.getTag(off)]}, expected ${TypeTag[tag]}`;
        throw new InstructionExecutionError(error);
      }
    }
  }
}

export class InstructionExecutionError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'InstructionExecutionError';
  }
}
