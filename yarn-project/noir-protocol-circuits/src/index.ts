import {
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  KernelCircuitPublicInputs,
  KernelCircuitPublicInputsFinal,
  MergeRollupInputs,
  PrivateKernelInputsInit,
  PrivateKernelInputsInner,
  PrivateKernelInputsOrdering,
  PublicKernelInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
} from '@aztec/circuits.js';
import { NoirCompiledCircuit } from '@aztec/types/noir';

import { WasmBlackBoxFunctionSolver, createBlackBoxSolver, executeCircuitWithBlackBoxSolver } from '@noir-lang/acvm_js';
import { Abi, abiDecode, abiEncode } from '@noir-lang/noirc_abi';

import PrivateKernelInitJson from './target/private_kernel_init.json' assert { type: 'json' };
import PrivateKernelInitSimulatedJson from './target/private_kernel_init_simulated.json' assert { type: 'json' };
import PrivateKernelInnerJson from './target/private_kernel_inner.json' assert { type: 'json' };
import PrivateKernelInnerSimulatedJson from './target/private_kernel_inner_simulated.json' assert { type: 'json' };
import PrivateKernelOrderingJson from './target/private_kernel_ordering.json' assert { type: 'json' };
import PrivateKernelOrderingSimulatedJson from './target/private_kernel_ordering_simulated.json' assert { type: 'json' };
import PublicKernelPrivatePreviousJson from './target/public_kernel_private_previous.json' assert { type: 'json' };
import PublicKernelPrivatePreviousSimulatedJson from './target/public_kernel_private_previous_simulated.json' assert { type: 'json' };
import PublicKernelPublicPreviousJson from './target/public_kernel_public_previous.json' assert { type: 'json' };
import PublicKernelPublicPreviousSimulatedJson from './target/public_kernel_public_previous_simulated.json' assert { type: 'json' };
import BaseRollupSimulatedJson from './target/rollup_base_simulated.json' assert { type: 'json' };
import MergeRollupJson from './target/rollup_merge.json' assert { type: 'json' };
import RootRollupJson from './target/rollup_root.json' assert { type: 'json' };
import {
  mapBaseOrMergeRollupPublicInputsFromNoir,
  mapBaseRollupInputsToNoir,
  mapKernelCircuitPublicInputsFinalFromNoir,
  mapKernelCircuitPublicInputsFromNoir,
  mapMergeRollupInputsToNoir,
  mapPrivateKernelInputsInitToNoir,
  mapPrivateKernelInputsInnerToNoir,
  mapPrivateKernelInputsOrderingToNoir,
  mapPublicKernelInputs,
  mapRootRollupInputsToNoir,
  mapRootRollupPublicInputsFromNoir,
} from './type_conversion.js';
import { InputType as InitInputType, ReturnType } from './types/private_kernel_init_types.js';
import { InputType as InnerInputType } from './types/private_kernel_inner_types.js';
import {
  ReturnType as FinalReturnType,
  InputType as OrderingInputType,
} from './types/private_kernel_ordering_types.js';
import {
  InputType as PublicPrivatePreviousInputType,
  ReturnType as PublicPrivatePreviousReturnType,
} from './types/public_kernel_private_previous_types.js';
import {
  InputType as PublicPublicPreviousInputType,
  ReturnType as PublicPublicPreviousReturnType,
} from './types/public_kernel_public_previous_types.js';
import { InputType as BaseRollupInputType, ReturnType as BaseRollupReturnType } from './types/rollup_base_types.js';
import { InputType as MergeRollupInputType, ReturnType as MergeRollupReturnType } from './types/rollup_merge_types.js';
import { InputType as RootRollupInputType, ReturnType as RootRollupReturnType } from './types/rollup_root_types.js';

// TODO(Tom): This should be exported from noirc_abi
/**
 * The decoded inputs from the circuit.
 */
export type DecodedInputs = {
  /**
   * The inputs to the circuit
   */
  inputs: Record<string, any>;
  /**
   * The return value of the circuit
   */
  return_value: any;
};

export const PrivateKernelInitArtifact = PrivateKernelInitJson as NoirCompiledCircuit;

export const PrivateKernelInnerArtifact = PrivateKernelInnerJson as NoirCompiledCircuit;

export const PrivateKernelOrderingArtifact = PrivateKernelOrderingJson as NoirCompiledCircuit;

export const PublicKernelPrivatePreviousArtifact = PublicKernelPrivatePreviousJson as NoirCompiledCircuit;

export const PublicKernelPublicPreviousArtifact = PublicKernelPublicPreviousJson as NoirCompiledCircuit;

/**
 * Executes the init private kernel.
 * @param privateKernelInputsInit - The private kernel inputs.
 * @returns The public inputs.
 */
export async function executeInit(
  privateKernelInputsInit: PrivateKernelInputsInit,
): Promise<KernelCircuitPublicInputs> {
  const params: InitInputType = {
    input: mapPrivateKernelInputsInitToNoir(privateKernelInputsInit),
  };

  const returnType = await executePrivateKernelInitWithACVM(params);

  return mapKernelCircuitPublicInputsFromNoir(returnType);
}

let solver: Promise<WasmBlackBoxFunctionSolver>;

const getSolver = (): Promise<WasmBlackBoxFunctionSolver> => {
  if (!solver) {
    solver = createBlackBoxSolver();
  }
  return solver;
};

/**
 * Executes the inner private kernel.
 * @param privateKernelInputsInner - The private kernel inputs.
 * @returns The public inputs.
 */
export async function executeInner(
  privateKernelInputsInner: PrivateKernelInputsInner,
): Promise<KernelCircuitPublicInputs> {
  const params: InnerInputType = {
    input: mapPrivateKernelInputsInnerToNoir(privateKernelInputsInner),
  };

  const returnType = await executePrivateKernelInnerWithACVM(params);

  return mapKernelCircuitPublicInputsFromNoir(returnType);
}

/**
 * Executes the inner private kernel.
 * @param privateKernelInputsInit - The private kernel inputs.
 * @returns The public inputs.
 */
export async function executeOrdering(
  privateKernelInputsOrdering: PrivateKernelInputsOrdering,
): Promise<KernelCircuitPublicInputsFinal> {
  const params: OrderingInputType = {
    input: mapPrivateKernelInputsOrderingToNoir(privateKernelInputsOrdering),
  };

  const returnType = await executePrivateKernelOrderingWithACVM(params);

  return mapKernelCircuitPublicInputsFinalFromNoir(returnType);
}

/**
 * Executes the public kernel.
 * @param privateKernelInputsInit - The public kernel private inputs.
 * @returns The public inputs.
 */
export async function executePublicKernelPrivatePrevious(
  publicKernelPrivateInputs: PublicKernelInputs,
): Promise<KernelCircuitPublicInputs> {
  const params: PublicPrivatePreviousInputType = {
    input: mapPublicKernelInputs(publicKernelPrivateInputs),
  };

  const returnType = await executePublicKernelPrivatePreviousWithACVM(params);

  return mapKernelCircuitPublicInputsFromNoir(returnType);
}

/**
 * Executes the inner public kernel.
 * @param privateKernelInputsInit - The public kernel private inputs.
 * @returns The public inputs.
 */
export async function executePublicKernelPublicPrevious(
  publicKernelPrivateInputs: PublicKernelInputs,
): Promise<KernelCircuitPublicInputs> {
  const params: PublicPrivatePreviousInputType = {
    input: mapPublicKernelInputs(publicKernelPrivateInputs),
  };

  const returnType = await executePublicKernelPublicPreviousWithACVM(params);

  return mapKernelCircuitPublicInputsFromNoir(returnType);
}

/**
 * Executes the root rollup.
 * @param rootRollupInputs - The root rollup inputs.
 * @returns The public inputs.
 */
export async function executeRootRollup(rootRollupInputs: RootRollupInputs): Promise<RootRollupPublicInputs> {
  const params: RootRollupInputType = {
    inputs: mapRootRollupInputsToNoir(rootRollupInputs),
  };

  const returnType = await executeRootRollupWithACVM(params);

  return mapRootRollupPublicInputsFromNoir(returnType);
}

/**
 * Executes the merge rollup.
 * @param mergeRollupInputs - The merge rollup inputs.
 * @returns The public inputs.
 */
export async function executeMergeRollup(mergeRollupInputs: MergeRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
  const params: MergeRollupInputType = {
    inputs: mapMergeRollupInputsToNoir(mergeRollupInputs),
  };

  const returnType = await executeMergeRollupWithACVM(params);

  return mapBaseOrMergeRollupPublicInputsFromNoir(returnType);
}

/**
 * Executes the base rollup.
 * @param mergeRollupInputs - The merge rollup inputs.
 * @returns The public inputs.
 */
export async function executeBaseRollup(baseRollupInputs: BaseRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
  const params: BaseRollupInputType = {
    inputs: mapBaseRollupInputsToNoir(baseRollupInputs),
  };

  const returnType = await executeBaseRollupWithACVM(params);

  return mapBaseOrMergeRollupPublicInputsFromNoir(returnType);
}

/**
 * Executes the init private kernel with the given inputs using the acvm.
 *
 */
async function executePrivateKernelInitWithACVM(input: InitInputType): Promise<ReturnType> {
  const initialWitnessMap = abiEncode(PrivateKernelInitSimulatedJson.abi as Abi, input as any);

  // Execute the circuit on those initial witness values
  //
  // Decode the bytecode from base64 since the acvm does not know about base64 encoding
  const decodedBytecode = Buffer.from(PrivateKernelInitSimulatedJson.bytecode, 'base64');
  //
  // Execute the circuit
  const _witnessMap = await executeCircuitWithBlackBoxSolver(
    await getSolver(),
    decodedBytecode,
    initialWitnessMap,
    () => {
      throw Error('unexpected oracle during execution');
    },
  );

  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(PrivateKernelInitSimulatedJson.abi as Abi, _witnessMap);

  // Cast the inputs as the return type
  return decodedInputs.return_value as ReturnType;
}

/**
 * Executes the inner private kernel with the given inputs using the acvm.
 */
async function executePrivateKernelInnerWithACVM(input: InnerInputType): Promise<ReturnType> {
  const initialWitnessMap = abiEncode(PrivateKernelInnerSimulatedJson.abi as Abi, input as any);

  // Execute the circuit on those initial witness values
  //
  // Decode the bytecode from base64 since the acvm does not know about base64 encoding
  const decodedBytecode = Buffer.from(PrivateKernelInnerSimulatedJson.bytecode, 'base64');
  //
  // Execute the circuit
  const _witnessMap = await executeCircuitWithBlackBoxSolver(
    await getSolver(),
    decodedBytecode,
    initialWitnessMap,
    () => {
      throw Error('unexpected oracle during execution');
    },
  );

  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(PrivateKernelInnerSimulatedJson.abi as Abi, _witnessMap);

  // Cast the inputs as the return type
  return decodedInputs.return_value as ReturnType;
}

/**
 * Executes the ordering private kernel with the given inputs using the acvm.
 */
async function executePrivateKernelOrderingWithACVM(input: OrderingInputType): Promise<FinalReturnType> {
  const initialWitnessMap = abiEncode(PrivateKernelOrderingSimulatedJson.abi as Abi, input as any);

  // Execute the circuit on those initial witness values
  //
  // Decode the bytecode from base64 since the acvm does not know about base64 encoding
  const decodedBytecode = Buffer.from(PrivateKernelOrderingSimulatedJson.bytecode, 'base64');
  //
  // Execute the circuit
  const _witnessMap = await executeCircuitWithBlackBoxSolver(
    await getSolver(),
    decodedBytecode,
    initialWitnessMap,
    () => {
      throw Error('unexpected oracle during execution');
    },
  );

  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(PrivateKernelOrderingSimulatedJson.abi as Abi, _witnessMap);

  // Cast the inputs as the return type
  return decodedInputs.return_value as FinalReturnType;
}

/**
 * Executes the public kernel with private prevoius kernel with the given inputs
 */
async function executePublicKernelPrivatePreviousWithACVM(
  input: PublicPrivatePreviousInputType,
): Promise<PublicPrivatePreviousReturnType> {
  const initialWitnessMap = abiEncode(PublicKernelPrivatePreviousSimulatedJson.abi as Abi, input as any);
  const decodedBytecode = Buffer.from(PublicKernelPrivatePreviousSimulatedJson.bytecode, 'base64');
  // Execute the circuit
  const _witnessMap = await executeCircuitWithBlackBoxSolver(
    await getSolver(),
    decodedBytecode,
    initialWitnessMap,
    () => {
      throw Error('unexpected oracle during execution');
    },
  );

  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(PublicKernelPrivatePreviousSimulatedJson.abi as Abi, _witnessMap);
  // Cast the inputs as the return type
  return decodedInputs.return_value as PublicPrivatePreviousReturnType;
}

/**
 * Executes the ordering private kernel with the given inputs using the acvm.
 */
async function executePublicKernelPublicPreviousWithACVM(
  input: PublicPublicPreviousInputType,
): Promise<PublicPublicPreviousReturnType> {
  const initialWitnessMap = abiEncode(PublicKernelPublicPreviousSimulatedJson.abi as Abi, input as any);
  const decodedBytecode = Buffer.from(PublicKernelPublicPreviousSimulatedJson.bytecode, 'base64');
  // Execute the circuit
  const _witnessMap = await executeCircuitWithBlackBoxSolver(
    await getSolver(),
    decodedBytecode,
    initialWitnessMap,
    () => {
      throw Error('unexpected oracle during execution');
    },
  );

  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(PublicKernelPublicPreviousSimulatedJson.abi as Abi, _witnessMap);

  // Cast the inputs as the return type
  return decodedInputs.return_value as PublicPublicPreviousReturnType;
}

/**
 * Executes the root rollup with the given inputs using the acvm.
 */
async function executeRootRollupWithACVM(input: RootRollupInputType): Promise<RootRollupReturnType> {
  const initialWitnessMap = abiEncode(RootRollupJson.abi as Abi, input as any);

  // Execute the circuit on those initial witness values
  //
  // Decode the bytecode from base64 since the acvm does not know about base64 encoding
  const decodedBytecode = Buffer.from(RootRollupJson.bytecode, 'base64');
  //
  // Execute the circuit
  const _witnessMap = await executeCircuitWithBlackBoxSolver(
    await getSolver(),
    decodedBytecode,
    initialWitnessMap,
    () => {
      throw Error('unexpected oracle during execution');
    },
  );

  const decodedInputs: DecodedInputs = abiDecode(RootRollupJson.abi as Abi, _witnessMap);

  // Cast the inputs as the return type
  return decodedInputs.return_value as RootRollupReturnType;
}

/**
 * Executes the merge rollup with the given inputs using the acvm.
 */
async function executeMergeRollupWithACVM(input: MergeRollupInputType): Promise<MergeRollupReturnType> {
  const initialWitnessMap = abiEncode(MergeRollupJson.abi as Abi, input as any);

  // Execute the circuit on those initial witness values
  //
  // Decode the bytecode from base64 since the acvm does not know about base64 encoding
  const decodedBytecode = Buffer.from(MergeRollupJson.bytecode, 'base64');
  //
  // Execute the circuit
  const _witnessMap = await executeCircuitWithBlackBoxSolver(
    await getSolver(),
    decodedBytecode,
    initialWitnessMap,
    () => {
      throw Error('unexpected oracle during execution');
    },
  );

  const decodedInputs: DecodedInputs = abiDecode(MergeRollupJson.abi as Abi, _witnessMap);

  // Cast the inputs as the return type
  return decodedInputs.return_value as MergeRollupReturnType;
}

/**
 * Executes the base rollup with the given inputs using the acvm.
 */
async function executeBaseRollupWithACVM(input: BaseRollupInputType): Promise<BaseRollupReturnType> {
  const initialWitnessMap = abiEncode(BaseRollupSimulatedJson.abi as Abi, input as any);

  // Execute the circuit on those initial witness values
  //
  // Decode the bytecode from base64 since the acvm does not know about base64 encoding
  const decodedBytecode = Buffer.from(BaseRollupSimulatedJson.bytecode, 'base64');
  //
  // Execute the circuit
  const _witnessMap = await executeCircuitWithBlackBoxSolver(
    await getSolver(),
    decodedBytecode,
    initialWitnessMap,
    () => {
      throw Error('unexpected oracle during execution');
    },
  );

  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(BaseRollupSimulatedJson.abi as Abi, _witnessMap);

  // Cast the inputs as the return type
  return decodedInputs.return_value as BaseRollupReturnType;
}
