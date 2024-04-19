import { type MerkleTreeId, type ProcessedTx, type PublicKernelRequest, PublicKernelType } from '@aztec/circuit-types';
import { type AppendOnlyTreeSnapshot, type BaseRollupInputs, type Proof } from '@aztec/circuits.js';

import { type ProvingRequest, ProvingRequestType } from '../prover-pool/proving-request.js';

export enum TX_PROVING_CODE {
  NOT_READY,
  READY,
  COMPLETED,
}

export type PublicFunction = {
  vmProof: Proof | undefined;
  previousProofType: PublicKernelType;
  previousKernelProof: Proof | undefined;
  publicKernelRequest: PublicKernelRequest;
  provingRequest: ProvingRequest;
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
  ) {
    let previousKernelProof: Proof | undefined = processedTx.proof;
    let previousProofType = PublicKernelType.NON_PUBLIC;
    for (const kernelRequest of processedTx.publicKernelRequests) {
      const provingRequest: ProvingRequest =
        kernelRequest.type === PublicKernelType.TAIL
          ? {
              type: ProvingRequestType.PUBLIC_KERNEL_TAIL,
              kernelType: kernelRequest.type,
              inputs: kernelRequest.inputs,
            }
          : {
              type: ProvingRequestType.PUBLIC_KERNEL_NON_TAIL,
              kernelType: kernelRequest.type,
              inputs: kernelRequest.inputs,
            };
      const publicFunction: PublicFunction = {
        vmProof: undefined,
        previousProofType,
        previousKernelProof,
        publicKernelRequest: kernelRequest,
        provingRequest,
      };
      this.publicFunctions.push(publicFunction);
      previousKernelProof = undefined;
      previousProofType = kernelRequest.type;
    }
  }

  // Updates the transaction's proving state after completion of a kernel proof
  // Returns an instruction as to the next stage of tx proving
  public getNextPublicKernelFromKernelProof(provenIndex: number, proof: Proof): TxProvingInstruction {
    const kernelRequest = this.getPublicFunctionState(provenIndex).publicKernelRequest;
    const nextKernelIndex = provenIndex + 1;
    if (nextKernelIndex >= this.publicFunctions.length) {
      // The next kernel index is greater than our set of functions, we are done!
      return { code: TX_PROVING_CODE.COMPLETED, function: undefined };
    }

    // There is more work to do, are we ready?
    const nextFunction = this.publicFunctions[nextKernelIndex];
    nextFunction.previousKernelProof = proof;
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

    if (provenFunction.previousKernelProof === undefined) {
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
