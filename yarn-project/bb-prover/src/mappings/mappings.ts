import { PublicKernelType } from '@aztec/circuit-types';
import { type PublicKernelCircuitPrivateInputs, type PublicKernelCircuitPublicInputs } from '@aztec/circuits.js';
import {
  type ServerProtocolArtifact,
  convertPublicInnerRollupInputsToWitnessMap,
  convertPublicInnerRollupOutputFromWitnessMap,
  convertPublicSetupRollupInputsToWitnessMap,
  convertPublicSetupRollupOutputFromWitnessMap,
  convertPublicTeardownRollupInputsToWitnessMap,
  convertPublicTeardownRollupOutputFromWitnessMap,
} from '@aztec/noir-protocol-circuits-types';

import { type WitnessMap } from '@noir-lang/types';

export type PublicKernelProvingOps = {
  artifact: ServerProtocolArtifact;
  convertInputs: (inputs: PublicKernelCircuitPrivateInputs) => WitnessMap;
  convertOutputs: (outputs: WitnessMap) => PublicKernelCircuitPublicInputs;
};

export type KernelTypeToArtifact = Record<PublicKernelType, PublicKernelProvingOps | undefined>;

export const PublicKernelArtifactMapping: KernelTypeToArtifact = {
  [PublicKernelType.NON_PUBLIC]: undefined,
  [PublicKernelType.APP_LOGIC]: {
    artifact: 'PublicKernelAppLogicArtifact',
    convertInputs: convertPublicInnerRollupInputsToWitnessMap,
    convertOutputs: convertPublicInnerRollupOutputFromWitnessMap,
  },
  [PublicKernelType.SETUP]: {
    artifact: 'PublicKernelSetupArtifact',
    convertInputs: convertPublicSetupRollupInputsToWitnessMap,
    convertOutputs: convertPublicSetupRollupOutputFromWitnessMap,
  },
  [PublicKernelType.TEARDOWN]: {
    artifact: 'PublicKernelTeardownArtifact',
    convertInputs: convertPublicTeardownRollupInputsToWitnessMap,
    convertOutputs: convertPublicTeardownRollupOutputFromWitnessMap,
  },
  [PublicKernelType.TAIL]: undefined,
};
