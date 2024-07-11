import { makeTuple } from '@aztec/foundation/array';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { RECURSIVE_PROOF_LENGTH, VK_TREE_HEIGHT } from '../../constants.gen.js';
import { RecursiveProof, makeEmptyRecursiveProof } from '../recursive_proof.js';
import { type UInt32 } from '../shared.js';
import { VerificationKeyData } from '../verification_key.js';
import { KernelCircuitPublicInputs } from './kernel_circuit_public_inputs.js';

// TODO: less ambiguous name
export class KernelData {
  constructor(
    /**
     * Public inputs of the previous kernel.
     */
    public publicInputs: KernelCircuitPublicInputs,
    /**
     * The previous kernel's proof (may be a tube proof or public kernel proof).
     */
    public proof: RecursiveProof<typeof RECURSIVE_PROOF_LENGTH>,
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

  static empty(): KernelData {
    return new this(
      KernelCircuitPublicInputs.empty(),
      makeEmptyRecursiveProof(RECURSIVE_PROOF_LENGTH),
      VerificationKeyData.makeFake(),
      0,
      makeTuple(VK_TREE_HEIGHT, Fr.zero),
    );
  }

  static fromBuffer(buffer: Buffer | BufferReader): KernelData {
    const reader = BufferReader.asReader(buffer);
    return new this(
      reader.readObject(KernelCircuitPublicInputs),
      RecursiveProof.fromBuffer(reader, RECURSIVE_PROOF_LENGTH),
      reader.readObject(VerificationKeyData),
      reader.readNumber(),
      reader.readArray(VK_TREE_HEIGHT, Fr),
    );
  }

  toBuffer() {
    return serializeToBuffer(this.publicInputs, this.proof, this.vk, this.vkIndex, this.vkPath);
  }
}
