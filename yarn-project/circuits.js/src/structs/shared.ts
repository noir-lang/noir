import { Bufferable, serializeToBuffer } from '../utils/serialize.js';

/**
 * Implementation of a vector. Matches how we are serializing and deserializing vectors in cpp (length in the first position, followed by the items).
 */
export class Vector<T extends Bufferable> {
  constructor(
    /**
     * Items in the vector.
     */
    public items: T[],
  ) {}

  toBuffer() {
    return serializeToBuffer(this.items.length, this.items);
  }

  toFriendlyJSON() {
    return this.items;
  }
}

/**
 * A type alias for a 32-bit unsigned integer.
 */
export type UInt32 = number;

/**
 * CircuitType replaces ComposerType for now. When using Plonk, CircuitType is equivalent to the information of the proving system that will be used
 * to construct a proof. In the future Aztec zk stack, more information must be specified (e.g., the curve over which circuits are  constructed;
 * Plonk vs Honk; zk-SNARK or just SNARK; etc).
 */
export enum CircuitType {
  STANDARD = 0,
  TURBO = 1,
  ULTRA = 2,
}

/**
 * Rollup types.
 */
export enum RollupTypes {
  Base = 0,
  Merge = 1,
  Root = 2,
}

/**
 * String encoding of serialised buffer data
 */
export const STRING_ENCODING: BufferEncoding = 'hex';
