import { ExecutionResult, NewNoteData } from '@aztec/acir-simulator';
import {
  CONTRACT_TREE_HEIGHT,
  EcdsaSignature,
  MembershipWitness,
  PRIVATE_CALL_STACK_LENGTH,
  PreviousKernelData,
  PrivateCallData,
  PrivateCallStackItem,
  KernelCircuitPublicInputs,
  SignedTxRequest,
  TxRequest,
  VK_TREE_HEIGHT,
  VerificationKey,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { AztecAddress, Fr } from '@aztec/foundation';
import { KernelProofCreator, ProofCreator, ProofOutput } from './proof_creator.js';
import { ProvingDataOracle } from './proving_data_oracle.js';

export interface OutputNoteData {
  contractAddress: AztecAddress;
  data: NewNoteData;
  commitment: Fr;
}

export interface KernelProverOutput extends ProofOutput {
  outputNotes: OutputNoteData[];
}

export class KernelProver {
  constructor(private oracle: ProvingDataOracle, private proofCreator: ProofCreator = new KernelProofCreator()) {}

  async prove(
    txRequest: TxRequest,
    txSignature: EcdsaSignature,
    executionResult: ExecutionResult,
  ): Promise<KernelProverOutput> {
    const signedTxRequest = new SignedTxRequest(txRequest, txSignature);
    const executionStack = [executionResult];
    const newNotes: { [commitmentStr: string]: OutputNoteData } = {};
    let firstIteration = true;
    let previousVerificationKey = VerificationKey.makeFake();
    let output: ProofOutput = {
      publicInputs: KernelCircuitPublicInputs.empty(),
      proof: makeEmptyProof(),
    };
    while (executionStack.length) {
      const previousVkMembershipWitness = firstIteration
        ? MembershipWitness.random(VK_TREE_HEIGHT)
        : await this.oracle.getVkMembershipWitness(previousVerificationKey);
      const previousKernelData = new PreviousKernelData(
        output.publicInputs,
        output.proof,
        previousVerificationKey,
        previousVkMembershipWitness.leafIndex,
        previousVkMembershipWitness.siblingPath,
      );

      const currentExecution = executionStack.pop()!;
      executionStack.push(...currentExecution.nestedExecutions);
      const privateCallStackPreimages = currentExecution.nestedExecutions.map(result => result.callStackItem);
      if (privateCallStackPreimages.length > PRIVATE_CALL_STACK_LENGTH) {
        throw new Error(
          `Too many items in the call stack. Maximum amount is ${PRIVATE_CALL_STACK_LENGTH}. Got ${privateCallStackPreimages.length}.`,
        );
      }
      privateCallStackPreimages.push(
        ...Array(PRIVATE_CALL_STACK_LENGTH - privateCallStackPreimages.length)
          .fill(0)
          .map(() => PrivateCallStackItem.empty()),
      );

      const privateCallData = await this.createPrivateCallData(currentExecution, privateCallStackPreimages);

      output = await this.proofCreator.createProof(
        signedTxRequest,
        previousKernelData,
        privateCallData,
        firstIteration,
      );
      (await this.getNewNotes(currentExecution)).forEach(n => {
        newNotes[n.commitment.toString()] = n;
      });
      firstIteration = false;
      previousVerificationKey = privateCallData.vk;
    }

    // Only return the notes whose commitment is in the commitments of the final proof.
    const finalNewCommitments = output.publicInputs.end.newCommitments;
    const outputNotes = finalNewCommitments.map(c => newNotes[c.toString()]).filter(c => !!c);

    return { ...output, outputNotes };
  }

  private async createPrivateCallData(
    { callStackItem, vk }: ExecutionResult,
    privateCallStackPreimages: PrivateCallStackItem[],
  ) {
    const { contractAddress, functionData, publicInputs } = callStackItem;
    const { portalContractAddress } = publicInputs.callContext;

    const contractLeafMembershipWitness = functionData.isConstructor
      ? MembershipWitness.random(CONTRACT_TREE_HEIGHT)
      : await this.oracle.getContractMembershipWitness(contractAddress);

    const functionLeafMembershipWitness = await this.oracle.getFunctionMembershipWitness(
      contractAddress,
      functionData.functionSelector,
    );

    // TODO
    // FIXME: https://github.com/AztecProtocol/aztec3-packages/issues/262
    // const acirHash = keccak(Buffer.from(bytecode, 'hex'));
    const acirHash = Fr.fromBuffer(Buffer.alloc(32, 0));

    // TODO
    const proof = makeEmptyProof();

    return new PrivateCallData(
      callStackItem,
      privateCallStackPreimages,
      proof,
      VerificationKey.fromBuffer(vk),
      functionLeafMembershipWitness,
      contractLeafMembershipWitness,
      portalContractAddress,
      acirHash,
    );
  }

  private async getNewNotes(executionResult: ExecutionResult): Promise<OutputNoteData[]> {
    const {
      callStackItem: { publicInputs },
      preimages,
    } = executionResult;
    const contractAddress = publicInputs.callContext.storageContractAddress;
    // Assuming that for each new commitment there's an output note added to the execution result.
    const newCommitments = await this.proofCreator.getSiloedCommitments(publicInputs);
    return preimages.newNotes.map((data, i) => ({
      contractAddress,
      data,
      commitment: newCommitments[i],
    }));
  }
}
