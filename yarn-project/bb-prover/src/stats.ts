import { type PublicKernelRequest, PublicKernelType } from '@aztec/circuit-types';
import type { CircuitName } from '@aztec/circuit-types/stats';
import { type ClientProtocolArtifact, type ServerProtocolArtifact } from '@aztec/noir-protocol-circuits-types';

export function mapPublicKernelToCircuitName(kernelType: PublicKernelRequest['type']): CircuitName {
  switch (kernelType) {
    case PublicKernelType.SETUP:
      return 'public-kernel-setup';
    case PublicKernelType.APP_LOGIC:
      return 'public-kernel-app-logic';
    case PublicKernelType.TEARDOWN:
      return 'public-kernel-teardown';
    case PublicKernelType.TAIL:
      return 'public-kernel-tail';
    default:
      throw new Error(`Unknown kernel type: ${kernelType}`);
  }
}

export function mapProtocolArtifactNameToCircuitName(
  artifact: ServerProtocolArtifact | ClientProtocolArtifact,
): CircuitName {
  switch (artifact) {
    case 'BaseParityArtifact':
      return 'base-parity';
    case 'RootParityArtifact':
      return 'root-parity';
    case 'BaseRollupArtifact':
      return 'base-rollup';
    case 'MergeRollupArtifact':
      return 'merge-rollup';
    case 'RootRollupArtifact':
      return 'root-rollup';
    case 'PublicKernelSetupArtifact':
      return 'public-kernel-setup';
    case 'PublicKernelAppLogicArtifact':
      return 'public-kernel-app-logic';
    case 'PublicKernelTeardownArtifact':
      return 'public-kernel-teardown';
    case 'PublicKernelTailArtifact':
      return 'public-kernel-tail';
    case 'PrivateKernelInitArtifact':
      return 'private-kernel-init';
    case 'PrivateKernelInnerArtifact':
      return 'private-kernel-inner';
    case 'PrivateKernelTailArtifact':
      return 'private-kernel-tail';
    case 'PrivateKernelTailToPublicArtifact':
      return 'private-kernel-tail-to-public';
    case 'PrivateKernelResetFullArtifact':
      return 'private-kernel-reset-full';
    case 'PrivateKernelResetBigArtifact':
      return 'private-kernel-reset-big';
    case 'PrivateKernelResetMediumArtifact':
      return 'private-kernel-reset-medium';
    case 'PrivateKernelResetSmallArtifact':
      return 'private-kernel-reset-small';
    case 'EmptyNestedArtifact':
      return 'empty-nested';
    case 'PrivateKernelEmptyArtifact':
      return 'private-kernel-empty';
    default: {
      const _foo: never = artifact;
      throw new Error(`Unknown circuit type: ${artifact}`);
    }
  }
}
