import { makeTuple } from '@aztec/foundation/array';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { VK_TREE_HEIGHT } from '../../constants.gen.js';
import { Proof, makeEmptyProof } from '../proof.js';
import { type UInt32 } from '../shared.js';
import { VerificationKey } from '../verification_key.js';
import { KernelCircuitPublicInputs } from './kernel_circuit_public_inputs.js';

export class KernelData {
  constructor(
    /**
     * Public inputs of the previous kernel.
     */
    public publicInputs: KernelCircuitPublicInputs,
    /**
     * Proof of the previous kernel.
     */
    public proof: Proof,
    /**
     * Verification key of the previous kernel.
     */
    public vk: VerificationKey,
    /**
     * Index of the previous kernel's vk in a tree of vks.
     */
    public vkIndex: UInt32,
    /**
     * Sibling path of the previous kernel's vk in a tree of vks.
     */
    public vkPath: Tuple<Fr, typeof VK_TREE_HEIGHT>,
  ) {}

  static empty(): KernelData {
    return new this(
      KernelCircuitPublicInputs.empty(),
      makeEmptyProof(),
      VerificationKey.makeFake(),
      0,
      makeTuple(VK_TREE_HEIGHT, Fr.zero),
    );
  }

  static fromBuffer(buffer: Buffer | BufferReader): KernelData {
    const reader = BufferReader.asReader(buffer);
    return new this(
      reader.readObject(KernelCircuitPublicInputs),
      reader.readObject(Proof),
      reader.readObject(VerificationKey),
      reader.readNumber(),
      reader.readArray(VK_TREE_HEIGHT, Fr),
    );
  }

  toBuffer() {
    return serializeToBuffer(this.publicInputs, this.proof, this.vk, this.vkIndex, this.vkPath);
  }
}
