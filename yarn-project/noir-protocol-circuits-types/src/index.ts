import {
  type BaseOrMergeRollupPublicInputs,
  type BaseParityInputs,
  type BaseRollupInputs,
  Fr,
  type KernelCircuitPublicInputs,
  type MergeRollupInputs,
  type ParityPublicInputs,
  type PrivateKernelCircuitPublicInputs,
  type PrivateKernelEmptyInputs,
  type PrivateKernelInitCircuitPrivateInputs,
  type PrivateKernelInnerCircuitPrivateInputs,
  type PrivateKernelResetCircuitPrivateInputsVariants,
  type PrivateKernelResetTags,
  type PrivateKernelTailCircuitPrivateInputs,
  type PrivateKernelTailCircuitPublicInputs,
  type PublicKernelCircuitPrivateInputs,
  type PublicKernelCircuitPublicInputs,
  type PublicKernelTailCircuitPrivateInputs,
  type RootParityInputs,
  type RootRollupInputs,
  type RootRollupPublicInputs,
} from '@aztec/circuits.js';
import { applyStringFormatting, createDebugLogger } from '@aztec/foundation/log';

import { type ForeignCallInput, type ForeignCallOutput } from '@noir-lang/acvm_js';
import { type CompiledCircuit, type InputMap, Noir } from '@noir-lang/noir_js';
import { type Abi, abiDecode, abiEncode } from '@noir-lang/noirc_abi';
import { type WitnessMap } from '@noir-lang/types';
import { strict as assert } from 'assert';

import {
  ClientCircuitArtifacts,
  PrivateResetTagToArtifactName,
  ResetSimulatedArtifacts,
  ServerCircuitArtifacts,
  SimulatedClientCircuitArtifacts,
  SimulatedServerCircuitArtifacts,
} from './artifacts.js';
import {
  mapBaseOrMergeRollupPublicInputsFromNoir,
  mapBaseParityInputsToNoir,
  mapBaseRollupInputsToNoir,
  mapEmptyKernelInputsToNoir,
  mapKernelCircuitPublicInputsFromNoir,
  mapMergeRollupInputsToNoir,
  mapParityPublicInputsFromNoir,
  mapPrivateKernelCircuitPublicInputsFromNoir,
  mapPrivateKernelInitCircuitPrivateInputsToNoir,
  mapPrivateKernelInnerCircuitPrivateInputsToNoir,
  mapPrivateKernelResetCircuitPrivateInputsToNoir,
  mapPrivateKernelTailCircuitPrivateInputsToNoir,
  mapPrivateKernelTailCircuitPublicInputsForPublicFromNoir,
  mapPrivateKernelTailCircuitPublicInputsForRollupFromNoir,
  mapPrivateKernelTailToPublicCircuitPrivateInputsToNoir,
  mapPublicKernelCircuitPrivateInputsToNoir,
  mapPublicKernelCircuitPublicInputsFromNoir,
  mapPublicKernelTailCircuitPrivateInputsToNoir,
  mapRootParityInputsToNoir,
  mapRootRollupInputsToNoir,
  mapRootRollupPublicInputsFromNoir,
} from './type_conversion.js';
import {
  type ParityBaseReturnType as BaseParityReturnType,
  type RollupBaseReturnType as BaseRollupReturnType,
  type PrivateKernelInitReturnType as InitReturnType,
  type PrivateKernelInnerReturnType as InnerReturnType,
  type RollupMergeReturnType as MergeRollupReturnType,
  type PrivateKernelEmptyReturnType,
  type PublicKernelAppLogicReturnType as PublicPublicPreviousReturnType,
  type PublicKernelSetupReturnType as PublicSetupReturnType,
  type PrivateKernelResetReturnType as ResetReturnType,
  type ParityRootReturnType as RootParityReturnType,
  type RollupRootReturnType as RootRollupReturnType,
  type PrivateKernelTailReturnType as TailReturnType,
  PrivateKernelInit as executePrivateKernelInitWithACVM,
  PrivateKernelInner as executePrivateKernelInnerWithACVM,
  PrivateKernelTailToPublic as executePrivateKernelTailToPublicWithACVM,
  PrivateKernelTail as executePrivateKernelTailWithACVM,
} from './types/index.js';

export * from './artifacts.js';
export * from './vks.js';

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

/**
 * Executes the init private kernel.
 * @param privateKernelInitCircuitPrivateInputs - The private inputs to the initial private kernel.
 * @returns The public inputs.
 */
export async function executeInit(
  privateKernelInitCircuitPrivateInputs: PrivateKernelInitCircuitPrivateInputs,
): Promise<PrivateKernelCircuitPublicInputs> {
  const returnType = await executePrivateKernelInitWithACVM(
    mapPrivateKernelInitCircuitPrivateInputsToNoir(privateKernelInitCircuitPrivateInputs),
    SimulatedClientCircuitArtifacts.PrivateKernelInitArtifact as CompiledCircuit,
    foreignCallHandler,
  );

  return mapPrivateKernelCircuitPublicInputsFromNoir(returnType);
}

/**
 * Executes the inner private kernel.
 * @param privateKernelInnerCircuitPrivateInputs - The private inputs to the inner private kernel.
 * @returns The public inputs.
 */
export async function executeInner(
  privateKernelInnerCircuitPrivateInputs: PrivateKernelInnerCircuitPrivateInputs,
): Promise<PrivateKernelCircuitPublicInputs> {
  const returnType = await executePrivateKernelInnerWithACVM(
    mapPrivateKernelInnerCircuitPrivateInputsToNoir(privateKernelInnerCircuitPrivateInputs),
    SimulatedClientCircuitArtifacts.PrivateKernelInnerArtifact as CompiledCircuit,
    foreignCallHandler,
  );

  return mapPrivateKernelCircuitPublicInputsFromNoir(returnType);
}

/**
 * Executes the inner private kernel.
 * @param privateKernelResetCircuitPrivateInputs - The private inputs to the reset private kernel.
 * @returns The public inputs.
 */
export async function executeReset(
  privateKernelResetCircuitPrivateInputs: PrivateKernelResetCircuitPrivateInputsVariants,
): Promise<PrivateKernelCircuitPublicInputs> {
  const artifact =
    ResetSimulatedArtifacts[PrivateResetTagToArtifactName[privateKernelResetCircuitPrivateInputs.sizeTag]];
  const program = new Noir(artifact as CompiledCircuit);
  const args: InputMap = {
    input: mapPrivateKernelResetCircuitPrivateInputsToNoir(privateKernelResetCircuitPrivateInputs as any),
  };
  const { returnValue } = await program.execute(args, foreignCallHandler);
  return mapPrivateKernelCircuitPublicInputsFromNoir(returnValue as any);
}

/**
 * Executes the tail private kernel.
 * @param privateKernelCircuitPrivateInputs - The private inputs to the tail private kernel.
 * @returns The public inputs.
 */
export async function executeTail(
  privateInputs: PrivateKernelTailCircuitPrivateInputs,
): Promise<PrivateKernelTailCircuitPublicInputs> {
  const returnType = await executePrivateKernelTailWithACVM(
    mapPrivateKernelTailCircuitPrivateInputsToNoir(privateInputs),
    SimulatedClientCircuitArtifacts.PrivateKernelTailArtifact as CompiledCircuit,
    foreignCallHandler,
  );

  return mapPrivateKernelTailCircuitPublicInputsForRollupFromNoir(returnType);
}

/**
 * Executes the tail private kernel.
 * @param privateKernelInnerCircuitPrivateInputs - The private inputs to the tail private kernel.
 * @returns The public inputs.
 */
export async function executeTailForPublic(
  privateInputs: PrivateKernelTailCircuitPrivateInputs,
): Promise<PrivateKernelTailCircuitPublicInputs> {
  const returnType = await executePrivateKernelTailToPublicWithACVM(
    mapPrivateKernelTailToPublicCircuitPrivateInputsToNoir(privateInputs),
    SimulatedClientCircuitArtifacts.PrivateKernelTailToPublicArtifact as CompiledCircuit,
    foreignCallHandler,
  );

  return mapPrivateKernelTailCircuitPublicInputsForPublicFromNoir(returnType);
}

/**
 * Converts the inputs of the private kernel init circuit into a witness map
 * @param inputs - The private kernel inputs.
 * @returns The witness map
 */
export function convertPrivateKernelInitInputsToWitnessMap(
  privateKernelInitCircuitPrivateInputs: PrivateKernelInitCircuitPrivateInputs,
): WitnessMap {
  const mapped = mapPrivateKernelInitCircuitPrivateInputsToNoir(privateKernelInitCircuitPrivateInputs);
  const initialWitnessMap = abiEncode(ClientCircuitArtifacts.PrivateKernelInitArtifact.abi, {
    input: mapped as any,
  });
  return initialWitnessMap;
}

/**
 * Converts the inputs of the private kernel inner circuit into a witness map
 * @param inputs - The private kernel inputs.
 * @returns The witness map
 */
export function convertPrivateKernelInnerInputsToWitnessMap(
  privateKernelInnerCircuitPrivateInputs: PrivateKernelInnerCircuitPrivateInputs,
): WitnessMap {
  const mapped = mapPrivateKernelInnerCircuitPrivateInputsToNoir(privateKernelInnerCircuitPrivateInputs);
  const initialWitnessMap = abiEncode(ClientCircuitArtifacts.PrivateKernelInnerArtifact.abi, { input: mapped as any });
  return initialWitnessMap;
}

/**
 * Converts the inputs of the private kernel reset circuit into a witness map
 * @param inputs - The private kernel inputs.
 * @returns The witness map
 */
export function convertPrivateKernelResetInputsToWitnessMap(
  privateKernelResetCircuitPrivateInputs: PrivateKernelResetCircuitPrivateInputsVariants,
): WitnessMap {
  const mapped = mapPrivateKernelResetCircuitPrivateInputsToNoir(privateKernelResetCircuitPrivateInputs as any);
  const artifact =
    ClientCircuitArtifacts[PrivateResetTagToArtifactName[privateKernelResetCircuitPrivateInputs.sizeTag]];
  const initialWitnessMap = abiEncode(artifact.abi as Abi, { input: mapped as any });
  return initialWitnessMap;
}

/**
 * Converts the inputs of the private kernel tail circuit into a witness map
 * @param inputs - The private kernel inputs.
 * @returns The witness map
 */
export function convertPrivateKernelTailInputsToWitnessMap(
  privateKernelTailCircuitPrivateInputs: PrivateKernelTailCircuitPrivateInputs,
): WitnessMap {
  const mapped = mapPrivateKernelTailCircuitPrivateInputsToNoir(privateKernelTailCircuitPrivateInputs);
  const initialWitnessMap = abiEncode(ClientCircuitArtifacts.PrivateKernelTailArtifact.abi, { input: mapped as any });
  return initialWitnessMap;
}

/**
 * Converts the inputs of the private kernel tail to public circuit into a witness map
 * @param inputs - The private kernel inputs.
 * @returns The witness map
 */
export function convertPrivateKernelTailToPublicInputsToWitnessMap(
  privateKernelTailToPublicCircuitPrivateInputs: PrivateKernelTailCircuitPrivateInputs,
): WitnessMap {
  const mapped = mapPrivateKernelTailToPublicCircuitPrivateInputsToNoir(privateKernelTailToPublicCircuitPrivateInputs);
  const initialWitnessMap = abiEncode(ClientCircuitArtifacts.PrivateKernelTailToPublicArtifact.abi, {
    input: mapped as any,
  });
  return initialWitnessMap;
}

/**
 * Converts the outputs of the private kernel init circuit from a witness map.
 * @param outputs - The private kernel outputs as a witness map.
 * @returns The public inputs.
 */
export function convertPrivateKernelInitOutputsFromWitnessMap(outputs: WitnessMap): PrivateKernelCircuitPublicInputs {
  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(ClientCircuitArtifacts.PrivateKernelInitArtifact.abi, outputs);

  // Cast the inputs as the return type
  const returnType = decodedInputs.return_value as InitReturnType;

  return mapPrivateKernelCircuitPublicInputsFromNoir(returnType);
}

/**
 * Converts the outputs of the private kernel inner circuit from a witness map.
 * @param outputs - The private kernel outputs as a witness map.
 * @returns The public inputs.
 */
export function convertPrivateKernelInnerOutputsFromWitnessMap(outputs: WitnessMap): PrivateKernelCircuitPublicInputs {
  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(ClientCircuitArtifacts.PrivateKernelInnerArtifact.abi, outputs);

  // Cast the inputs as the return type
  const returnType = decodedInputs.return_value as InnerReturnType;

  return mapPrivateKernelCircuitPublicInputsFromNoir(returnType);
}

/**
 * Converts the outputs of the private kernel reset circuit from a witness map.
 * @param outputs - The private kernel outputs as a witness map.
 * @returns The public inputs.
 */
export function convertPrivateKernelResetOutputsFromWitnessMap(
  outputs: WitnessMap,
  sizeTag: PrivateKernelResetTags,
): PrivateKernelCircuitPublicInputs {
  // Decode the witness map into two fields, the return values and the inputs
  const artifact = ClientCircuitArtifacts[PrivateResetTagToArtifactName[sizeTag]];
  const decodedInputs: DecodedInputs = abiDecode(artifact.abi as Abi, outputs);

  // Cast the inputs as the return type
  const returnType = decodedInputs.return_value as ResetReturnType;

  return mapPrivateKernelCircuitPublicInputsFromNoir(returnType);
}

/**
 * Converts the outputs of the private kernel tail circuit from a witness map.
 * @param outputs - The private kernel outputs as a witness map.
 * @returns The public inputs.
 */
export function convertPrivateKernelTailOutputsFromWitnessMap(
  outputs: WitnessMap,
): PrivateKernelTailCircuitPublicInputs {
  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(ClientCircuitArtifacts.PrivateKernelTailArtifact.abi, outputs);

  // Cast the inputs as the return type
  const returnType = decodedInputs.return_value as TailReturnType;

  return mapPrivateKernelTailCircuitPublicInputsForRollupFromNoir(returnType);
}

/**
 * Converts the outputs of the private kernel tail for public circuit from a witness map.
 * @param outputs - The private kernel outputs as a witness map.
 * @returns The public inputs.
 */
export function convertPrivateKernelTailForPublicOutputsFromWitnessMap(
  outputs: WitnessMap,
): PrivateKernelTailCircuitPublicInputs {
  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(ClientCircuitArtifacts.PrivateKernelTailToPublicArtifact.abi, outputs);

  // Cast the inputs as the return type
  const returnType = decodedInputs.return_value as PublicPublicPreviousReturnType;

  return mapPrivateKernelTailCircuitPublicInputsForPublicFromNoir(returnType);
}

/**
 * Converts the inputs of the base parity circuit into a witness map.
 * @param inputs - The base parity inputs.
 * @returns The witness map
 */
export function convertBaseParityInputsToWitnessMap(inputs: BaseParityInputs): WitnessMap {
  const mapped = mapBaseParityInputsToNoir(inputs);
  const initialWitnessMap = abiEncode(ServerCircuitArtifacts.BaseParityArtifact.abi, { inputs: mapped as any });
  return initialWitnessMap;
}

/**
 * Converts the inputs of the root parity circuit into a witness map.
 * @param inputs - The root parity inputs.
 * @returns The witness map
 */
export function convertRootParityInputsToWitnessMap(inputs: RootParityInputs): WitnessMap {
  const mapped = mapRootParityInputsToNoir(inputs);
  const initialWitnessMap = abiEncode(ServerCircuitArtifacts.RootParityArtifact.abi, { inputs: mapped as any });
  return initialWitnessMap;
}

/**
 * Converts the inputs of the base rollup circuit into a witness map.
 * @param inputs - The base rollup inputs.
 * @returns The witness map
 */
export function convertBaseRollupInputsToWitnessMap(inputs: BaseRollupInputs): WitnessMap {
  const mapped = mapBaseRollupInputsToNoir(inputs);
  const initialWitnessMap = abiEncode(ServerCircuitArtifacts.BaseRollupArtifact.abi, { inputs: mapped as any });
  return initialWitnessMap;
}

export function convertPrivateKernelEmptyInputsToWitnessMap(inputs: PrivateKernelEmptyInputs): WitnessMap {
  const mapped = mapEmptyKernelInputsToNoir(inputs);
  const initialWitnessMap = abiEncode(ServerCircuitArtifacts.PrivateKernelEmptyArtifact.abi, { input: mapped as any });
  return initialWitnessMap;
}

/**
 * Converts the inputs of the simulated base rollup circuit into a witness map.
 * @param inputs - The base rollup inputs.
 * @returns The witness map
 */
export function convertSimulatedBaseRollupInputsToWitnessMap(inputs: BaseRollupInputs): WitnessMap {
  const mapped = mapBaseRollupInputsToNoir(inputs);
  const initialWitnessMap = abiEncode(SimulatedServerCircuitArtifacts.BaseRollupArtifact.abi, {
    inputs: mapped as any,
  });
  return initialWitnessMap;
}

/**
 * Converts the inputs of the merge rollup circuit into a witness map.
 * @param inputs - The merge rollup inputs.
 * @returns The witness map
 */
export function convertMergeRollupInputsToWitnessMap(inputs: MergeRollupInputs): WitnessMap {
  const mapped = mapMergeRollupInputsToNoir(inputs);
  const initialWitnessMap = abiEncode(ServerCircuitArtifacts.MergeRollupArtifact.abi, { inputs: mapped as any });
  return initialWitnessMap;
}

/**
 * Converts the inputs of the root rollup circuit into a witness map.
 * @param inputs - The root rollup inputs.
 * @returns The witness map
 */
export function convertRootRollupInputsToWitnessMap(inputs: RootRollupInputs): WitnessMap {
  const mapped = mapRootRollupInputsToNoir(inputs);
  const initialWitnessMap = abiEncode(ServerCircuitArtifacts.RootRollupArtifact.abi, { inputs: mapped as any });
  return initialWitnessMap;
}
/**
 * Converts the inputs of the public setup circuit into a witness map
 * @param inputs - The public kernel inputs.
 * @returns The witness map
 */
export function convertSimulatedPublicSetupInputsToWitnessMap(inputs: PublicKernelCircuitPrivateInputs): WitnessMap {
  const mapped = mapPublicKernelCircuitPrivateInputsToNoir(inputs);
  const initialWitnessMap = abiEncode(SimulatedServerCircuitArtifacts.PublicKernelSetupArtifact.abi, {
    input: mapped as any,
  });
  return initialWitnessMap;
}

/**
 * Converts the inputs of the public app logic circuit into a witness map
 * @param inputs - The public kernel inputs.
 * @returns The witness map
 */
export function convertSimulatedPublicInnerInputsToWitnessMap(inputs: PublicKernelCircuitPrivateInputs): WitnessMap {
  const mapped = mapPublicKernelCircuitPrivateInputsToNoir(inputs);
  const initialWitnessMap = abiEncode(SimulatedServerCircuitArtifacts.PublicKernelAppLogicArtifact.abi, {
    input: mapped as any,
  });
  return initialWitnessMap;
}

/**
 * Converts the inputs of the public teardown circuit into a witness map
 * @param inputs - The public kernel inputs.
 * @returns The witness map
 */
export function convertSimulatedPublicTeardownInputsToWitnessMap(inputs: PublicKernelCircuitPrivateInputs): WitnessMap {
  const mapped = mapPublicKernelCircuitPrivateInputsToNoir(inputs);
  const initialWitnessMap = abiEncode(SimulatedServerCircuitArtifacts.PublicKernelTeardownArtifact.abi, {
    input: mapped as any,
  });
  return initialWitnessMap;
}

/**
 * Converts the inputs of the public tail circuit into a witness map
 * @param inputs - The public kernel inputs.
 * @returns The witness map
 */
export function convertSimulatedPublicTailInputsToWitnessMap(inputs: PublicKernelTailCircuitPrivateInputs): WitnessMap {
  const mapped = mapPublicKernelTailCircuitPrivateInputsToNoir(inputs);
  const initialWitnessMap = abiEncode(SimulatedServerCircuitArtifacts.PublicKernelTailArtifact.abi, {
    input: mapped as any,
  });
  return initialWitnessMap;
}

/**
 * Converts the inputs of the public setup circuit into a witness map
 * @param inputs - The public kernel inputs.
 * @returns The witness map
 */
export function convertPublicSetupInputsToWitnessMap(inputs: PublicKernelCircuitPrivateInputs): WitnessMap {
  const mapped = mapPublicKernelCircuitPrivateInputsToNoir(inputs);
  const initialWitnessMap = abiEncode(ServerCircuitArtifacts.PublicKernelSetupArtifact.abi, { input: mapped as any });
  return initialWitnessMap;
}

/**
 * Converts the inputs of the public setup circuit into a witness map
 * @param inputs - The public kernel inputs.
 * @returns The witness map
 */
export function convertPublicInnerInputsToWitnessMap(inputs: PublicKernelCircuitPrivateInputs): WitnessMap {
  const mapped = mapPublicKernelCircuitPrivateInputsToNoir(inputs);
  const initialWitnessMap = abiEncode(ServerCircuitArtifacts.PublicKernelAppLogicArtifact.abi, {
    input: mapped as any,
  });
  return initialWitnessMap;
}

/**
 * Converts the inputs of the public teardown circuit into a witness map
 * @param inputs - The public kernel inputs.
 * @returns The witness map
 */
export function convertPublicTeardownInputsToWitnessMap(inputs: PublicKernelCircuitPrivateInputs): WitnessMap {
  const mapped = mapPublicKernelCircuitPrivateInputsToNoir(inputs);
  const initialWitnessMap = abiEncode(ServerCircuitArtifacts.PublicKernelTeardownArtifact.abi, {
    input: mapped as any,
  });
  return initialWitnessMap;
}

/**
 * Converts the inputs of the public tail circuit into a witness map
 * @param inputs - The public kernel inputs.
 * @returns The witness map
 */
export function convertPublicTailInputsToWitnessMap(inputs: PublicKernelTailCircuitPrivateInputs): WitnessMap {
  const mapped = mapPublicKernelTailCircuitPrivateInputsToNoir(inputs);
  const initialWitnessMap = abiEncode(ServerCircuitArtifacts.PublicKernelTailArtifact.abi, { input: mapped as any });
  return initialWitnessMap;
}

export function convertPrivateKernelEmptyOutputsFromWitnessMap(outputs: WitnessMap): KernelCircuitPublicInputs {
  const decodedInputs: DecodedInputs = abiDecode(ServerCircuitArtifacts.PrivateKernelEmptyArtifact.abi, outputs);
  const returnType = decodedInputs.return_value as PrivateKernelEmptyReturnType;

  return mapKernelCircuitPublicInputsFromNoir(returnType);
}

export function convertSimulatedPrivateKernelEmptyOutputsFromWitnessMap(
  outputs: WitnessMap,
): KernelCircuitPublicInputs {
  const decodedInputs: DecodedInputs = abiDecode(
    SimulatedServerCircuitArtifacts.PrivateKernelEmptyArtifact.abi,
    outputs,
  );
  const returnType = decodedInputs.return_value as PrivateKernelEmptyReturnType;

  return mapKernelCircuitPublicInputsFromNoir(returnType);
}

/**
 * Converts the outputs of the simulated base rollup circuit from a witness map.
 * @param outputs - The base rollup outputs as a witness map.
 * @returns The public inputs.
 */
export function convertSimulatedBaseRollupOutputsFromWitnessMap(outputs: WitnessMap): BaseOrMergeRollupPublicInputs {
  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(SimulatedServerCircuitArtifacts.BaseRollupArtifact.abi, outputs);

  // Cast the inputs as the return type
  const returnType = decodedInputs.return_value as BaseRollupReturnType;

  return mapBaseOrMergeRollupPublicInputsFromNoir(returnType);
}

/**
 * Converts the outputs of the base rollup circuit from a witness map.
 * @param outputs - The base rollup outputs as a witness map.
 * @returns The public inputs.
 */
export function convertBaseRollupOutputsFromWitnessMap(outputs: WitnessMap): BaseOrMergeRollupPublicInputs {
  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(ServerCircuitArtifacts.BaseRollupArtifact.abi, outputs);

  // Cast the inputs as the return type
  const returnType = decodedInputs.return_value as BaseRollupReturnType;

  return mapBaseOrMergeRollupPublicInputsFromNoir(returnType);
}

/**
 * Converts the outputs of the merge rollup circuit from a witness map.
 * @param outputs - The merge rollup outputs as a witness map.
 * @returns The public inputs.
 */
export function convertMergeRollupOutputsFromWitnessMap(outputs: WitnessMap): BaseOrMergeRollupPublicInputs {
  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(ServerCircuitArtifacts.MergeRollupArtifact.abi, outputs);

  // Cast the inputs as the return type
  const returnType = decodedInputs.return_value as MergeRollupReturnType;

  return mapBaseOrMergeRollupPublicInputsFromNoir(returnType);
}

/**
 * Converts the outputs of the root rollup circuit from a witness map.
 * @param outputs - The root rollup outputs as a witness map.
 * @returns The public inputs.
 */
export function convertRootRollupOutputsFromWitnessMap(outputs: WitnessMap): RootRollupPublicInputs {
  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(ServerCircuitArtifacts.RootRollupArtifact.abi, outputs);

  // Cast the inputs as the return type
  const returnType = decodedInputs.return_value as RootRollupReturnType;

  return mapRootRollupPublicInputsFromNoir(returnType);
}

/**
 * Converts the outputs of the base parity circuit from a witness map.
 * @param outputs - The base parity outputs as a witness map.
 * @returns The public inputs.
 */
export function convertBaseParityOutputsFromWitnessMap(outputs: WitnessMap): ParityPublicInputs {
  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(ServerCircuitArtifacts.BaseParityArtifact.abi, outputs);

  // Cast the inputs as the return type
  const returnType = decodedInputs.return_value as BaseParityReturnType;

  return mapParityPublicInputsFromNoir(returnType);
}

/**
 * Converts the outputs of the root parity circuit from a witness map.
 * @param outputs - The root parity outputs as a witness map.
 * @returns The public inputs.
 */
export function convertRootParityOutputsFromWitnessMap(outputs: WitnessMap): ParityPublicInputs {
  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(ServerCircuitArtifacts.RootParityArtifact.abi, outputs);

  // Cast the inputs as the return type
  const returnType = decodedInputs.return_value as RootParityReturnType;

  return mapParityPublicInputsFromNoir(returnType);
}

/**
 * Converts the outputs of the public setup circuit from a witness map.
 * @param outputs - The public kernel outputs as a witness map.
 * @returns The public inputs.
 */
export function convertSimulatedPublicSetupOutputFromWitnessMap(outputs: WitnessMap): PublicKernelCircuitPublicInputs {
  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(
    SimulatedServerCircuitArtifacts.PublicKernelSetupArtifact.abi,
    outputs,
  );

  // Cast the inputs as the return type
  const returnType = decodedInputs.return_value as PublicSetupReturnType;

  return mapPublicKernelCircuitPublicInputsFromNoir(returnType);
}

/**
 * Converts the outputs of the public inner circuit from a witness map.
 * @param outputs - The public kernel outputs as a witness map.
 * @returns The public inputs.
 */
export function convertSimulatedPublicInnerOutputFromWitnessMap(outputs: WitnessMap): PublicKernelCircuitPublicInputs {
  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(
    SimulatedServerCircuitArtifacts.PublicKernelAppLogicArtifact.abi,
    outputs,
  );

  // Cast the inputs as the return type
  const returnType = decodedInputs.return_value as PublicPublicPreviousReturnType;

  return mapPublicKernelCircuitPublicInputsFromNoir(returnType);
}

/**
 * Converts the outputs of the public tail circuit from a witness map.
 * @param outputs - The public kernel outputs as a witness map.
 * @returns The public inputs.
 */
export function convertSimulatedPublicTeardownOutputFromWitnessMap(
  outputs: WitnessMap,
): PublicKernelCircuitPublicInputs {
  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(
    SimulatedServerCircuitArtifacts.PublicKernelTeardownArtifact.abi,
    outputs,
  );

  // Cast the inputs as the return type
  const returnType = decodedInputs.return_value as PublicPublicPreviousReturnType;

  return mapPublicKernelCircuitPublicInputsFromNoir(returnType);
}

/**
 * Converts the outputs of the public tail circuit from a witness map.
 * @param outputs - The public kernel outputs as a witness map.
 * @returns The public inputs.
 */
export function convertSimulatedPublicTailOutputFromWitnessMap(outputs: WitnessMap): KernelCircuitPublicInputs {
  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(SimulatedServerCircuitArtifacts.PublicKernelTailArtifact.abi, outputs);

  // Cast the inputs as the return type
  const returnType = decodedInputs.return_value as TailReturnType;

  return mapKernelCircuitPublicInputsFromNoir(returnType);
}

/**
 * Converts the outputs of the public setup circuit from a witness map.
 * @param outputs - The public kernel outputs as a witness map.
 * @returns The public inputs.
 */
export function convertPublicSetupOutputFromWitnessMap(outputs: WitnessMap): PublicKernelCircuitPublicInputs {
  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(ServerCircuitArtifacts.PublicKernelSetupArtifact.abi, outputs);

  // Cast the inputs as the return type
  const returnType = decodedInputs.return_value as PublicSetupReturnType;

  return mapPublicKernelCircuitPublicInputsFromNoir(returnType);
}

/**
 * Converts the outputs of the public inner circuit from a witness map.
 * @param outputs - The public kernel outputs as a witness map.
 * @returns The public inputs.
 */
export function convertPublicInnerOutputFromWitnessMap(outputs: WitnessMap): PublicKernelCircuitPublicInputs {
  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(ServerCircuitArtifacts.PublicKernelAppLogicArtifact.abi, outputs);

  // Cast the inputs as the return type
  const returnType = decodedInputs.return_value as PublicPublicPreviousReturnType;

  return mapPublicKernelCircuitPublicInputsFromNoir(returnType);
}

/**
 * Converts the outputs of the public tail circuit from a witness map.
 * @param outputs - The public kernel outputs as a witness map.
 * @returns The public inputs.
 */
export function convertPublicTeardownOutputFromWitnessMap(outputs: WitnessMap): PublicKernelCircuitPublicInputs {
  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(ServerCircuitArtifacts.PublicKernelTeardownArtifact.abi, outputs);

  // Cast the inputs as the return type
  const returnType = decodedInputs.return_value as PublicPublicPreviousReturnType;

  return mapPublicKernelCircuitPublicInputsFromNoir(returnType);
}

/**
 * Converts the outputs of the public tail circuit from a witness map.
 * @param outputs - The public kernel outputs as a witness map.
 * @returns The public inputs.
 */
export function convertPublicTailOutputFromWitnessMap(outputs: WitnessMap): KernelCircuitPublicInputs {
  // Decode the witness map into two fields, the return values and the inputs
  const decodedInputs: DecodedInputs = abiDecode(ServerCircuitArtifacts.PublicKernelTailArtifact.abi, outputs);

  // Cast the inputs as the return type
  const returnType = decodedInputs.return_value as TailReturnType;

  return mapKernelCircuitPublicInputsFromNoir(returnType);
}

function fromACVMField(field: string): Fr {
  return Fr.fromBuffer(Buffer.from(field.slice(2), 'hex'));
}

export function foreignCallHandler(name: string, args: ForeignCallInput[]): Promise<ForeignCallOutput[]> {
  // ForeignCallInput is actually a string[], so the args are string[][].
  const log = createDebugLogger('aztec:noir-protocol-circuits:oracle');

  if (name === 'debugLog') {
    assert(args.length === 3, 'expected 3 arguments for debugLog: msg, fields_length, fields');
    const [msgRaw, _ignoredFieldsSize, fields] = args;
    const msg: string = msgRaw.map(acvmField => String.fromCharCode(fromACVMField(acvmField).toNumber())).join('');
    const fieldsFr: Fr[] = fields.map((field: string) => fromACVMField(field));
    log.verbose('debug_log ' + applyStringFormatting(msg, fieldsFr));
  } else {
    throw Error(`unexpected oracle during execution: ${name}`);
  }

  return Promise.resolve([]);
}
