import { assert } from 'console';

import { AvmMachineState } from '../avm_machine_state.js';
import { TypeTag } from '../avm_memory_types.js';
import { AvmJournal } from '../journal/index.js';
import { BufferCursor } from '../serialization/buffer_cursor.js';
import { OperandType, deserialize, serialize } from '../serialization/instruction_serialization.js';

/**
 * Parent class for all AVM instructions.
 * It's most important aspects are execution and (de)serialization.
 */
export abstract class Instruction {
  public abstract execute(machineState: AvmMachineState, journal: AvmJournal): Promise<void>;

  incrementPc(machineState: AvmMachineState): void {
    machineState.pc++;
  }

  halt(machineState: AvmMachineState): void {
    machineState.halted = true;
  }

  revert(machineState: AvmMachineState): void {
    machineState.halted = true;
    machineState.reverted = true;
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

  /**
   * Deserializes a subclass of Instruction from a Buffer.
   * If you want to use this, your subclass should specify a {@code static wireFormat: OperandType[]}.
   * @param this Class object to deserialize to.
   * @param buf Buffer to read from.
   * @returns Constructed instance of Class.
   */
  public static deserialize<T extends { new (...args: any[]): InstanceType<T>; wireFormat: OperandType[] }>(
    this: T,
    buf: BufferCursor | Buffer,
  ): InstanceType<T> {
    const res = deserialize(buf, this.wireFormat);
    const args = res.slice(1) as ConstructorParameters<T>; // Remove opcode.
    return new this(...args);
  }

  public serialize(this: any): Buffer {
    assert(this instanceof Instruction);
    return serialize(this.constructor.wireFormat, this);
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
