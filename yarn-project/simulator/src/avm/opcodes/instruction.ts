import { strict as assert } from 'assert';

import type { AvmContext } from '../avm_context.js';
import { DynamicGasCost, type Gas, GasCosts } from '../avm_gas.js';
import { type BufferCursor } from '../serialization/buffer_cursor.js';
import { Opcode, type OperandType, deserialize, serialize } from '../serialization/instruction_serialization.js';

type InstructionConstructor = {
  new (...args: any[]): Instruction;
  wireFormat?: OperandType[];
};

/**
 * Parent class for all AVM instructions.
 * It's most important aspects are execute and (de)serialize.
 */
export abstract class Instruction {
  /**
   * Consumes gas and executes the instruction.
   * This is the main entry point for the instruction.
   * @param context - The AvmContext in which the instruction executes.
   */
  public run(context: AvmContext): Promise<void> {
    context.machineState.consumeGas(this.gasCost());
    return this.execute(context);
  }

  /**
   * Loads default gas cost for the instruction from the GasCosts table.
   * Instruction sub-classes can override this if their gas cost is not fixed.
   */
  protected gasCost(): Gas {
    const gasCost = GasCosts[this.opcode];
    if (gasCost === DynamicGasCost) {
      throw new Error(`Instruction ${this.type} must define its own gas cost`);
    }
    return gasCost;
  }

  /**
   * Execute the instruction.
   * Instruction sub-classes must implement this.
   * As an AvmContext executes its contract code, it calls this function for
   * each instruction until the machine state signals "halted".
   * @param context - The AvmContext in which the instruction executes.
   */
  protected abstract execute(context: AvmContext): Promise<void>;

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
    assert(!!this.constructor.wireFormat, 'wireFormat must be defined on the class');
    return serialize(this.constructor.wireFormat, this);
  }

  /**
   * Deserializes a subclass of Instruction from a Buffer.
   * If you want to use this, your subclass should specify a {@code static wireFormat: OperandType[]}.
   * @param this Class object to deserialize to.
   * @param buf Buffer to read from.
   * @returns Constructed instance of Class.
   */
  public static deserialize(this: InstructionConstructor, buf: BufferCursor | Buffer): Instruction {
    assert(!!this.wireFormat, 'wireFormat must be defined on the instruction class');
    const res = deserialize(buf, this.wireFormat);
    const args = res.slice(1); // Remove opcode.
    return new this(...args);
  }

  /**
   * Returns the stringified type of the instruction.
   * Instruction sub-classes should have a static `type` property.
   */
  public get type(): string {
    const type = 'type' in this.constructor && (this.constructor.type as string);
    if (!type) {
      throw new Error(`Instruction class ${this.constructor.name} does not have a static 'type' property defined.`);
    }
    return type;
  }

  /**
   * Returns the opcode of the instruction.
   * Instruction sub-classes should have a static `opcode` property.
   */
  public get opcode(): Opcode {
    const opcode = 'opcode' in this.constructor ? (this.constructor.opcode as Opcode) : undefined;
    if (opcode === undefined || Opcode[opcode] === undefined) {
      throw new Error(`Instruction class ${this.constructor.name} does not have a static 'opcode' property defined.`);
    }
    return opcode;
  }
}
