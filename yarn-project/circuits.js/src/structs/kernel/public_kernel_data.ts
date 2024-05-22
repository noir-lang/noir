import { makeTuple } from '@aztec/foundation/array';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { NESTED_RECURSIVE_PROOF_LENGTH, VK_TREE_HEIGHT } from '../../constants.gen.js';
import { RecursiveProof, makeEmptyRecursiveProof } from '../recursive_proof.js';
import { type UInt32 } from '../shared.js';
import { VerificationKeyData } from '../verification_key.js';
import { PublicKernelCircuitPublicInputs } from './public_kernel_circuit_public_inputs.js';

/**
 * Data of the previous public kernel iteration in the chain of kernels.
 */
export class PublicKernelData {
  constructor(
    /**
     * Public inputs of the previous kernel.
     */
    public publicInputs: PublicKernelCircuitPublicInputs,
    /**
     * Proof of the previous kernel.
     */
    public proof: RecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>,
    /**
     * Verification key of the previous kernel.
     */
    public vk: VerificationKeyData,
    /**
     * Index of the previous kernel's vk in a tree of vks.
     */
    public vkIndex: UInt32,
    /**
     * Sibling path of the previous kernel's vk in a tree of vks.
     */
    public vkPath: Tuple<Fr, typeof VK_TREE_HEIGHT>,
  ) {}

  static fromBuffer(buffer: Buffer | BufferReader): PublicKernelData {
    const reader = BufferReader.asReader(buffer);
    return new this(
      reader.readObject(PublicKernelCircuitPublicInputs),
      RecursiveProof.fromBuffer(reader, NESTED_RECURSIVE_PROOF_LENGTH),
      reader.readObject(VerificationKeyData),
      reader.readNumber(),
      reader.readArray(VK_TREE_HEIGHT, Fr),
    );
  }

  static empty(): PublicKernelData {
    return new this(
      PublicKernelCircuitPublicInputs.empty(),
      makeEmptyRecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>(NESTED_RECURSIVE_PROOF_LENGTH),
      VerificationKeyData.makeFake(),
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
