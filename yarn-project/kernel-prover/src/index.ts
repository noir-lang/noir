import { ExecutionResult } from '@aztec/acir-simulator';
import {
  AccumulatedData,
  AffineElement,
  AggregationObject,
  AztecAddress,
  ConstantData,
  EMITTED_EVENTS_LENGTH,
  EthAddress,
  Fq,
  Fr,
  FunctionData,
  KERNEL_L1_MSG_STACK_LENGTH,
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_CONTRACTS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH,
  KERNEL_PRIVATE_CALL_STACK_LENGTH,
  KERNEL_PUBLIC_CALL_STACK_LENGTH,
  NewContractData,
  OldTreeRoots,
  OptionallyRevealedData,
  PrivateKernelPublicInputs,
  TxRequest,
} from '@aztec/circuits.js';
import { randomBytes } from 'crypto';

export class KernelProver {
  prove(
    txRequest: TxRequest,
    txSignature: unknown,
    executionResult: ExecutionResult,
    oldRoots: OldTreeRoots,
  ): Promise<{ publicInputs: PrivateKernelPublicInputs; proof: Buffer }> {
    // TODO: implement this
    const createRandomFields = (num: number) => {
      return Array(num)
        .fill(0)
        .map(() => Fr.random());
    };
    const createRandomContractData = () => {
      return new NewContractData(AztecAddress.random(), new EthAddress(randomBytes(20)), createRandomFields(1)[0]);
    };
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
    newContracts.push(
      ...Array(KERNEL_NEW_CONTRACTS_LENGTH - newContracts.length)
        .fill(0)
        .map(() => createRandomContractData()),
    );

    const aggregationObject = new AggregationObject(
      new AffineElement(new Fq(0n), new Fq(0n)),
      new AffineElement(new Fq(0n), new Fq(0n)),
      [],
      [],
      false,
    );
    const createOptionallyRevealedData = () => {
      const optionallyRevealedData = new OptionallyRevealedData(
        createRandomFields(1)[0],
        new FunctionData(1, true, true),
        createRandomFields(EMITTED_EVENTS_LENGTH),
        createRandomFields(1)[0],
        new EthAddress(randomBytes(20)),
        true,
        true,
        true,
        true,
      );
      return optionallyRevealedData;
    };
    const accumulatedTxData = new AccumulatedData(
      aggregationObject,
      new Fr(0n),
      createRandomFields(KERNEL_NEW_COMMITMENTS_LENGTH),
      createRandomFields(KERNEL_NEW_NULLIFIERS_LENGTH),
      createRandomFields(KERNEL_PRIVATE_CALL_STACK_LENGTH),
      createRandomFields(KERNEL_PUBLIC_CALL_STACK_LENGTH),
      createRandomFields(KERNEL_L1_MSG_STACK_LENGTH),
      newContracts,
      Array(KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH)
        .fill(0)
        .map(() => createOptionallyRevealedData()),
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
