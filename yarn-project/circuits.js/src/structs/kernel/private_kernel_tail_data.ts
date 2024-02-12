import { makeTuple } from '@aztec/foundation/array';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { VK_TREE_HEIGHT } from '../../constants.gen.js';
import { Proof, makeEmptyProof } from '../proof.js';
import { UInt32 } from '../shared.js';
import { VerificationKey } from '../verification_key.js';
import { PrivateKernelTailCircuitPublicInputs } from './private_kernel_tail_circuit_public_inputs.js';

/**
 * Data of the private kernel tail.
 */
export class PrivateKernelTailData {
  constructor(
    /**
     * Outputs of the private kernel tail.
     */
    public publicInputs: PrivateKernelTailCircuitPublicInputs,
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

  static fromBuffer(buffer: Buffer | BufferReader): PrivateKernelTailData {
    const reader = BufferReader.asReader(buffer);
    return new this(
      reader.readObject(PrivateKernelTailCircuitPublicInputs),
      reader.readObject(Proof),
      reader.readObject(VerificationKey),
      reader.readNumber(),
      reader.readArray(VK_TREE_HEIGHT, Fr),
    );
  }

  static empty(): PrivateKernelTailData {
    return new this(
      PrivateKernelTailCircuitPublicInputs.empty(),
      makeEmptyProof(),
      VerificationKey.makeFake(),
      0,
      makeTuple(VK_TREE_HEIGHT, Fr.zero),
    );
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.publicInputs, this.proof, this.vk, this.vkIndex, this.vkPath);
  }
}
