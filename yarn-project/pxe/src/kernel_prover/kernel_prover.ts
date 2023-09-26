import { ExecutionResult, NewNoteData } from '@aztec/acir-simulator';
import {
  AztecAddress,
  CONTRACT_TREE_HEIGHT,
  EMPTY_NULLIFIED_COMMITMENT,
  Fr,
  MAX_NEW_COMMITMENTS_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
  MAX_READ_REQUESTS_PER_CALL,
  MAX_READ_REQUESTS_PER_TX,
  MembershipWitness,
  PreviousKernelData,
  PrivateCallData,
  PrivateCallStackItem,
  PrivateKernelInputsInit,
  PrivateKernelInputsInner,
  PrivateKernelInputsOrdering,
  PrivateKernelPublicInputs,
  ReadRequestMembershipWitness,
  TxRequest,
  VK_TREE_HEIGHT,
  VerificationKey,
  makeEmptyProof,
  makeTuple,
} from '@aztec/circuits.js';
import { Tuple, assertLength } from '@aztec/foundation/serialize';

import { KernelProofCreator, ProofCreator, ProofOutput, ProofOutputFinal } from './proof_creator.js';
import { ProvingDataOracle } from './proving_data_oracle.js';

/**
 * Represents an output note data object.
 * Contains the contract address, new note data and commitment for the note,
 * resulting from the execution of a transaction in the Aztec network.
 */
export interface OutputNoteData {
  /**
   * The address of the contract the note was created in.
   */
  contractAddress: AztecAddress;
  /**
   * The encrypted note data for an output note.
   */
  data: NewNoteData;
  /**
   * The unique value representing the note.
   */
  commitment: Fr;
}

/**
 * Represents the output data of the Kernel Prover.
 * Provides information about the newly created notes, along with the public inputs and proof.
 */
export interface KernelProverOutput extends ProofOutputFinal {
  /**
   * An array of output notes containing the contract address, note data, and commitment for each new note.
   */
  outputNotes: OutputNoteData[];
}

/**
 * The KernelProver class is responsible for generating kernel proofs.
 * It takes a transaction request, its signature, and the simulation result as inputs, and outputs a proof
 * along with output notes. The class interacts with a ProvingDataOracle to fetch membership witnesses and
 * constructs private call data based on the execution results.
 */
export class KernelProver {
  constructor(private oracle: ProvingDataOracle, private proofCreator: ProofCreator = new KernelProofCreator()) {}

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
  async prove(txRequest: TxRequest, executionResult: ExecutionResult): Promise<KernelProverOutput> {
    const executionStack = [executionResult];
    const newNotes: { [commitmentStr: string]: OutputNoteData } = {};
    let firstIteration = true;
    let previousVerificationKey = VerificationKey.makeFake();

    let output: ProofOutput = {
      publicInputs: PrivateKernelPublicInputs.empty(),
      proof: makeEmptyProof(),
    };

    while (executionStack.length) {
      const currentExecution = executionStack.pop()!;
      executionStack.push(...currentExecution.nestedExecutions);
      const privateCallStackPreimages = currentExecution.nestedExecutions.map(result => result.callStackItem);
      if (privateCallStackPreimages.length > MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL) {
        throw new Error(
          `Too many items in the call stack. Maximum amount is ${MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL}. Got ${privateCallStackPreimages.length}.`,
        );
      }
      // Pad with empty items to reach max/const length expected by circuit.
      privateCallStackPreimages.push(
        ...Array(MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL - privateCallStackPreimages.length)
          .fill(0)
          .map(() => PrivateCallStackItem.empty()),
      );

      // Start with the partially filled in read request witnesses from the simulator
      // and fill the non-transient ones in with sibling paths via oracle.
      const readRequestMembershipWitnesses = currentExecution.readRequestPartialWitnesses;
      for (let rr = 0; rr < readRequestMembershipWitnesses.length; rr++) {
        if (currentExecution.callStackItem.publicInputs.readRequests[rr] == Fr.zero()) {
          throw new Error(
            'Number of read requests output from Noir circuit does not match number of read request commitment indices output from simulator.',
          );
        }
        const rrWitness = readRequestMembershipWitnesses[rr];
        if (!rrWitness.isTransient) {
          // Non-transient reads must contain full membership witness with sibling path from commitment to root.
          // Get regular membership witness to fill in sibling path in the read request witness.
          const membershipWitness = await this.oracle.getNoteMembershipWitness(rrWitness.leafIndex.toBigInt());
          rrWitness.siblingPath = membershipWitness.siblingPath;
        }
      }

      // fill in witnesses for remaining/empty read requests
      readRequestMembershipWitnesses.push(
        ...Array(MAX_READ_REQUESTS_PER_CALL - readRequestMembershipWitnesses.length)
          .fill(0)
          .map(() => ReadRequestMembershipWitness.empty(BigInt(0))),
      );

      const privateCallData = await this.createPrivateCallData(
        currentExecution,
        readRequestMembershipWitnesses,
        makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, i => privateCallStackPreimages[i], 0),
      );

      if (firstIteration) {
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/778): remove historic root
        // from app circuit public inputs and add it to PrivateCallData
        privateCallData.callStackItem.publicInputs.historicBlockData.privateDataTreeRoot =
          await this.oracle.getPrivateDataRoot();

        output = await this.proofCreator.createProofInit(new PrivateKernelInputsInit(txRequest, privateCallData));
      } else {
        const previousVkMembershipWitness = await this.oracle.getVkMembershipWitness(previousVerificationKey);
        const previousKernelData = new PreviousKernelData(
          output.publicInputs,
          output.proof,
          previousVerificationKey,
          Number(previousVkMembershipWitness.leafIndex),
          assertLength<Fr, typeof VK_TREE_HEIGHT>(previousVkMembershipWitness.siblingPath, VK_TREE_HEIGHT),
        );
        output = await this.proofCreator.createProofInner(
          new PrivateKernelInputsInner(previousKernelData, privateCallData),
        );
      }
      (await this.getNewNotes(currentExecution)).forEach(n => {
        newNotes[n.commitment.toString()] = n;
      });
      firstIteration = false;
      previousVerificationKey = privateCallData.vk;
    }

    const previousVkMembershipWitness = await this.oracle.getVkMembershipWitness(previousVerificationKey);
    const previousKernelData = new PreviousKernelData(
      output.publicInputs,
      output.proof,
      previousVerificationKey,
      Number(previousVkMembershipWitness.leafIndex),
      assertLength<Fr, typeof VK_TREE_HEIGHT>(previousVkMembershipWitness.siblingPath, VK_TREE_HEIGHT),
    );

    const readCommitmentHints = this.getReadRequestHints(
      output.publicInputs.end.readRequests,
      output.publicInputs.end.newCommitments,
    );

    const nullifierCommitmentHints = this.getNullifierHints(
      output.publicInputs.end.nullifiedCommitments,
      output.publicInputs.end.newCommitments,
    );

    const privateInputs = new PrivateKernelInputsOrdering(
      previousKernelData,
      readCommitmentHints,
      nullifierCommitmentHints,
    );
    const outputFinal = await this.proofCreator.createProofOrdering(privateInputs);

    // Only return the notes whose commitment is in the commitments of the final proof.
    const finalNewCommitments = outputFinal.publicInputs.end.newCommitments;
    const outputNotes = finalNewCommitments.map(c => newNotes[c.toString()]).filter(c => !!c);

    return { ...outputFinal, outputNotes };
  }

  private async createPrivateCallData(
    { callStackItem, vk }: ExecutionResult,
    readRequestMembershipWitnesses: ReadRequestMembershipWitness[],
    privateCallStackPreimages: Tuple<PrivateCallStackItem, typeof MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL>,
  ) {
    const { contractAddress, functionData, publicInputs } = callStackItem;
    const { portalContractAddress } = publicInputs.callContext;

    const contractLeafMembershipWitness = functionData.isConstructor
      ? MembershipWitness.random(CONTRACT_TREE_HEIGHT)
      : await this.oracle.getContractMembershipWitness(contractAddress);

    const functionLeafMembershipWitness = await this.oracle.getFunctionMembershipWitness(
      contractAddress,
      functionData.selector,
    );

    // TODO(#262): Use real acir hash
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
      makeTuple(MAX_READ_REQUESTS_PER_CALL, i => readRequestMembershipWitnesses[i], 0),
      portalContractAddress.toField(),
      acirHash,
    );
  }

  /**
   * Retrieves the new output notes for a given execution result.
   * The function maps over the new note preimages and associates them with their corresponding
   * commitments in the public inputs of the execution result. It also includes the contract address
   * from the call context of the public inputs.
   *
   * @param executionResult - The execution result object containing note preimages and public inputs.
   * @returns An array of OutputNoteData objects, each representing an output note with its associated data.
   */
  private async getNewNotes(executionResult: ExecutionResult): Promise<OutputNoteData[]> {
    const {
      callStackItem: { publicInputs },
      newNotes,
    } = executionResult;
    const contractAddress = publicInputs.callContext.storageContractAddress;
    // Assuming that for each new commitment there's an output note added to the execution result.
    const newCommitments = await this.proofCreator.getSiloedCommitments(publicInputs);
    return newNotes.map((data, i) => ({
      contractAddress,
      data,
      commitment: newCommitments[i],
    }));
  }

  /**
   * Performs the matching between an array of read request and an array of commitments. This produces
   * hints for the private kernel ordering circuit to efficiently match a read request with the corresponding
   * commitment.
   *
   * @param readRequests - The array of read requests.
   * @param commitments - The array of commitments.
   * @returns An array of hints where each element is the index of the commitment in commitments array
   *  corresponding to the read request. In other words we have readRequests[i] == commitments[hints[i]].
   */
  private getReadRequestHints(
    readRequests: Tuple<Fr, typeof MAX_READ_REQUESTS_PER_TX>,
    commitments: Tuple<Fr, typeof MAX_NEW_COMMITMENTS_PER_TX>,
  ): Tuple<Fr, typeof MAX_READ_REQUESTS_PER_TX> {
    const hints = makeTuple(MAX_READ_REQUESTS_PER_TX, Fr.zero);
    for (let i = 0; i < MAX_READ_REQUESTS_PER_TX && !readRequests[i].isZero(); i++) {
      const equalToRR = (cmt: Fr) => cmt.equals(readRequests[i]);
      const result = commitments.findIndex(equalToRR);
      if (result == -1) {
        throw new Error(
          `The read request at index ${i} with value ${readRequests[i].toString()} does not match to any commitment.`,
        );
      } else {
        hints[i] = new Fr(result);
      }
    }
    return hints;
  }

  /**
   *  Performs the matching between an array of nullified commitments and an array of commitments. This produces
   * hints for the private kernel ordering circuit to efficiently match a nullifier with the corresponding
   * commitment.
   *
   * @param nullifiedCommitments - The array of nullified commitments.
   * @param commitments - The array of commitments.
   * @returns An array of hints where each element is the index of the commitment in commitments array
   *  corresponding to the nullified commitments. In other words we have nullifiedCommitments[i] == commitments[hints[i]].
   */
  private getNullifierHints(
    nullifiedCommitments: Tuple<Fr, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    commitments: Tuple<Fr, typeof MAX_NEW_COMMITMENTS_PER_TX>,
  ): Tuple<Fr, typeof MAX_NEW_NULLIFIERS_PER_TX> {
    const hints = makeTuple(MAX_NEW_NULLIFIERS_PER_TX, Fr.zero);
    for (let i = 0; i < MAX_NEW_NULLIFIERS_PER_TX; i++) {
      if (!nullifiedCommitments[i].isZero() && !nullifiedCommitments[i].equals(new Fr(EMPTY_NULLIFIED_COMMITMENT))) {
        const equalToCommitment = (cmt: Fr) => cmt.equals(nullifiedCommitments[i]);
        const result = commitments.findIndex(equalToCommitment);
        if (result == -1) {
          throw new Error(
            `The nullified commitment at index ${i} with value ${nullifiedCommitments[
              i
            ].toString()} does not match to any commitment.`,
          );
        } else {
          hints[i] = new Fr(result);
        }
      }
    }
    return hints;
  }
}
