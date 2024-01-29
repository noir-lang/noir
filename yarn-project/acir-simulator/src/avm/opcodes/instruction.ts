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
    for (const offset of offsets) {
      checkTag(machineState, tag, offset);
    }
  }

  static checkTagsRange(machineState: AvmMachineState, tag: TypeTag, startOffset: number, size: number) {
    for (let offset = startOffset; offset < startOffset + size; offset++) {
      checkTag(machineState, tag, offset);
    }
  }
}

/**
 * Checks that the memory at the given offset has the given tag.
 */
function checkTag(machineState: AvmMachineState, tag: TypeTag, offset: number) {
  if (machineState.memory.getTag(offset) !== tag) {
    const error = `Offset ${offset} has tag ${TypeTag[machineState.memory.getTag(offset)]}, expected ${TypeTag[tag]}`;
    throw new InstructionExecutionError(error);
  }
}

export class InstructionExecutionError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'InstructionExecutionError';
  }
}
