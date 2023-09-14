import { BufferReader, Tuple } from '@aztec/foundation/serialize';

import { privateKernelDummyPreviousKernel } from '../../cbind/circuits.gen.js';
import { CircuitsWasm, VK_TREE_HEIGHT, makeTuple } from '../../index.js';
import { serializeToBuffer } from '../../utils/serialize.js';
import { Fr } from '../index.js';
import { Proof, makeEmptyProof } from '../proof.js';
import { UInt32 } from '../shared.js';
import { VerificationKey } from '../verification_key.js';
import { KernelCircuitPublicInputs } from './public_inputs.js';

/**
 * Data of the previous kernel iteration in the chain of kernels.
 */
export class PreviousKernelData {
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

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.publicInputs, this.proof, this.vk, this.vkIndex, this.vkPath);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PreviousKernelData {
    const reader = BufferReader.asReader(buffer);
    return new this(
      reader.readObject(KernelCircuitPublicInputs),
      reader.readObject(Proof),
      reader.readObject(VerificationKey),
      reader.readNumber(),
      reader.readArray(VK_TREE_HEIGHT, Fr),
    );
  }

  /**
   * Creates an empty instance, valid enough to be accepted by circuits.
   * @returns The empty instance.
   */
  static empty(): PreviousKernelData {
    return new PreviousKernelData(
      KernelCircuitPublicInputs.empty(),
      makeEmptyProof(),
      VerificationKey.makeFake(),
      0,
      makeTuple(VK_TREE_HEIGHT, Fr.zero),
    );
  }
}

/**
 * Dummy data used in the first kernel in the chain of kernels.
 */
export class DummyPreviousKernelData {
  private static instance: DummyPreviousKernelData;

  private constructor(private data: PreviousKernelData) {}

  /**
   * Gets the dummy data.
   * @param wasm - The circuits wasm instance.
   * @returns The dummy previous kernel data.
   */
  public static getDummyPreviousKernelData(wasm: CircuitsWasm): PreviousKernelData {
    if (!DummyPreviousKernelData.instance) {
      const data = privateKernelDummyPreviousKernel(wasm);
      DummyPreviousKernelData.instance = new DummyPreviousKernelData(data);
    }

    return DummyPreviousKernelData.instance.getData();
  }

  /**
   * Gets the the dummy data.
   * @returns The dummy previous kernel data.
   */
  public getData(): PreviousKernelData {
    return this.data;
  }
}
