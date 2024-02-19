import {
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  MergeRollupInputs,
  PrivateKernelInitCircuitPrivateInputs,
  PrivateKernelInnerCircuitPrivateInputs,
  PrivateKernelInnerCircuitPublicInputs,
  PrivateKernelTailCircuitPrivateInputs,
  PrivateKernelTailCircuitPublicInputs,
  PublicKernelCircuitPrivateInputs,
  PublicKernelCircuitPublicInputs,
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
import PrivateKernelTailJson from './target/private_kernel_tail.json' assert { type: 'json' };
import PrivateKernelTailSimulatedJson from './target/private_kernel_tail_simulated.json' assert { type: 'json' };
import PublicKernelAppLogicJson from './target/public_kernel_app_logic.json' assert { type: 'json' };
import PublicKernelAppLogicSimulatedJson from './target/public_kernel_app_logic_simulated.json' assert { type: 'json' };
import PublicKernelSetupJson from './target/public_kernel_setup.json' assert { type: 'json' };
import PublicKernelSetupSimulatedJson from './target/public_kernel_setup_simulated.json' assert { type: 'json' };
import PublicKernelTeardownJson from './target/public_kernel_teardown.json' assert { type: 'json' };
import PublicKernelTeardownSimulatedJson from './target/public_kernel_teardown_simulated.json' assert { type: 'json' };
import BaseRollupSimulatedJson from './target/rollup_base_simulated.json' assert { type: 'json' };
import MergeRollupJson from './target/rollup_merge.json' assert { type: 'json' };
import RootRollupJson from './target/rollup_root.json' assert { type: 'json' };
import {
  mapBaseOrMergeRollupPublicInputsFromNoir,
  mapBaseRollupInputsToNoir,
  mapMergeRollupInputsToNoir,
  mapPrivateKernelInitCircuitPrivateInputsToNoir,
  mapPrivateKernelInnerCircuitPrivateInputsToNoir,
  mapPrivateKernelInnerCircuitPublicInputsFromNoir,
  mapPrivateKernelTailCircuitPrivateInputsToNoir,
  mapPrivateKernelTailCircuitPublicInputsFromNoir,
  mapPublicKernelCircuitPrivateInputsToNoir,
  mapPublicKernelCircuitPublicInputsFromNoir,
  mapRootRollupInputsToNoir,
  mapRootRollupPublicInputsFromNoir,
} from './type_conversion.js';
import { InputType as InitInputType, ReturnType as InitReturnType } from './types/private_kernel_init_types.js';
import { InputType as InnerInputType, ReturnType as InnerReturnType } from './types/private_kernel_inner_types.js';
import { InputType as TailInputType, ReturnType as TailReturnType } from './types/private_kernel_tail_types.js';
import {
  InputType as PublicPublicPreviousInputType,
  ReturnType as PublicPublicPreviousReturnType,
} from './types/public_kernel_app_logic_types.js';
import {
  InputType as PublicSetupInputType,
  ReturnType as PublicSetupReturnType,
} from './types/public_kernel_setup_types.js';
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

export const PrivateKernelTailArtifact = PrivateKernelTailJson as NoirCompiledCircuit;

export const PublicKernelSetupArtifact = PublicKernelSetupJson as NoirCompiledCircuit;

export const PublicKernelAppLogicArtifact = PublicKernelAppLogicJson as NoirCompiledCircuit;

export const PublicKernelTeardownArtifact = PublicKernelTeardownJson as NoirCompiledCircuit;

/**
 * Executes the init private kernel.
 * @param privateKernelInitCircuitPrivateInputs - The private inputs to the initial private kernel.
 * @returns The public inputs.
 */
export async function executeInit(
  privateKernelInitCircuitPrivateInputs: PrivateKernelInitCircuitPrivateInputs,
): Promise<PrivateKernelInnerCircuitPublicInputs> {
  const params: InitInputType = {
    input: mapPrivateKernelInitCircuitPrivateInputsToNoir(privateKernelInitCircuitPrivateInputs),
  };

  const returnType = await executePrivateKernelInitWithACVM(params);

  return mapPrivateKernelInnerCircuitPublicInputsFromNoir(returnType);
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
 * @param privateKernelInnerCircuitPrivateInputs - The private inputs to the inner private kernel.
 * @returns The public inputs.
 */
export async function executeInner(
  privateKernelInnerCircuitPrivateInputs: PrivateKernelInnerCircuitPrivateInputs,
): Promise<PrivateKernelInnerCircuitPublicInputs> {
  const params: InnerInputType = {
    input: mapPrivateKernelInnerCircuitPrivateInputsToNoir(privateKernelInnerCircuitPrivateInputs),
  };
  const returnType = await executePrivateKernelInnerWithACVM(params);

  return mapPrivateKernelInnerCircuitPublicInputsFromNoir(returnType);
}

/**
 * Executes the tail private kernel.
 * @param privateKernelInnerCircuitPrivateInputs - The private inputs to the tail private kernel.
 * @returns The public inputs.
 */
export async function executeTail(
  privateKernelInnerCircuitPrivateInputs: PrivateKernelTailCircuitPrivateInputs,
): Promise<PrivateKernelTailCircuitPublicInputs> {
  const params: TailInputType = {
    input: mapPrivateKernelTailCircuitPrivateInputsToNoir(privateKernelInnerCircuitPrivateInputs),
  };

  const returnType = await executePrivateKernelTailWithACVM(params);

  return mapPrivateKernelTailCircuitPublicInputsFromNoir(returnType);
}

/**
 * Executes the public kernel in the setup phase.
 * @param publicKernelPrivateInputs - The public kernel setup circuit private inputs.
 * @returns The public inputs.
 */
export async function executePublicKernelSetup(
  publicKernelPrivateInputs: PublicKernelCircuitPrivateInputs,
): Promise<PublicKernelCircuitPublicInputs> {
  const params: PublicSetupInputType = {
    input: mapPublicKernelCircuitPrivateInputsToNoir(publicKernelPrivateInputs),
  };

  const returnType = await executePublicKernelSetupWithACVM(params);

  return mapPublicKernelCircuitPublicInputsFromNoir(returnType);
}

/**
 * Executes the public kernel in the app logic phase.
 * @param publicKernelPrivateInputs - The public kernel app logic circuit private inputs.
 * @returns The public inputs.
 */
export async function executePublicKernelAppLogic(
  publicKernelPrivateInputs: PublicKernelCircuitPrivateInputs,
): Promise<PublicKernelCircuitPublicInputs> {
  const params: PublicPublicPreviousInputType = {
    input: mapPublicKernelCircuitPrivateInputsToNoir(publicKernelPrivateInputs),
  };

  const returnType = await executePublicKernelAppLogicWithACVM(params);

  return mapPublicKernelCircuitPublicInputsFromNoir(returnType);
}

/**
 * Executes the public kernel in the teardown phase.
 * @param publicKernelPrivateInputs - The public kernel teardown circuit private inputs.
 * @returns The public inputs.
 */
export async function executePublicKernelTeardown(
  publicKernelPrivateInputs: PublicKernelCircuitPrivateInputs,
): Promise<PublicKernelCircuitPublicInputs> {
  const params: PublicPublicPreviousInputType = {
    input: mapPublicKernelCircuitPrivateInputsToNoir(publicKernelPrivateInputs),
  };

  const returnType = await executePublicKernelTeardownWithACVM(params);

  return mapPublicKernelCircuitPublicInputsFromNoir(returnType);
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
 * Executes the private init kernel with the given inputs using the acvm.
 *
 */
async function executePrivateKernelInitWithACVM(input: InitInputType): Promise<InitReturnType> {
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
  return decodedInputs.return_value as InitReturnType;
}

/**
 * Executes the private inner kernel with the given inputs using the acvm.
 */
async function executePrivateKernelInnerWithACVM(input: InnerInputType): Promise<InnerReturnType> {
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
  return decodedInputs.return_value as InnerReturnType;
}

/**
 * Executes the private tail kernel with the given inputs using the acvm.
 */
async function executePrivateKernelTailWithACVM(input: TailInputType): Promise<TailReturnType> {
  const initialWitnessMap = abiEncode(PrivateKernelTailSimulatedJson.abi as Abi, input as any);

  // Execute the circuit on those initial witness values
  //
  // Decode the bytecode from base64 since the acvm does not know about base64 encoding
  const decodedBytecode = Buffer.from(PrivateKernelTailSimulatedJson.bytecode, 'base64');
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
  const decodedInputs: DecodedInputs = abiDecode(PrivateKernelTailSimulatedJson.abi as Abi, _witnessMap);

  // Cast the inputs as the return type
  return decodedInputs.return_value as TailReturnType;
}

/**
 * Executes the public setup kernel with the given inputs
 */
async function executePublicKernelSetupWithACVM(input: PublicSetupInputType): Promise<PublicSetupReturnType> {
  const initialWitnessMap = abiEncode(PublicKernelSetupSimulatedJson.abi as Abi, input as any);
  const decodedBytecode = Buffer.from(PublicKernelSetupSimulatedJson.bytecode, 'base64');
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
  const decodedInputs: DecodedInputs = abiDecode(PublicKernelSetupSimulatedJson.abi as Abi, _witnessMap);
  // Cast the inputs as the return type
  return decodedInputs.return_value as PublicSetupReturnType;
}

/**
 * Executes the public app logic kernel with the given inputs using the acvm.
 */
async function executePublicKernelAppLogicWithACVM(
  input: PublicPublicPreviousInputType,
): Promise<PublicPublicPreviousReturnType> {
  const initialWitnessMap = abiEncode(PublicKernelAppLogicSimulatedJson.abi as Abi, input as any);
  const decodedBytecode = Buffer.from(PublicKernelAppLogicSimulatedJson.bytecode, 'base64');
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
  const decodedInputs: DecodedInputs = abiDecode(PublicKernelAppLogicSimulatedJson.abi as Abi, _witnessMap);

  // Cast the inputs as the return type
  return decodedInputs.return_value as PublicPublicPreviousReturnType;
}

/**
 * Executes the public teardown kernel with the given inputs using the acvm.
 */
async function executePublicKernelTeardownWithACVM(
  input: PublicPublicPreviousInputType,
): Promise<PublicPublicPreviousReturnType> {
  const initialWitnessMap = abiEncode(PublicKernelTeardownSimulatedJson.abi as Abi, input as any);
  const decodedBytecode = Buffer.from(PublicKernelTeardownSimulatedJson.bytecode, 'base64');
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
  const decodedInputs: DecodedInputs = abiDecode(PublicKernelTeardownSimulatedJson.abi as Abi, _witnessMap);

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
