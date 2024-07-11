import {
  AVM_REQUEST,
  type AvmProvingRequest,
  type MerkleTreeId,
  type ProcessedTx,
  type PublicKernelRequest,
  PublicKernelType,
} from '@aztec/circuit-types';
import {
  type AppendOnlyTreeSnapshot,
  type BaseRollupInputs,
  type NESTED_RECURSIVE_PROOF_LENGTH,
  type Proof,
  type RecursiveProof,
  type VerificationKeyData,
} from '@aztec/circuits.js';

export enum TX_PROVING_CODE {
  NOT_READY,
  READY,
  COMPLETED,
}

export type PublicFunction = {
  vmRequest: AvmProvingRequest | undefined;
  vmProof: Proof | undefined;
  previousProofType: PublicKernelType;
  previousKernelProven: boolean;
  publicKernelRequest: PublicKernelRequest;
};

// Type encapsulating the instruction to the orchestrator as to what
// needs to be proven next
export type TxProvingInstruction = {
  code: TX_PROVING_CODE;
  function: PublicFunction | undefined;
};

/**
 * Helper class to manage the proving cycle of a transaction
 * This includes the public VMs and the public kernels
 * Also stores the inputs to the base rollup for this transaction and the tree snapshots
 */
export class TxProvingState {
  private publicFunctions: PublicFunction[] = [];

  constructor(
    public readonly processedTx: ProcessedTx,
    public readonly baseRollupInputs: BaseRollupInputs,
    public readonly treeSnapshots: Map<MerkleTreeId, AppendOnlyTreeSnapshot>,
    privateKernelVk: VerificationKeyData,
  ) {
    let previousProofType = PublicKernelType.NON_PUBLIC;
    for (let i = 0; i < processedTx.publicProvingRequests.length; i++) {
      const provingRequest = processedTx.publicProvingRequests[i];
      const kernelRequest = provingRequest.type === AVM_REQUEST ? provingRequest.kernelRequest : provingRequest;
      // the first circuit has a valid previous proof, it came from private
      if (i === 0) {
        kernelRequest.inputs.previousKernel.vk = privateKernelVk;
        kernelRequest.inputs.previousKernel.clientIvcProof = processedTx.clientIvcProof;
      }
      const vmRequest = provingRequest.type === AVM_REQUEST ? provingRequest : undefined;
      const publicFunction: PublicFunction = {
        vmRequest,
        vmProof: undefined,
        previousProofType,
        previousKernelProven: i === 0,
        publicKernelRequest: kernelRequest,
      };
      this.publicFunctions.push(publicFunction);
      previousProofType = kernelRequest.type;
    }
  }

  // Updates the transaction's proving state after completion of a kernel proof
  // Returns an instruction as to the next stage of tx proving
  public getNextPublicKernelFromKernelProof(
    provenIndex: number,
    proof: RecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>,
    verificationKey: VerificationKeyData,
  ): TxProvingInstruction {
    const kernelRequest = this.getPublicFunctionState(provenIndex).publicKernelRequest;
    const nextKernelIndex = provenIndex + 1;
    if (nextKernelIndex >= this.publicFunctions.length) {
      // The next kernel index is greater than our set of functions, we are done!
      return { code: TX_PROVING_CODE.COMPLETED, function: undefined };
    }

    // There is more work to do, are we ready?
    const nextFunction = this.publicFunctions[nextKernelIndex];

    // pass both the proof and verification key forward to the next circuit
    nextFunction.publicKernelRequest.inputs.previousKernel.proof = proof;
    nextFunction.publicKernelRequest.inputs.previousKernel.vk = verificationKey;

    // We need to update this so the state machine knows this proof is ready
    nextFunction.previousKernelProven = true;
    nextFunction.previousProofType = kernelRequest.type;
    if (nextFunction.vmProof === undefined) {
      // The VM proof for the next function is not ready
      return { code: TX_PROVING_CODE.NOT_READY, function: undefined };
    }

    // The VM proof is ready, we can continue
    return { code: TX_PROVING_CODE.READY, function: nextFunction };
  }

  // Updates the transaction's proving state after completion of a VM proof
  // Returns an instruction as to the next stage of tx proving
  public getNextPublicKernelFromVMProof(provenIndex: number, proof: Proof): TxProvingInstruction {
    const provenFunction = this.publicFunctions[provenIndex];
    provenFunction.vmProof = proof;

    if (!provenFunction.previousKernelProven) {
      // The previous kernel is not yet ready
      return { code: TX_PROVING_CODE.NOT_READY, function: undefined };
    }
    // The previous kernel is ready so we can prove this kernel
    return { code: TX_PROVING_CODE.READY, function: provenFunction };
  }

  // Returns the public function state at the given index
  // Throws if out of bounds
  public getPublicFunctionState(functionIndex: number) {
    if (functionIndex < 0 || functionIndex >= this.publicFunctions.length) {
      throw new Error(`Requested public function index was out of bounds`);
    }
    return this.publicFunctions[functionIndex];
  }

  // Returns the number of public kernels required by this transaction
  public getNumPublicKernels() {
    return this.publicFunctions.length;
  }
}
