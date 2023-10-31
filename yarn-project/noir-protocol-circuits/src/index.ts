import {
  KernelCircuitPublicInputs,
  KernelCircuitPublicInputsFinal,
  PrivateKernelInputsInit,
  PrivateKernelInputsInner,
  PrivateKernelInputsOrdering,
} from '@aztec/circuits.js';
import { NoirCompiledCircuit } from '@aztec/noir-compiler';

import { WasmBlackBoxFunctionSolver, createBlackBoxSolver, executeCircuitWithBlackBoxSolver } from '@noir-lang/acvm_js';
import { abiDecode, abiEncode } from '@noir-lang/noirc_abi';

import PrivateKernelInitJson from './target/private_kernel_init.json' assert { type: 'json' };
import PrivateKernelInitSimulatedJson from './target/private_kernel_init_simulated.json' assert { type: 'json' };
import PrivateKernelInnerJson from './target/private_kernel_inner.json' assert { type: 'json' };
import PrivateKernelInnerSimulatedJson from './target/private_kernel_inner_simulated.json' assert { type: 'json' };
import PrivateKernelOrderingJson from './target/private_kernel_ordering.json' assert { type: 'json' };
import PrivateKernelOrderingSimulatedJson from './target/private_kernel_ordering_simulated.json' assert { type: 'json' };
import {
  mapKernelCircuitPublicInputsFinalFromNoir,
  mapKernelCircuitPublicInputsFromNoir,
  mapPrivateKernelInputsInitToNoir,
  mapPrivateKernelInputsInnerToNoir,
  mapPrivateKernelInputsOrderingToNoir,
} from './type_conversion.js';
import { InputType as InitInputType, ReturnType } from './types/private_kernel_init_types.js';
import { InputType as InnerInputType } from './types/private_kernel_inner_types.js';
import {
  ReturnType as FinalReturnType,
  InputType as OrderingInputType,
} from './types/private_kernel_ordering_types.js';

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
 * Executes the init private kernel with the given inputs using the acvm.
 *
 */
async function executePrivateKernelInitWithACVM(input: InitInputType): Promise<ReturnType> {
  const initialWitnessMap = abiEncode(PrivateKernelInitSimulatedJson.abi, input, null);

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
  const decodedInputs: DecodedInputs = abiDecode(PrivateKernelInitSimulatedJson.abi, _witnessMap);

  // Cast the inputs as the return type
  return decodedInputs.return_value as ReturnType;
}

/**
 * Executes the inner private kernel with the given inputs using the acvm.
 */
async function executePrivateKernelInnerWithACVM(input: InnerInputType): Promise<ReturnType> {
  const initialWitnessMap = abiEncode(PrivateKernelInnerSimulatedJson.abi, input, null);

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
  const decodedInputs: DecodedInputs = abiDecode(PrivateKernelInnerSimulatedJson.abi, _witnessMap);

  // Cast the inputs as the return type
  return decodedInputs.return_value as ReturnType;
}

/**
 * Executes the ordering private kernel with the given inputs using the acvm.
 */
async function executePrivateKernelOrderingWithACVM(input: OrderingInputType): Promise<FinalReturnType> {
  const initialWitnessMap = abiEncode(PrivateKernelOrderingSimulatedJson.abi, input, null);

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
  const decodedInputs: DecodedInputs = abiDecode(PrivateKernelOrderingSimulatedJson.abi, _witnessMap);

  // Cast the inputs as the return type
  return decodedInputs.return_value as FinalReturnType;
}
