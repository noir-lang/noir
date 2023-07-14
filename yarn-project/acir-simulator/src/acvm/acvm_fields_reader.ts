import { Fr } from '@aztec/foundation/fields';

import { ACVMField, fromACVMField } from './acvm.js';

/**
 * A utility for reading an array of fields.
 */
export class ACVMFieldsReader {
  private offset = 0;

  constructor(private fields: ACVMField[]) {}

  /**
   * Reads a field.
   * @returns The field.
   */
  public readField(): Fr {
    const acvmField = this.fields[this.offset];
    if (!acvmField) throw new Error('Not enough fields.');
    this.offset += 1;
    return fromACVMField(acvmField);
  }

  /**
   * Reads an array of fields.
   * @param length - The length of the array.
   * @returns The array of fields.
   */
  public readFieldArray(length: number): Fr[] {
    const arr: Fr[] = [];
    for (let i = 0; i < length; i++) {
      arr.push(this.readField());
    }
    return arr;
  }

  /**
   * Reads a number.
   * @returns The number.
   */
  public readNumber(): number {
    const num = +this.fields[this.offset];
    this.offset += 1;
    return num;
  }

  /**
   * Reads a number array.
   * @param length - The length of the array.
   * @returns The number.
   */
  public readNumberArray(length: number): number[] {
    const arr: number[] = [];
    for (let i = 0; i < length; i++) {
      arr.push(this.readNumber());
    }
    return arr;
  }
}
