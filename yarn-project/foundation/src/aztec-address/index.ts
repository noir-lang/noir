import { Fr } from '../fields/index.js';
import { toBigIntBE } from '../index.js';

export class AztecAddress extends Fr {
  constructor(public readonly buffer: Buffer) {
    super(toBigIntBE(buffer));
  }
}
