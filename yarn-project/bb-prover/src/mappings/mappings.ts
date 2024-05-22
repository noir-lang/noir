import { PublicKernelType } from '@aztec/circuit-types';
import { type PublicKernelCircuitPrivateInputs, type PublicKernelCircuitPublicInputs } from '@aztec/circuits.js';
import {
  type ServerProtocolArtifact,
  convertPublicInnerInputsToWitnessMap,
  convertPublicInnerOutputFromWitnessMap,
  convertPublicSetupInputsToWitnessMap,
  convertPublicSetupOutputFromWitnessMap,
  convertPublicTeardownInputsToWitnessMap,
  convertPublicTeardownOutputFromWitnessMap,
  convertSimulatedPublicInnerInputsToWitnessMap,
  convertSimulatedPublicInnerOutputFromWitnessMap,
  convertSimulatedPublicSetupInputsToWitnessMap,
  convertSimulatedPublicSetupOutputFromWitnessMap,
  convertSimulatedPublicTeardownInputsToWitnessMap,
  convertSimulatedPublicTeardownOutputFromWitnessMap,
} from '@aztec/noir-protocol-circuits-types';

import { type WitnessMap } from '@noir-lang/types';

export type PublicKernelProvingOps = {
  artifact: ServerProtocolArtifact;
  convertInputs: (inputs: PublicKernelCircuitPrivateInputs) => WitnessMap;
  convertOutputs: (outputs: WitnessMap) => PublicKernelCircuitPublicInputs;
};

export type KernelTypeToArtifact = Record<PublicKernelType, PublicKernelProvingOps | undefined>;

export const SimulatedPublicKernelArtifactMapping: KernelTypeToArtifact = {
  [PublicKernelType.NON_PUBLIC]: undefined,
  [PublicKernelType.APP_LOGIC]: {
    artifact: 'PublicKernelAppLogicArtifact',
    convertInputs: convertSimulatedPublicInnerInputsToWitnessMap,
    convertOutputs: convertSimulatedPublicInnerOutputFromWitnessMap,
  },
  [PublicKernelType.SETUP]: {
    artifact: 'PublicKernelSetupArtifact',
    convertInputs: convertSimulatedPublicSetupInputsToWitnessMap,
    convertOutputs: convertSimulatedPublicSetupOutputFromWitnessMap,
  },
  [PublicKernelType.TEARDOWN]: {
    artifact: 'PublicKernelTeardownArtifact',
    convertInputs: convertSimulatedPublicTeardownInputsToWitnessMap,
    convertOutputs: convertSimulatedPublicTeardownOutputFromWitnessMap,
  },
  [PublicKernelType.TAIL]: undefined,
};

export const PublicKernelArtifactMapping: KernelTypeToArtifact = {
  [PublicKernelType.NON_PUBLIC]: undefined,
  [PublicKernelType.APP_LOGIC]: {
    artifact: 'PublicKernelAppLogicArtifact',
    convertInputs: convertPublicInnerInputsToWitnessMap,
    convertOutputs: convertPublicInnerOutputFromWitnessMap,
  },
  [PublicKernelType.SETUP]: {
    artifact: 'PublicKernelSetupArtifact',
    convertInputs: convertPublicSetupInputsToWitnessMap,
    convertOutputs: convertPublicSetupOutputFromWitnessMap,
  },
  [PublicKernelType.TEARDOWN]: {
    artifact: 'PublicKernelTeardownArtifact',
    convertInputs: convertPublicTeardownInputsToWitnessMap,
    convertOutputs: convertPublicTeardownOutputFromWitnessMap,
  },
  [PublicKernelType.TAIL]: undefined,
};
