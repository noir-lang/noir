import { ExecutionResult } from '@aztec/acir-simulator';
import {
  AccumulatedData,
  AggregationObject,
  ConstantData,
  Fr,
  NewContractData,
  OldTreeRoots,
  PrivateKernelPublicInputs,
} from '@aztec/circuits.js';
import { Signature, TxRequest } from './circuits.js';
export class KernelProver {
  prove(
    txRequest: TxRequest,
    txSignature: Signature,
    executionResult: ExecutionResult,
    oldRoots: OldTreeRoots,
  ): Promise<{ publicInputs: PrivateKernelPublicInputs; proof: Buffer }> {
    // TODO: implement this
    const newContracts = [];
    if (txRequest.functionData.isConstructor) {
      newContracts.push(
        new NewContractData(
          txRequest.to,
          txRequest.txContext.contractDeploymentData.portalContractAddress,
          txRequest.txContext.contractDeploymentData.functionTreeRoot,
        ),
      );
    }
    const accumulatedTxData = new AccumulatedData(
      AggregationObject.fromBuffer(Buffer.alloc(0)), // TODO - Fix this.
      new Fr(Buffer.from([1])),
      [], // newCommitments
      [], // newNullifiers
      [], // privateCallStack
      [], // publicCallStack
      [], // l1MsgStack
      newContracts,
      [], // optionallyRevealedData
    );

    const publicInputs = new PrivateKernelPublicInputs(
      accumulatedTxData,
      new ConstantData(oldRoots, txRequest.txContext),
      true,
    );

    return Promise.resolve({
      publicInputs,
      proof: Buffer.alloc(0),
    });
  }
}
