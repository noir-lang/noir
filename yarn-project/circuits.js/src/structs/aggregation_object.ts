import { Fq, Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';

import times from 'lodash.times';

import { serializeToBuffer } from '../utils/serialize.js';
import { UInt32, Vector } from './shared.js';
import { G1AffineElement } from './verification_key.js';

/**
 * Contains the aggregated proof of all the previous kernel iterations.
 *
 * See barretenberg/cpp/src/barretenberg/stdlib/recursion/aggregation_state/native_aggregation_state.hpp
 * for more context.
 */
export class AggregationObject {
  constructor(
    /**
     * One of the 2 aggregated elements storing the verification results of proofs in the past.
     */
    public p0: G1AffineElement,
    /**
     * One of the 2 aggregated elements storing the verification results of proofs in the past.
     */
    public p1: G1AffineElement,
    /**
     * The public inputs of the inner proof (these become the private inputs to the recursive circuit).
     */
    public publicInputs: Fr[],
    /**
     * Witness indices that point to (P0, P1).
     */
    public proofWitnessIndices: UInt32[],
    /**
     * Indicates if this aggregation state contain past (P0, P1).
     */
    public hasData = false,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.p0,
      this.p1,
      new Vector(this.publicInputs),
      new Vector(this.proofWitnessIndices),
      this.hasData,
    );
  }

  /**
   * Deserializes this object from a buffer.
   * @param buffer - The buffer representation of this object.
   * @returns The deserialized object.
   */
  public static fromBuffer(buffer: Buffer | BufferReader): AggregationObject {
    const reader = BufferReader.asReader(buffer);
    return new AggregationObject(
      reader.readObject(G1AffineElement),
      reader.readObject(G1AffineElement),
      reader.readVector(Fr),
      reader.readNumberVector(),
      reader.readBoolean(),
    );
  }

  /**
   * Creates a fake object for testing.
   * @returns The fake object.
   */
  public static makeFake(): AggregationObject {
    return new AggregationObject(
      new G1AffineElement(new Fq(1n), new Fq(2n)),
      new G1AffineElement(new Fq(1n), new Fq(2n)),
      [],
      times(16, i => 3027 + i),
      false,
    );
  }
}
