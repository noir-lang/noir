import { BufferReader } from '@aztec/foundation/serialize';

import times from 'lodash.times';

import { serializeToBuffer } from '../utils/serialize.js';
import { Fq } from './index.js';
import { CircuitType } from './shared.js';

/**
 * Curve data.
 */
export class G1AffineElement {
  /**
   * Element's x coordinate.
   */
  public x: Fq;
  /**
   * Element's y coordinate.
   */
  public y: Fq;

  constructor(x: Fq | bigint, y: Fq | bigint) {
    this.x = typeof x === 'bigint' ? new Fq(x) : x;
    this.y = typeof y === 'bigint' ? new Fq(y) : y;
  }
  /**
   * Serialize as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.x, this.y);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer  or BufferReader to read from.
   * @returns The G1AffineElement.
   */
  static fromBuffer(buffer: Buffer | BufferReader): G1AffineElement {
    const reader = BufferReader.asReader(buffer);
    return new G1AffineElement(reader.readFr(), reader.readFr());
  }
}

/**
 * Used store and serialize a key-value map of commitments where key is the name of the commitment and value is
 * the commitment itself. The name can be e.g. Q_1, Q_2, SIGMA_1 etc.
 */
export class CommitmentMap {
  constructor(
    /**
     * An object used to store the commitments.
     */
    public record: { [name: string]: G1AffineElement },
  ) {}

  /**
   * Serialize as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    const values = Object.entries(this.record);
    return serializeToBuffer(values.length, ...values.flat());
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or BufferReader to read from.
   * @returns The CommitmentMap.
   */
  static fromBuffer(buffer: Buffer | BufferReader): CommitmentMap {
    const reader = BufferReader.asReader(buffer);
    return new CommitmentMap(reader.readMap(G1AffineElement));
  }
}

/**
 * Kate commitment key object for verifying pairing equations.
 * @see proof_system/verification_key/verification_key.hpp
 */
export class VerificationKey {
  constructor(
    /**
     * For Plonk, this is equivalent to the proving system used to prove and verify.
     */
    public circuitType: CircuitType,
    /**
     * The number of gates in this circuit.
     */
    public circuitSize: number,
    /**
     * The number of public inputs in this circuit.
     */
    public numPublicInputs: number,
    /**
     * The commitments for this circuit.
     */
    public commitments: Record<string, G1AffineElement>,
    /**
     * Contains a recursive proof?
     */
    public containsRecursiveProof: boolean,
    /**
     * Recursion stack.
     */
    public recursiveProofPublicInputIndices: number[],
  ) {}

  /**
   * Serialize as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(
      this.circuitType,
      this.circuitSize,
      this.numPublicInputs,
      new CommitmentMap(this.commitments),
      this.containsRecursiveProof,
      serializeToBuffer(this.recursiveProofPublicInputIndices.length, this.recursiveProofPublicInputIndices),
    );
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer to read from.
   * @returns The VerificationKey.
   */
  static fromBuffer(buffer: Buffer | BufferReader): VerificationKey {
    const reader = BufferReader.asReader(buffer);
    return new VerificationKey(
      reader.readNumber(),
      reader.readNumber(),
      reader.readNumber(),
      reader.readObject(CommitmentMap).record,
      reader.readBoolean(),
      reader.readNumberVector(),
    );
  }

  /**
   * Builds a fake verification key that should be accepted by circuits.
   * @returns A turbo verification key.
   */
  static makeFake(): VerificationKey {
    return new VerificationKey(
      CircuitType.TURBO,
      2048,
      116,
      {
        Q_1: new G1AffineElement(
          0x09623eb3c25aa5b16a1a79fd558bac7a7ce62c4560a8c537c77ce80dd339128dn,
          0x1d37b6582ee9e6df9567efb64313471dfa18f520f9ce53161b50dbf7731bc5f9n,
        ),
        Q_2: new G1AffineElement(
          0x2bc4cce83a486a92c92fd59bd84e0f92595baa639fc2ed86b00ffa0dfded2a09n,
          0x2a669a3bdb7a273a015eda494457cc7ed5236f26cee330c290d45a33b9daa948n,
        ),
        Q_3: new G1AffineElement(
          0x2729426c008c085a81bd34d8ef12dd31e80130339ef99d50013a89e4558eee6dn,
          0x0fa4ffe2ee7b7b62eb92608b2251ac31396a718f9b34978888789042b790a301n,
        ),
        Q_4: new G1AffineElement(
          0x2be6b6824a913eb7a57b03cb1ee7bfb4de02f2f65fe8a4e97baa7766ddb353a8n,
          0x2a8a25c49dc63778cd9fe96173f12a2bc77f3682f4c4448f98f1df82c75234a1n,
        ),
        Q_5: new G1AffineElement(
          0x1f85760d6ab567465aadc2f180af9eae3800e6958fec96aef53fd8a7b195d7c0n,
          0x00c6267a0dd5cfc22b3fe804f53e266069c0e36f51885baec1e7e67650c62e17n,
        ),
        Q_ARITHMETIC: new G1AffineElement(
          0x0d9d0f8ece2aa12012fa21e6e5c859e97bd5704e5c122064a66051294bc5e042n,
          0x13f61f54a0ebdf6fee4d4a6ecf693478191de0c2899bcd8e86a636c8d3eff434n,
        ),
        Q_C: new G1AffineElement(
          0x224a99d02c86336737c8dd5b746c40d2be6aead8393889a76a18d664029096e9n,
          0x0f7fe81adcc92a74350eada9622ac453f49ebac24a066a1f83b394df54dfa013n,
        ),
        Q_FIXED_BASE: new G1AffineElement(
          0x060e8a013ed289c2f9fd7473b04f6594b138ddb4b4cf6b901622a14088f04b8dn,
          0x2c83ff74fce56e3d5573b99c7b26d85d5046ce0c6559506acb7a675e7713eb3an,
        ),
        Q_LOGIC: new G1AffineElement(
          0x0721a91cb8da4b917e054f72147e1760cfe0ef3d45090ac0f4961d84ec199696n,
          0x1a25e787b26bd8b50b1a99450f77a424a83513c2b33af268cd253b0587ff50c7n,
        ),
        Q_M: new G1AffineElement(
          0x05dbd8623b8652511e1eb38d38887a69eceb082f807514f09e127237c5213b40n,
          0x1b9325b48c6c225968002318095f89d0ef9cf629b2b7f0172e03bc39aacf6ed8n,
        ),
        Q_RANGE: new G1AffineElement(
          0x04b57a3805e41df328f5ca9aefa40fad5917391543b7b65c6476e60b8f72e9adn,
          0x07c92f3b3e11c8feae96dedc4b14a6226ef3201244f37cfc1ee5b96781f48d2bn,
        ),
        SIGMA_1: new G1AffineElement(
          0x25001d1954a18571eaa007144c5a567bb0d2be4def08a8be918b8c05e3b27d31n,
          0x2c59ed41e09e144eab5de77ca89a2fd783be702a47c951d3112e3de02ce6e47cn,
        ),
        SIGMA_2: new G1AffineElement(
          0x23994e6a23618e60fa01c449a7ab88378709197e186d48d604bfb6931ffb15adn,
          0x11c5ec7a0700570f80088fd5198ab5d5c227f2ad2a455a6edeec024156bb7bebn,
        ),
        SIGMA_3: new G1AffineElement(
          0x00cda5845f23468a13275d18bddae27c6bb189cf9aa95b6a03a0cb6688c7e8d8n,
          0x29639b45cf8607c525cc400b55ebf90205f2f378626dc3406cc59b2d1b474fban,
        ),
        SIGMA_4: new G1AffineElement(
          0x2d299e7928496ea2d37f10b43afd6a80c90a33b483090d18069ffa275eedb2fcn,
          0x2f82121e8de43dc036d99b478b6227ceef34248939987a19011f065d8b5cef5cn,
        ),
      },
      false,
      times(16, i => i),
    );
  }
}
