import { inspect } from 'util';

import { toBufferBE } from '../bigint-buffer/index.js';
import { Fr } from '../fields/index.js';

/** A selector is the first 4 bytes of the hash of a signature. */
export abstract class Selector {
  /** The size of the selector in bytes. */
  public static SIZE = 4;

  constructor(/** Value of the selector */ public value: number) {
    if (value > 2 ** (Selector.SIZE * 8) - 1) {
      throw new Error(`Selector must fit in ${Selector.SIZE} bytes (got value ${value}).`);
    }
  }

  /**
   * Checks if the selector is empty (all bytes are 0).
   * @returns True if the selector is empty (all bytes are 0).
   */
  public isEmpty(): boolean {
    return this.value === 0;
  }

  /**
   * Serialize as a buffer.
   * @param bufferSize - The buffer size.
   * @returns The buffer.
   */
  toBuffer(bufferSize = Selector.SIZE): Buffer {
    return toBufferBE(BigInt(this.value), bufferSize);
  }

  /**
   * Serialize as a hex string.
   * @returns The string.
   */
  toString(): string {
    return '0x' + this.toBuffer().toString('hex');
  }

  [inspect.custom]() {
    return `Selector<${this.toString()}>`;
  }

  /**
   * Checks if this selector is equal to another.
   * @param other - The other selector.
   * @returns True if the selectors are equal.
   */
  equals(other: Selector): boolean {
    return this.value === other.value;
  }

  /**
   * Returns a new field with the same contents as this EthAddress.
   *
   * @returns An Fr instance.
   */
  public toField() {
    return new Fr(BigInt(this.value));
  }
}
