import { BufferReader } from '@aztec/foundation';
import { Fq, Fr } from '@aztec/foundation/fields';
import { serializeToBuffer } from '../utils/serialize.js';
import times from 'lodash.times';
import { Vector, UInt32, AffineElement } from './shared.js';

export class AggregationObject {
  public publicInputs: Vector<Fr>;
  public proofWitnessIndices: Vector<UInt32>;

  constructor(
    public p0: AffineElement,
    public p1: AffineElement,
    publicInputsData: Fr[],
    proofWitnessIndicesData: UInt32[],
    public hasData = false,
  ) {
    this.publicInputs = new Vector(publicInputsData);
    this.proofWitnessIndices = new Vector(proofWitnessIndicesData);
  }

  toBuffer() {
    return serializeToBuffer(this.p0, this.p1, this.publicInputs, this.proofWitnessIndices, this.hasData);
  }

  static fromBuffer(buffer: Buffer | BufferReader): AggregationObject {
    const reader = BufferReader.asReader(buffer);
    return new AggregationObject(
      reader.readObject(AffineElement),
      reader.readObject(AffineElement),
      reader.readVector(Fr),
      reader.readNumberVector(),
      reader.readBoolean(),
    );
  }

  static makeFake() {
    return new AggregationObject(
      new AffineElement(new Fq(1n), new Fq(2n)),
      new AffineElement(new Fq(1n), new Fq(2n)),
      [],
      times(16, i => 3027 + i),
      false,
    );
  }
}
