import {
  CallRequest,
  Fr,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
  PrivateCallData,
  PrivateKernelCircuitPublicInputs,
  PrivateKernelData,
  PrivateKernelInitCircuitPrivateInputs,
  PrivateKernelInnerCircuitPrivateInputs,
  PrivateKernelTailCircuitPrivateInputs,
  type PrivateKernelTailCircuitPublicInputs,
  type Proof,
  type TxRequest,
  VK_TREE_HEIGHT,
  VerificationKey,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { padArrayEnd } from '@aztec/foundation/collection';
import { createDebugLogger } from '@aztec/foundation/log';
import { assertLength } from '@aztec/foundation/serialize';
import { pushTestData } from '@aztec/foundation/testing';
import { type ExecutionResult, collectNoteHashLeafIndexMap, collectNullifiedNoteHashCounters } from '@aztec/simulator';

import { type ProofCreator, type ProofOutput } from './interface/proof_creator.js';
import {
  buildPrivateKernelInnerHints,
  buildPrivateKernelTailHints,
  buildPrivateKernelTailOutputs,
} from './private_inputs_builders/index.js';
import { type ProvingDataOracle } from './proving_data_oracle.js';

/**
 * The KernelProver class is responsible for generating kernel proofs.
 * It takes a transaction request, its signature, and the simulation result as inputs, and outputs a proof
 * along with output notes. The class interacts with a ProvingDataOracle to fetch membership witnesses and
 * constructs private call data based on the execution results.
 */
export class KernelProver {
  private log = createDebugLogger('aztec:kernel-prover');

  constructor(private oracle: ProvingDataOracle, private proofCreator: ProofCreator) {}

  /**
   * Generate a proof for a given transaction request and execution result.
   * The function iterates through the nested executions in the execution result, creates private call data,
   * and generates a proof using the provided ProofCreator instance. It also maintains an index of new notes
   * created during the execution and returns them as a part of the KernelProverOutput.
   *
   * @param txRequest - The authenticated transaction request object.
   * @param executionResult - The execution result object containing nested executions and preimages.
   * @returns A Promise that resolves to a KernelProverOutput object containing proof, public inputs, and output notes.
   */
  async prove(
    txRequest: TxRequest,
    executionResult: ExecutionResult,
  ): Promise<ProofOutput<PrivateKernelTailCircuitPublicInputs>> {
    const executionStack = [executionResult];
    let firstIteration = true;
    let previousVerificationKey = VerificationKey.makeFake();

    let output: ProofOutput<PrivateKernelCircuitPublicInputs> = {
      publicInputs: PrivateKernelCircuitPublicInputs.empty(),
      proof: makeEmptyProof(),
    };

    const noteHashLeafIndexMap = collectNoteHashLeafIndexMap(executionResult);
    const noteHashNullifierCounterMap = collectNullifiedNoteHashCounters(executionResult);

    while (executionStack.length) {
      const currentExecution = executionStack.pop()!;
      executionStack.push(...currentExecution.nestedExecutions);

      const privateCallRequests = currentExecution.nestedExecutions.map(result =>
        result.callStackItem.toCallRequest(currentExecution.callStackItem.publicInputs.callContext),
      );
      const publicCallRequests = currentExecution.enqueuedPublicFunctionCalls.map(result => result.toCallRequest());

      const proof = await this.proofCreator.createAppCircuitProof(
        currentExecution.partialWitness,
        currentExecution.acir,
      );

      const privateCallData = await this.createPrivateCallData(
        currentExecution,
        privateCallRequests,
        publicCallRequests,
        proof,
      );

      const hints = buildPrivateKernelInnerHints(
        currentExecution.callStackItem.publicInputs,
        noteHashNullifierCounterMap,
      );

      if (firstIteration) {
        const proofInput = new PrivateKernelInitCircuitPrivateInputs(txRequest, privateCallData, hints);
        pushTestData('private-kernel-inputs-init', proofInput);
        output = await this.proofCreator.createProofInit(proofInput);
      } else {
        const previousVkMembershipWitness = await this.oracle.getVkMembershipWitness(previousVerificationKey);
        const previousKernelData = new PrivateKernelData(
          output.publicInputs,
          output.proof,
          previousVerificationKey,
          Number(previousVkMembershipWitness.leafIndex),
          assertLength<Fr, typeof VK_TREE_HEIGHT>(previousVkMembershipWitness.siblingPath, VK_TREE_HEIGHT),
        );
        const proofInput = new PrivateKernelInnerCircuitPrivateInputs(previousKernelData, privateCallData, hints);
        pushTestData('private-kernel-inputs-inner', proofInput);
        output = await this.proofCreator.createProofInner(proofInput);
      }
      firstIteration = false;
      previousVerificationKey = privateCallData.vk;
    }

    const previousVkMembershipWitness = await this.oracle.getVkMembershipWitness(previousVerificationKey);
    const previousKernelData = new PrivateKernelData(
      output.publicInputs,
      output.proof,
      previousVerificationKey,
      Number(previousVkMembershipWitness.leafIndex),
      assertLength<Fr, typeof VK_TREE_HEIGHT>(previousVkMembershipWitness.siblingPath, VK_TREE_HEIGHT),
    );

    this.log.debug(
      `Calling private kernel tail with hwm ${previousKernelData.publicInputs.minRevertibleSideEffectCounter}`,
    );

    const hints = await buildPrivateKernelTailHints(output.publicInputs, noteHashLeafIndexMap, this.oracle);

    const expectedOutputs = buildPrivateKernelTailOutputs(hints.sortedNewNoteHashes, hints.sortedNewNullifiers);

    const privateInputs = new PrivateKernelTailCircuitPrivateInputs(previousKernelData, expectedOutputs, hints);

    pushTestData('private-kernel-inputs-ordering', privateInputs);
    return await this.proofCreator.createProofTail(privateInputs);
  }

  private async createPrivateCallData(
    { callStackItem, vk }: ExecutionResult,
    privateCallRequests: CallRequest[],
    publicCallRequests: CallRequest[],
    proof: Proof,
  ) {
    const { contractAddress, functionData } = callStackItem;

    // Pad with empty items to reach max/const length expected by circuit.
    const privateCallStack = padArrayEnd(
      privateCallRequests,
      CallRequest.empty(),
      MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
    );
    const publicCallStack = padArrayEnd(publicCallRequests, CallRequest.empty(), MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL);

    const functionLeafMembershipWitness = await this.oracle.getFunctionMembershipWitness(
      contractAddress,
      functionData.selector,
    );
    const { contractClassId, publicKeysHash, saltedInitializationHash } = await this.oracle.getContractAddressPreimage(
      contractAddress,
    );
    const { artifactHash: contractClassArtifactHash, publicBytecodeCommitment: contractClassPublicBytecodeCommitment } =
      await this.oracle.getContractClassIdPreimage(contractClassId);

    // TODO(#262): Use real acir hash
    // const acirHash = keccak256(Buffer.from(bytecode, 'hex'));
    const acirHash = Fr.fromBuffer(Buffer.alloc(32, 0));

    return PrivateCallData.from({
      callStackItem,
      privateCallStack,
      publicCallStack,
      proof,
      vk: VerificationKey.fromBuffer(vk),
      publicKeysHash,
      contractClassArtifactHash,
      contractClassPublicBytecodeCommitment,
      saltedInitializationHash,
      functionLeafMembershipWitness,
      acirHash,
    });
  }
}
