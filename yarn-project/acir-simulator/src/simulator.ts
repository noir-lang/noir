import { ExecutionPreimages } from './acvm.js';
import {
  CallContext,
  PrivateCircuitPublicInputs,
  EthAddress,
  OldTreeRoots,
  Fr,
  ARGS_LENGTH,
  EMITTED_EVENTS_LENGTH,
  NEW_COMMITMENTS_LENGTH,
  NEW_NULLIFIERS_LENGTH,
  PRIVATE_CALL_STACK_LENGTH,
  PUBLIC_CALL_STACK_LENGTH,
  L1_MSG_STACK_LENGTH,
  RETURN_VALUES_LENGTH,
  TxRequest,
} from '@aztec/circuits.js';
import { PrivateCallStackItem } from './db_oracle.js';

export interface ExecutionResult {
  // Needed for prover
  acir: Buffer;
  partialWitness: Buffer;
  // Needed for the verifier (kernel)
  callStackItem: PrivateCallStackItem;
  // Needed for the user
  preimages: ExecutionPreimages;
  // Nested executions
  nestedExecutions: this[];
}

/**
 * A placeholder for the Acir Simulator.
 */
export class AcirSimulator {
  run(
    request: TxRequest,
    entryPointACIR: Buffer,
    portalContractAddress: EthAddress,
    oldRoots: OldTreeRoots,
  ): Promise<ExecutionResult> {
    const callContext = new CallContext(
      request.from,
      request.to,
      portalContractAddress,
      false,
      false,
      request.functionData.isConstructor,
    );

    const randomFields = (num: number) => {
      return Array(num)
        .fill(0)
        .map(() => new Fr(0n));
    };

    const publicInputs = new PrivateCircuitPublicInputs(
      callContext,
      request.args.concat(randomFields(ARGS_LENGTH - request.args.length)),
      randomFields(RETURN_VALUES_LENGTH), // returnValues,
      randomFields(EMITTED_EVENTS_LENGTH), // emittedEvents,
      randomFields(NEW_COMMITMENTS_LENGTH), // newCommitments,
      randomFields(NEW_NULLIFIERS_LENGTH), // newNullifiers,
      randomFields(PRIVATE_CALL_STACK_LENGTH), // privateCallStack,
      randomFields(PUBLIC_CALL_STACK_LENGTH), // publicCallStack,
      randomFields(L1_MSG_STACK_LENGTH), // l1MsgStack,
      oldRoots.privateDataTreeRoot,
      oldRoots.nullifierTreeRoot,
      oldRoots.contractTreeRoot,
      request.txContext.contractDeploymentData,
    );

    return Promise.resolve({
      acir: entryPointACIR,
      partialWitness: Buffer.alloc(0),
      callStackItem: new PrivateCallStackItem(request.to, request.functionData.functionSelector, publicInputs),
      preimages: {
        newNotes: [],
        nullifiedNotes: [],
      },
      nestedExecutions: [],
    });
  }
}
