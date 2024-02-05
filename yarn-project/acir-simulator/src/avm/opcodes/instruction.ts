import { assert } from 'console';

import type { AvmContext } from '../avm_context.js';
import { BufferCursor } from '../serialization/buffer_cursor.js';
import { OperandType, deserialize, serialize } from '../serialization/instruction_serialization.js';

/**
 * Parent class for all AVM instructions.
 * It's most important aspects are execute and (de)serialize.
 */
export abstract class Instruction {
  /**
   * Execute the instruction.
   * Instruction sub-classes must implement this.
   * As an AvmContext executes its contract code, it calls this function for
   * each instruction until the machine state signals "halted".
   * @param context - The AvmContext in which the instruction executes.
   */
  public abstract execute(context: AvmContext): Promise<void>;

  /**
   * Generate a string representation of the instruction including
   * the instruction sub-class name all of its flags and operands.
   * @returns Thee string representation.
   */
  public toString(): string {
    let instructionStr = this.constructor.name + ': ';
    // assumes that all properties are flags or operands
    for (const prop of Object.getOwnPropertyNames(this) as (keyof Instruction)[]) {
      instructionStr += `${prop}:${this[prop].toString()}, `;
    }
    return instructionStr;
  }

  /**
   * Serialize the instruction to a Buffer according to its wire format specified in its subclass.
   * If you want to use this, your subclass should specify a {@code static wireFormat: OperandType[]}.
   * @param this - The instruction to serialize.
   * @returns The serialized instruction.
   */
  public serialize(this: any): Buffer {
    assert(this instanceof Instruction);
    return serialize(this.constructor.wireFormat, this);
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
}
