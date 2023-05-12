import { Fq, Fr } from '@aztec/foundation/fields';
import { serializeToBuffer } from '../utils/serialize.js';
import times from 'lodash.times';
import { Vector, UInt32, AffineElement } from './shared.js';
import { BufferReader } from '@aztec/foundation/serialize';

/**
 * Contains the aggregated proof of all the previous kernel iterations.
 *
 * See circuits/cpp/barretenberg/cpp/src/barretenberg/stdlib/recursion/aggregation_state/native_aggregation_state.hpp
 * for more context.
 */
export class AggregationObject {
  /**
   * The public inputs of the inner proof (these become the private inputs to the recursive circuit).
   */
  public publicInputs: Vector<Fr>;
  /**
   * Witness indices that point to (P0, P1).
   */
  public proofWitnessIndices: Vector<UInt32>;

  constructor(
    /**
     * One of the 2 aggregated elements storing the verification results of proofs in the past.
     */
    public p0: AffineElement,
    /**
     * One of the 2 aggregated elements storing the verification results of proofs in the past.
     */
    public p1: AffineElement,
    publicInputsData: Fr[],
    proofWitnessIndicesData: UInt32[],
    /**
     * Indicates if this aggregation state contain past (P0, P1).
     */
    public hasData = false,
  ) {
    this.publicInputs = new Vector(publicInputsData);
    this.proofWitnessIndices = new Vector(proofWitnessIndicesData);
  }

  /**
   * Serializes this object to a buffer.
   * @returns The buffer representation of this object.
   */
  public toBuffer(): Buffer {
    return serializeToBuffer(this.p0, this.p1, this.publicInputs, this.proofWitnessIndices, this.hasData);
  }

  /**
   * Deserializes this object from a buffer.
   * @param buffer - The buffer representation of this object.
   * @returns The deserialized object.
   */
  public static fromBuffer(buffer: Buffer | BufferReader): AggregationObject {
    const reader = BufferReader.asReader(buffer);
    return new AggregationObject(
      reader.readObject(AffineElement),
      reader.readObject(AffineElement),
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
      new AffineElement(new Fq(1n), new Fq(2n)),
      new AffineElement(new Fq(1n), new Fq(2n)),
      [],
      times(16, i => 3027 + i),
      false,
    );
  }
}
