import { ExecutionPreimages } from './acvm.js';
import {
  CallContext,
  PrivateCircuitPublicInputs,
  TxRequest,
  EthAddress,
  PrivateCallStackItem,
  OldTreeRoots,
} from './circuits.js';

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
      request.functionData.isContructor,
    );

    const publicInputs = new PrivateCircuitPublicInputs(
      callContext,
      request.args,
      [], // returnValues,
      [], // emittedEvents,
      [], // newCommitments,
      [], // newNullifiers,
      [], // privateCallStack,
      [], // publicCallStack,
      [], // l1MsgStack,
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
