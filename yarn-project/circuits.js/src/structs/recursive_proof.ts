import { makeTuple } from '@aztec/foundation/array';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { Proof, makeEmptyProof } from './proof.js';

/**
 * The Recursive proof class is a wrapper around the circuit's proof.
 * We store the proof in 2 forms for convenience. The first is in the 'fields' format.
 * This is a list of fields, for which there are distinct lengths based on the level of recursion.
 * This 'fields' version does not contain the circuits public inputs
 * We also store the raw binary proof which van be directly verified.
 *
 * The 'fieldsValid' member is set to false in the case where this object is constructed solely from the 'binary' proof
 * This is usually when the proof has been received from clients and signals to provers that the 'fields' version needs to be generated
 */
export class RecursiveProof<N extends number> {
  constructor(
    /**
     * Holds the serialized proof data in an array of fields, this is without the public inputs
     */
    public proof: Tuple<Fr, N>,

    /**
     * Holds the serialized proof data in a binary buffer, this contains the public inputs
     */
    public binaryProof: Proof,
    /**
     * This flag determines if the 'proof' member is valid, or if we need to generate it from the 'binaryProof' first
     */
    public fieldsValid: boolean,
  ) {}

  /**
   * Create a Proof from a Buffer or BufferReader.
   * Expects a length-encoding.
   *
   * @param buffer - A Buffer or BufferReader containing the length-encoded proof data.
   * @returns A Proof instance containing the decoded proof data.
   */
  static fromBuffer<N extends number | undefined>(
    buffer: Buffer | BufferReader,
    expectedSize?: N,
  ): RecursiveProof<N extends number ? N : number> {
    const reader = BufferReader.asReader(buffer);
    const size = reader.readNumber();
    if (typeof expectedSize === 'number' && expectedSize !== size) {
      throw new Error(`Expected proof length ${expectedSize}, got ${size}`);
    }
    return new RecursiveProof(reader.readArray(size, Fr) as any, Proof.fromBuffer(reader), reader.readBoolean());
  }

  /**
   * Convert the Proof instance to a custom Buffer format.
   * This function serializes the Proof's buffer length and data sequentially into a new Buffer.
   *
   * @returns A Buffer containing the serialized proof data in custom format.
   */
  public toBuffer() {
    return serializeToBuffer(this.proof.length, this.proof, this.binaryProof, this.fieldsValid);
  }

  /**
   * Serialize the Proof instance to a hex string.
   * @returns The hex string representation of the proof data.
   */
  public toString() {
    return this.toBuffer().toString('hex');
  }

  /**
   * Deserialize a Proof instance from a hex string.
   * @param str - A hex string to deserialize from.
   * @returns - A new Proof instance.
   */
  static fromString<N extends number | undefined>(
    str: string,
    expectedSize?: N,
  ): RecursiveProof<N extends number ? N : number> {
    return RecursiveProof.fromBuffer(Buffer.from(str, 'hex'), expectedSize);
  }
}

/**
 * Makes an empty proof.
 * Note: Used for local devnet milestone where we are not proving anything yet.
 * @returns The empty "proof".
 */
export function makeEmptyRecursiveProof<N extends number>(size: N) {
  return new RecursiveProof(makeTuple<Fr, N>(size, Fr.zero), makeEmptyProof(), true);
}

export function makeRecursiveProof<PROOF_LENGTH extends number>(size: PROOF_LENGTH, seed = 1) {
  return new RecursiveProof<PROOF_LENGTH>(
    makeTuple<Fr, PROOF_LENGTH>(size, (i: number) => new Fr(i), seed),
    makeEmptyProof(),
    true,
  );
}

/**
 * Makes an instance of the recursive proof from a binary only proof
 * @returns The proof object
 */
export function makeRecursiveProofFromBinary<PROOF_LENGTH extends number>(proof: Proof, size: PROOF_LENGTH) {
  return new RecursiveProof<PROOF_LENGTH>(makeTuple<Fr, PROOF_LENGTH>(size, Fr.zero), proof, false);
}
