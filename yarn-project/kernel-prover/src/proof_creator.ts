import { pedersenCompressWithHashIndex } from '@aztec/barretenberg.js/crypto';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import {
  CircuitsWasm,
  PreviousKernelData,
  PrivateCallData,
  PrivateCircuitPublicInputs,
  KernelCircuitPublicInputs,
  SignedTxRequest,
  UInt8Vector,
  makeEmptyProof,
  privateKernelSim,
} from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation';
import { createDebugLogger } from '@aztec/foundation/log';

export interface ProofOutput {
  publicInputs: KernelCircuitPublicInputs;
  proof: UInt8Vector;
}

export interface ProofCreator {
  getSiloedCommitments(publicInputs: PrivateCircuitPublicInputs): Promise<Fr[]>;
  createProof(
    signedTxRequest: SignedTxRequest,
    previousKernelData: PreviousKernelData,
    privateCallData: PrivateCallData,
    firstIteration: boolean,
  ): Promise<ProofOutput>;
}

const OUTER_COMMITMENT = 3;

export class KernelProofCreator {
  constructor(private log = createDebugLogger('aztec:kernel_proof_creator')) {}

  public async getSiloedCommitments(publicInputs: PrivateCircuitPublicInputs) {
    const bbWasm = await BarretenbergWasm.get();
    const contractAddress = publicInputs.callContext.storageContractAddress.toBuffer();
    // TODO
    // Should match `add_contract_address_to_commitment` in hash.hpp.
    return publicInputs.newCommitments.map(commitment =>
      Fr.fromBuffer(pedersenCompressWithHashIndex(bbWasm, [contractAddress, commitment.toBuffer()], OUTER_COMMITMENT)),
    );
  }

  public async createProof(
    signedTxRequest: SignedTxRequest,
    previousKernelData: PreviousKernelData,
    privateCallData: PrivateCallData,
    firstIteration: boolean,
  ): Promise<ProofOutput> {
    const wasm = await CircuitsWasm.get();
    this.log('Executing private kernel simulation...');
    const publicInputs = await privateKernelSim(
      wasm,
      signedTxRequest,
      previousKernelData,
      privateCallData,
      firstIteration,
    );
    this.log('Skipping private kernel proving...');
    // TODO
    const proof = makeEmptyProof();
    this.log('Kernel Prover Completed!');

    return {
      publicInputs,
      proof,
    };
  }
}
