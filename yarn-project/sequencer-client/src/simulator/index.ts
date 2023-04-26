import {
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  EthAddress,
  MergeRollupInputs,
  PublicCircuitPublicInputs,
  PublicKernelInputsNoPreviousKernel,
  PublicKernelInputs,
  PublicKernelPublicInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
  TxRequest,
} from '@aztec/circuits.js';

export interface RollupSimulator {
  baseRollupCircuit(input: BaseRollupInputs): Promise<BaseOrMergeRollupPublicInputs>;
  mergeRollupCircuit(input: MergeRollupInputs): Promise<BaseOrMergeRollupPublicInputs>;
  rootRollupCircuit(input: RootRollupInputs): Promise<RootRollupPublicInputs>;
}

export interface PublicCircuitSimulator {
  publicCircuit(tx: TxRequest, functionBytecode: Buffer, portalAddress: EthAddress): Promise<PublicCircuitPublicInputs>;
}

export interface PublicKernelCircuitSimulator {
  publicKernelCircuitNoInput(inputs: PublicKernelInputsNoPreviousKernel): Promise<PublicKernelPublicInputs>;
  publicKernelCircuitPrivateInput(inputs: PublicKernelInputs): Promise<PublicKernelPublicInputs>;
  publicKernelCircuitNonFirstIteration(inputs: PublicKernelInputs): Promise<PublicKernelPublicInputs>;
}
