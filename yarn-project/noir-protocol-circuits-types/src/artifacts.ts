import { type PrivateKernelResetTags } from '@aztec/circuits.js';
import { type NoirCompiledCircuit } from '@aztec/types/noir';

import EmptyNestedJson from '../artifacts/empty_nested.json' assert { type: 'json' };
import EmptyNestedSimulatedJson from '../artifacts/empty_nested_simulated.json' assert { type: 'json' };
import BaseParityJson from '../artifacts/parity_base.json' assert { type: 'json' };
import RootParityJson from '../artifacts/parity_root.json' assert { type: 'json' };
import PrivateKernelEmptyJson from '../artifacts/private_kernel_empty.json' assert { type: 'json' };
import PrivateKernelEmptySimulatedJson from '../artifacts/private_kernel_empty_simulated.json' assert { type: 'json' };
import PrivateKernelInitJson from '../artifacts/private_kernel_init.json' assert { type: 'json' };
import PrivateKernelInitSimulatedJson from '../artifacts/private_kernel_init_simulated.json' assert { type: 'json' };
import PrivateKernelInnerJson from '../artifacts/private_kernel_inner.json' assert { type: 'json' };
import PrivateKernelInnerSimulatedJson from '../artifacts/private_kernel_inner_simulated.json' assert { type: 'json' };
import PrivateKernelResetJson from '../artifacts/private_kernel_reset.json' assert { type: 'json' };
import PrivateKernelResetBigJson from '../artifacts/private_kernel_reset_big.json' assert { type: 'json' };
import PrivateKernelResetMediumJson from '../artifacts/private_kernel_reset_medium.json' assert { type: 'json' };
import PrivateKernelResetSimulatedJson from '../artifacts/private_kernel_reset_simulated.json' assert { type: 'json' };
import PrivateKernelResetBigSimulatedJson from '../artifacts/private_kernel_reset_simulated_big.json' assert { type: 'json' };
import PrivateKernelResetMediumSimulatedJson from '../artifacts/private_kernel_reset_simulated_medium.json' assert { type: 'json' };
import PrivateKernelResetSmallSimulatedJson from '../artifacts/private_kernel_reset_simulated_small.json' assert { type: 'json' };
import PrivateKernelResetSmallJson from '../artifacts/private_kernel_reset_small.json' assert { type: 'json' };
import PrivateKernelTailJson from '../artifacts/private_kernel_tail.json' assert { type: 'json' };
import PrivateKernelTailSimulatedJson from '../artifacts/private_kernel_tail_simulated.json' assert { type: 'json' };
import PrivateKernelTailToPublicJson from '../artifacts/private_kernel_tail_to_public.json' assert { type: 'json' };
import PrivateKernelTailToPublicSimulatedJson from '../artifacts/private_kernel_tail_to_public_simulated.json' assert { type: 'json' };
import PublicKernelAppLogicJson from '../artifacts/public_kernel_app_logic.json' assert { type: 'json' };
import PublicKernelAppLogicSimulatedJson from '../artifacts/public_kernel_app_logic_simulated.json' assert { type: 'json' };
import PublicKernelSetupJson from '../artifacts/public_kernel_setup.json' assert { type: 'json' };
import PublicKernelSetupSimulatedJson from '../artifacts/public_kernel_setup_simulated.json' assert { type: 'json' };
import PublicKernelTailJson from '../artifacts/public_kernel_tail.json' assert { type: 'json' };
import PublicKernelTailSimulatedJson from '../artifacts/public_kernel_tail_simulated.json' assert { type: 'json' };
import PublicKernelTeardownJson from '../artifacts/public_kernel_teardown.json' assert { type: 'json' };
import PublicKernelTeardownSimulatedJson from '../artifacts/public_kernel_teardown_simulated.json' assert { type: 'json' };
import BaseRollupJson from '../artifacts/rollup_base.json' assert { type: 'json' };
import BaseRollupSimulatedJson from '../artifacts/rollup_base_simulated.json' assert { type: 'json' };
import MergeRollupJson from '../artifacts/rollup_merge.json' assert { type: 'json' };
import RootRollupJson from '../artifacts/rollup_root.json' assert { type: 'json' };

export type PrivateResetArtifacts =
  | 'PrivateKernelResetFullArtifact'
  | 'PrivateKernelResetBigArtifact'
  | 'PrivateKernelResetMediumArtifact'
  | 'PrivateKernelResetSmallArtifact';

export const PrivateResetTagToArtifactName: Record<PrivateKernelResetTags, PrivateResetArtifacts> = {
  full: 'PrivateKernelResetFullArtifact',
  big: 'PrivateKernelResetBigArtifact',
  medium: 'PrivateKernelResetMediumArtifact',
  small: 'PrivateKernelResetSmallArtifact',
};

export type ServerProtocolArtifact =
  | 'EmptyNestedArtifact'
  | 'PrivateKernelEmptyArtifact'
  | 'PublicKernelSetupArtifact'
  | 'PublicKernelAppLogicArtifact'
  | 'PublicKernelTeardownArtifact'
  | 'PublicKernelTailArtifact'
  | 'BaseParityArtifact'
  | 'RootParityArtifact'
  | 'BaseRollupArtifact'
  | 'MergeRollupArtifact'
  | 'RootRollupArtifact';

export type ClientProtocolArtifact =
  | 'PrivateKernelInitArtifact'
  | 'PrivateKernelInnerArtifact'
  | 'PrivateKernelTailArtifact'
  | 'PrivateKernelTailToPublicArtifact'
  | PrivateResetArtifacts;

export type ProtocolArtifact = ServerProtocolArtifact | ClientProtocolArtifact;

export const ServerCircuitArtifacts: Record<ServerProtocolArtifact, NoirCompiledCircuit> = {
  EmptyNestedArtifact: EmptyNestedJson as NoirCompiledCircuit,
  PrivateKernelEmptyArtifact: PrivateKernelEmptyJson as NoirCompiledCircuit,
  PublicKernelSetupArtifact: PublicKernelSetupJson as NoirCompiledCircuit,
  PublicKernelAppLogicArtifact: PublicKernelAppLogicJson as NoirCompiledCircuit,
  PublicKernelTeardownArtifact: PublicKernelTeardownJson as NoirCompiledCircuit,
  PublicKernelTailArtifact: PublicKernelTailJson as NoirCompiledCircuit,
  BaseParityArtifact: BaseParityJson as NoirCompiledCircuit,
  RootParityArtifact: RootParityJson as NoirCompiledCircuit,
  BaseRollupArtifact: BaseRollupJson as NoirCompiledCircuit,
  MergeRollupArtifact: MergeRollupJson as NoirCompiledCircuit,
  RootRollupArtifact: RootRollupJson as NoirCompiledCircuit,
};

export const SimulatedServerCircuitArtifacts: Record<ServerProtocolArtifact, NoirCompiledCircuit> = {
  EmptyNestedArtifact: EmptyNestedSimulatedJson as NoirCompiledCircuit,
  PrivateKernelEmptyArtifact: PrivateKernelEmptySimulatedJson as NoirCompiledCircuit,
  PublicKernelSetupArtifact: PublicKernelSetupSimulatedJson as NoirCompiledCircuit,
  PublicKernelAppLogicArtifact: PublicKernelAppLogicSimulatedJson as NoirCompiledCircuit,
  PublicKernelTeardownArtifact: PublicKernelTeardownSimulatedJson as NoirCompiledCircuit,
  PublicKernelTailArtifact: PublicKernelTailSimulatedJson as NoirCompiledCircuit,
  BaseParityArtifact: BaseParityJson as NoirCompiledCircuit,
  RootParityArtifact: RootParityJson as NoirCompiledCircuit,
  BaseRollupArtifact: BaseRollupSimulatedJson as NoirCompiledCircuit,
  MergeRollupArtifact: MergeRollupJson as NoirCompiledCircuit,
  RootRollupArtifact: RootRollupJson as NoirCompiledCircuit,
};

export const ResetSimulatedArtifacts: Record<PrivateResetArtifacts, NoirCompiledCircuit> = {
  PrivateKernelResetFullArtifact: PrivateKernelResetSimulatedJson as NoirCompiledCircuit,
  PrivateKernelResetBigArtifact: PrivateKernelResetBigSimulatedJson as NoirCompiledCircuit,
  PrivateKernelResetMediumArtifact: PrivateKernelResetMediumSimulatedJson as NoirCompiledCircuit,
  PrivateKernelResetSmallArtifact: PrivateKernelResetSmallSimulatedJson as NoirCompiledCircuit,
};

export const ClientCircuitArtifacts: Record<ClientProtocolArtifact, NoirCompiledCircuit> = {
  PrivateKernelInitArtifact: PrivateKernelInitJson as NoirCompiledCircuit,
  PrivateKernelInnerArtifact: PrivateKernelInnerJson as NoirCompiledCircuit,
  PrivateKernelResetFullArtifact: PrivateKernelResetJson as NoirCompiledCircuit,
  PrivateKernelResetBigArtifact: PrivateKernelResetBigJson as NoirCompiledCircuit,
  PrivateKernelResetMediumArtifact: PrivateKernelResetMediumJson as NoirCompiledCircuit,
  PrivateKernelResetSmallArtifact: PrivateKernelResetSmallJson as NoirCompiledCircuit,
  PrivateKernelTailArtifact: PrivateKernelTailJson as NoirCompiledCircuit,
  PrivateKernelTailToPublicArtifact: PrivateKernelTailToPublicJson as NoirCompiledCircuit,
};

export const SimulatedClientCircuitArtifacts: Record<ClientProtocolArtifact, NoirCompiledCircuit> = {
  PrivateKernelInitArtifact: PrivateKernelInitSimulatedJson as NoirCompiledCircuit,
  PrivateKernelInnerArtifact: PrivateKernelInnerSimulatedJson as NoirCompiledCircuit,
  PrivateKernelTailArtifact: PrivateKernelTailSimulatedJson as NoirCompiledCircuit,
  PrivateKernelTailToPublicArtifact: PrivateKernelTailToPublicSimulatedJson as NoirCompiledCircuit,
  ...ResetSimulatedArtifacts,
};

export const ProtocolCircuitArtifacts: Record<ProtocolArtifact, NoirCompiledCircuit> = {
  ...ClientCircuitArtifacts,
  ...ServerCircuitArtifacts,
};
