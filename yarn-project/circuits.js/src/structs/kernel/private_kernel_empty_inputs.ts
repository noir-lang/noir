import { type Fr } from '@aztec/foundation/fields';
import { serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { type RECURSIVE_PROOF_LENGTH } from '../../constants.gen.js';
import { type Header } from '../header.js';
import { type RecursiveProof } from '../recursive_proof.js';
import { type VerificationKeyAsFields } from '../verification_key.js';

export type PrivateKernelEmptyInputData = Omit<FieldsOf<PrivateKernelEmptyInputs>, 'emptyNested'>;

export class PrivateKernelEmptyInputs {
  constructor(
    public readonly emptyNested: EmptyNestedData,
    public readonly header: Header,
    public readonly chainId: Fr,
    public readonly version: Fr,
    public readonly vkTreeRoot: Fr,
  ) {}

  toBuffer(): Buffer {
    return serializeToBuffer(this.emptyNested, this.header, this.chainId, this.version, this.vkTreeRoot);
  }

  static from(fields: FieldsOf<PrivateKernelEmptyInputs>) {
    return new PrivateKernelEmptyInputs(
      fields.emptyNested,
      fields.header,
      fields.chainId,
      fields.version,
      fields.vkTreeRoot,
    );
  }
}

export class EmptyNestedCircuitInputs {
  toBuffer(): Buffer {
    return Buffer.alloc(0);
  }
}

export class EmptyNestedData {
  constructor(
    public readonly proof: RecursiveProof<typeof RECURSIVE_PROOF_LENGTH>,
    public readonly vk: VerificationKeyAsFields,
  ) {}

  toBuffer(): Buffer {
    return serializeToBuffer(this.proof, this.vk);
  }
}
