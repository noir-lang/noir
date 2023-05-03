import { Fr } from '@aztec/foundation/fields';
import { assertLength, range } from '../utils/jsUtils.js';
import { serializeToBuffer } from '../utils/serialize.js';
import { toBufferBE } from '@aztec/foundation/bigint-buffer';

export class MembershipWitness<N extends number> {
  constructor(pathSize: N, public leafIndex: bigint, public siblingPath: Fr[]) {
    assertLength(this, 'siblingPath', pathSize);
  }

  toBuffer() {
    return serializeToBuffer(toBufferBE(this.leafIndex, 32), ...this.siblingPath);
  }

  static mock(size: number, start: number) {
    return new MembershipWitness(
      size,
      BigInt(start),
      range(size, start).map(x => new Fr(BigInt(x))),
    );
  }

  public static random<N extends number>(pathSize: N) {
    return new MembershipWitness<N>(
      pathSize,
      0n,
      Array(pathSize)
        .fill(0)
        .map(() => Fr.random()),
    );
  }

  public static empty<N extends number>(pathSize: N, leafIndex: bigint) {
    const arr = Array(pathSize)
      .fill(0)
      .map(() => Fr.ZERO);
    return new MembershipWitness<N>(pathSize, leafIndex, arr);
  }

  static fromBufferArray(leafIndex: bigint, siblingPath: Buffer[]) {
    return new MembershipWitness(
      siblingPath.length,
      leafIndex,
      siblingPath.map(x => Fr.fromBuffer(x)),
    );
  }
}
