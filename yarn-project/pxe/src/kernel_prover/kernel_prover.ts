import { type KernelProofOutput, type ProofCreator } from '@aztec/circuit-types';
import {
  CallRequest,
  Fr,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
  NESTED_RECURSIVE_PROOF_LENGTH,
  PrivateCallData,
  PrivateKernelCircuitPublicInputs,
  PrivateKernelData,
  PrivateKernelInitCircuitPrivateInputs,
  PrivateKernelInnerCircuitPrivateInputs,
  PrivateKernelResetCircuitPrivateInputs,
  PrivateKernelTailCircuitPrivateInputs,
  type PrivateKernelTailCircuitPublicInputs,
  type RECURSIVE_PROOF_LENGTH,
  type RecursiveProof,
  type TxRequest,
  VK_TREE_HEIGHT,
  VerificationKeyAsFields,
  makeRecursiveProof,
} from '@aztec/circuits.js';
import { padArrayEnd } from '@aztec/foundation/collection';
import { createDebugLogger } from '@aztec/foundation/log';
import { assertLength } from '@aztec/foundation/serialize';
import { pushTestData } from '@aztec/foundation/testing';
import { type ExecutionResult, collectNoteHashLeafIndexMap, collectNullifiedNoteHashCounters } from '@aztec/simulator';

import {
  buildPrivateKernelInitHints,
  buildPrivateKernelInnerHints,
  buildPrivateKernelResetHints,
  buildPrivateKernelResetOutputs,
  buildPrivateKernelTailHints,
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
  ): Promise<KernelProofOutput<PrivateKernelTailCircuitPublicInputs>> {
    const executionStack = [executionResult];
    let firstIteration = true;

    let output: KernelProofOutput<PrivateKernelCircuitPublicInputs> = {
      publicInputs: PrivateKernelCircuitPublicInputs.empty(),
      proof: makeRecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>(NESTED_RECURSIVE_PROOF_LENGTH),
      verificationKey: VerificationKeyAsFields.makeEmpty(),
    };

    const noteHashLeafIndexMap = collectNoteHashLeafIndexMap(executionResult);
    const noteHashNullifierCounterMap = collectNullifiedNoteHashCounters(executionResult);

    while (executionStack.length) {
      const currentExecution = executionStack.pop()!;
      executionStack.push(...[...currentExecution.nestedExecutions].reverse());

      const publicCallRequests = currentExecution.enqueuedPublicFunctionCalls.map(result => result.toCallRequest());
      const publicTeardownCallRequest = currentExecution.publicTeardownFunctionCall.isEmpty()
        ? CallRequest.empty()
        : currentExecution.publicTeardownFunctionCall.toCallRequest();

      const functionName = await this.oracle.getFunctionName(
        currentExecution.callStackItem.contractAddress,
        currentExecution.callStackItem.functionData.selector,
      );

      const proofOutput = await this.proofCreator.createAppCircuitProof(
        currentExecution.partialWitness,
        currentExecution.acir,
        functionName,
      );

      const privateCallData = await this.createPrivateCallData(
        currentExecution,
        publicCallRequests,
        publicTeardownCallRequest,
        proofOutput.proof,
        proofOutput.verificationKey,
      );

      if (firstIteration) {
        const hints = buildPrivateKernelInitHints(
          currentExecution.callStackItem.publicInputs,
          noteHashNullifierCounterMap,
          currentExecution.callStackItem.publicInputs.privateCallRequests,
        );
        const proofInput = new PrivateKernelInitCircuitPrivateInputs(txRequest, privateCallData, hints);
        pushTestData('private-kernel-inputs-init', proofInput);
        output = await this.proofCreator.createProofInit(proofInput);
      } else {
        const hints = buildPrivateKernelInnerHints(
          currentExecution.callStackItem.publicInputs,
          noteHashNullifierCounterMap,
        );
        const previousVkMembershipWitness = await this.oracle.getVkMembershipWitness(output.verificationKey);
        const previousKernelData = new PrivateKernelData(
          output.publicInputs,
          output.proof,
          output.verificationKey,
          Number(previousVkMembershipWitness.leafIndex),
          assertLength<Fr, typeof VK_TREE_HEIGHT>(previousVkMembershipWitness.siblingPath, VK_TREE_HEIGHT),
        );
        const proofInput = new PrivateKernelInnerCircuitPrivateInputs(previousKernelData, privateCallData, hints);
        pushTestData('private-kernel-inputs-inner', proofInput);
        output = await this.proofCreator.createProofInner(proofInput);
      }
      firstIteration = false;
    }

    let previousVkMembershipWitness = await this.oracle.getVkMembershipWitness(output.verificationKey);
    let previousKernelData = new PrivateKernelData(
      output.publicInputs,
      output.proof,
      output.verificationKey,
      Number(previousVkMembershipWitness.leafIndex),
      assertLength<Fr, typeof VK_TREE_HEIGHT>(previousVkMembershipWitness.siblingPath, VK_TREE_HEIGHT),
    );

    const expectedOutputs = buildPrivateKernelResetOutputs(
      output.publicInputs.end.newNoteHashes,
      output.publicInputs.end.newNullifiers,
      output.publicInputs.end.noteEncryptedLogsHashes,
    );

    output = await this.proofCreator.createProofReset(
      new PrivateKernelResetCircuitPrivateInputs(
        previousKernelData,
        expectedOutputs,
        await buildPrivateKernelResetHints(output.publicInputs, noteHashLeafIndexMap, this.oracle),
      ),
    );

    previousVkMembershipWitness = await this.oracle.getVkMembershipWitness(output.verificationKey);
    previousKernelData = new PrivateKernelData(
      output.publicInputs,
      output.proof,
      output.verificationKey,
      Number(previousVkMembershipWitness.leafIndex),
      assertLength<Fr, typeof VK_TREE_HEIGHT>(previousVkMembershipWitness.siblingPath, VK_TREE_HEIGHT),
    );

    this.log.debug(
      `Calling private kernel tail with hwm ${previousKernelData.publicInputs.minRevertibleSideEffectCounter}`,
    );

    const hints = buildPrivateKernelTailHints(output.publicInputs);

    const privateInputs = new PrivateKernelTailCircuitPrivateInputs(previousKernelData, hints);

    pushTestData('private-kernel-inputs-ordering', privateInputs);
    return await this.proofCreator.createProofTail(privateInputs);
  }

  private async createPrivateCallData(
    { callStackItem }: ExecutionResult,
    publicCallRequests: CallRequest[],
    publicTeardownCallRequest: CallRequest,
    proof: RecursiveProof<typeof RECURSIVE_PROOF_LENGTH>,
    vk: VerificationKeyAsFields,
  ) {
    const { contractAddress, functionData } = callStackItem;

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
      publicCallStack,
      publicTeardownCallRequest,
      proof,
      vk,
      publicKeysHash,
      contractClassArtifactHash,
      contractClassPublicBytecodeCommitment,
      saltedInitializationHash,
      functionLeafMembershipWitness,
      acirHash,
    });
  }
}
