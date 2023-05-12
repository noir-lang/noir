import { Fr } from '@aztec/foundation/fields';
import times from 'lodash.times';
import { CircuitsWasm, getDummyPreviousKernelData } from '../../index.js';
import { assertLength } from '../../utils/jsUtils.js';
import { serializeToBuffer } from '../../utils/serialize.js';
import { VK_TREE_HEIGHT } from '../constants.js';
import { UInt32, UInt8Vector } from '../shared.js';
import { VerificationKey } from '../verification_key.js';
import { KernelCircuitPublicInputs } from './public_inputs.js';
import { makeEmptyProof } from './private_kernel.js';
import { BufferReader } from '@aztec/foundation/serialize';

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
    public proof: UInt8Vector,
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
    public vkSiblingPath: Fr[],
  ) {
    assertLength(this, 'vkSiblingPath', VK_TREE_HEIGHT);
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.publicInputs, this.proof, this.vk, this.vkIndex, this.vkSiblingPath);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PreviousKernelData {
    const reader = BufferReader.asReader(buffer);
    return new PreviousKernelData(
      reader.readObject(KernelCircuitPublicInputs),
      reader.readObject(UInt8Vector),
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
      times(VK_TREE_HEIGHT, Fr.zero),
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
  public static async getDummyPreviousKernelData(wasm: CircuitsWasm): Promise<PreviousKernelData> {
    if (!DummyPreviousKernelData.instance) {
      const data = await getDummyPreviousKernelData(wasm);
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
