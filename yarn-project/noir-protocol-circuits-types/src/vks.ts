import {
  BASE_PARITY_INDEX,
  BASE_ROLLUP_INDEX,
  EMPTY_NESTED_INDEX,
  Fr,
  MERGE_ROLLUP_INDEX,
  type MerkleTree,
  MerkleTreeCalculator,
  PRIVATE_KERNEL_EMPTY_INDEX,
  PRIVATE_KERNEL_INIT_INDEX,
  PRIVATE_KERNEL_INNER_INDEX,
  PRIVATE_KERNEL_RESET_BIG_INDEX,
  PRIVATE_KERNEL_RESET_FULL_INDEX,
  PRIVATE_KERNEL_RESET_MEDIUM_INDEX,
  PRIVATE_KERNEL_RESET_SMALL_INDEX,
  PRIVATE_KERNEL_TAIL_INDEX,
  PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX,
  PUBLIC_KERNEL_APP_LOGIC_INDEX,
  PUBLIC_KERNEL_SETUP_INDEX,
  PUBLIC_KERNEL_TAIL_INDEX,
  PUBLIC_KERNEL_TEARDOWN_INDEX,
  ROOT_PARITY_INDEX,
  ROOT_ROLLUP_INDEX,
  VERIFICATION_KEY_LENGTH_IN_FIELDS,
  VK_TREE_HEIGHT,
  VerificationKeyAsFields,
  VerificationKeyData,
} from '@aztec/circuits.js';
import { assertLength } from '@aztec/foundation/serialize';

import EmptyNestedVkJson from '../artifacts/keys/empty_nested.vk.data.json' assert { type: 'json' };
import BaseParityVkJson from '../artifacts/keys/parity_base.vk.data.json' assert { type: 'json' };
import RootParityVkJson from '../artifacts/keys/parity_root.vk.data.json' assert { type: 'json' };
import PrivateKernelEmptyVkJson from '../artifacts/keys/private_kernel_empty.vk.data.json' assert { type: 'json' };
import PrivateKernelInitVkJson from '../artifacts/keys/private_kernel_init.vk.data.json' assert { type: 'json' };
import PrivateKernelInnerVkJson from '../artifacts/keys/private_kernel_inner.vk.data.json' assert { type: 'json' };
import PrivateKernelResetFullVkJson from '../artifacts/keys/private_kernel_reset.vk.data.json' assert { type: 'json' };
import PrivateKernelResetBigVkJson from '../artifacts/keys/private_kernel_reset_big.vk.data.json' assert { type: 'json' };
import PrivateKernelResetMediumVkJson from '../artifacts/keys/private_kernel_reset_medium.vk.data.json' assert { type: 'json' };
import PrivateKernelResetSmallVkJson from '../artifacts/keys/private_kernel_reset_small.vk.data.json' assert { type: 'json' };
import PrivateKernelTailVkJson from '../artifacts/keys/private_kernel_tail.vk.data.json' assert { type: 'json' };
import PrivateKernelTailToPublicVkJson from '../artifacts/keys/private_kernel_tail_to_public.vk.data.json' assert { type: 'json' };
import PublicKernelAppLogicVkJson from '../artifacts/keys/public_kernel_app_logic.vk.data.json' assert { type: 'json' };
import PublicKernelSetupVkJson from '../artifacts/keys/public_kernel_setup.vk.data.json' assert { type: 'json' };
import PublicKernelTailVkJson from '../artifacts/keys/public_kernel_tail.vk.data.json' assert { type: 'json' };
import PublicKernelTeardownVkJson from '../artifacts/keys/public_kernel_teardown.vk.data.json' assert { type: 'json' };
import BaseRollupVkJson from '../artifacts/keys/rollup_base.vk.data.json' assert { type: 'json' };
import MergeRollupVkJson from '../artifacts/keys/rollup_merge.vk.data.json' assert { type: 'json' };
import RootRollupVkJson from '../artifacts/keys/rollup_root.vk.data.json' assert { type: 'json' };
import { type ClientProtocolArtifact, type ProtocolArtifact, type ServerProtocolArtifact } from './artifacts.js';

interface VkJson {
  keyAsBytes: string;
  keyAsFields: string[];
}

function keyJsonToVKData(json: VkJson): VerificationKeyData {
  const { keyAsBytes, keyAsFields } = json;
  return new VerificationKeyData(
    new VerificationKeyAsFields(
      assertLength(
        keyAsFields.map((str: string) => new Fr(Buffer.from(str.slice(2), 'hex'))),
        VERIFICATION_KEY_LENGTH_IN_FIELDS,
      ),
      // TODO(#7410) what should be the vk hash here?
      new Fr(Buffer.from(keyAsFields[0].slice(2), 'hex')),
    ),
    Buffer.from(keyAsBytes, 'hex'),
  );
}

const ServerCircuitVks: Record<ServerProtocolArtifact, VerificationKeyData> = {
  EmptyNestedArtifact: keyJsonToVKData(EmptyNestedVkJson),
  PrivateKernelEmptyArtifact: keyJsonToVKData(PrivateKernelEmptyVkJson),
  PublicKernelSetupArtifact: keyJsonToVKData(PublicKernelSetupVkJson),
  PublicKernelAppLogicArtifact: keyJsonToVKData(PublicKernelAppLogicVkJson),
  PublicKernelTeardownArtifact: keyJsonToVKData(PublicKernelTeardownVkJson),
  PublicKernelTailArtifact: keyJsonToVKData(PublicKernelTailVkJson),
  BaseParityArtifact: keyJsonToVKData(BaseParityVkJson),
  RootParityArtifact: keyJsonToVKData(RootParityVkJson),
  BaseRollupArtifact: keyJsonToVKData(BaseRollupVkJson),
  MergeRollupArtifact: keyJsonToVKData(MergeRollupVkJson),
  RootRollupArtifact: keyJsonToVKData(RootRollupVkJson),
};

const ClientCircuitVks: Record<ClientProtocolArtifact, VerificationKeyData> = {
  PrivateKernelInitArtifact: keyJsonToVKData(PrivateKernelInitVkJson),
  PrivateKernelInnerArtifact: keyJsonToVKData(PrivateKernelInnerVkJson),
  PrivateKernelResetFullArtifact: keyJsonToVKData(PrivateKernelResetFullVkJson),
  PrivateKernelResetBigArtifact: keyJsonToVKData(PrivateKernelResetBigVkJson),
  PrivateKernelResetMediumArtifact: keyJsonToVKData(PrivateKernelResetMediumVkJson),
  PrivateKernelResetSmallArtifact: keyJsonToVKData(PrivateKernelResetSmallVkJson),
  PrivateKernelTailArtifact: keyJsonToVKData(PrivateKernelTailVkJson),
  PrivateKernelTailToPublicArtifact: keyJsonToVKData(PrivateKernelTailToPublicVkJson),
};

export const ProtocolCircuitVks: Record<ProtocolArtifact, VerificationKeyData> = {
  ...ClientCircuitVks,
  ...ServerCircuitVks,
};

export const ProtocolCircuitVkIndexes: Record<ProtocolArtifact, number> = {
  EmptyNestedArtifact: EMPTY_NESTED_INDEX,
  PrivateKernelEmptyArtifact: PRIVATE_KERNEL_EMPTY_INDEX,
  PrivateKernelInitArtifact: PRIVATE_KERNEL_INIT_INDEX,
  PrivateKernelInnerArtifact: PRIVATE_KERNEL_INNER_INDEX,
  PrivateKernelResetFullArtifact: PRIVATE_KERNEL_RESET_FULL_INDEX,
  PrivateKernelResetBigArtifact: PRIVATE_KERNEL_RESET_BIG_INDEX,
  PrivateKernelResetMediumArtifact: PRIVATE_KERNEL_RESET_MEDIUM_INDEX,
  PrivateKernelResetSmallArtifact: PRIVATE_KERNEL_RESET_SMALL_INDEX,
  PrivateKernelTailArtifact: PRIVATE_KERNEL_TAIL_INDEX,
  PrivateKernelTailToPublicArtifact: PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX,
  PublicKernelSetupArtifact: PUBLIC_KERNEL_SETUP_INDEX,
  PublicKernelAppLogicArtifact: PUBLIC_KERNEL_APP_LOGIC_INDEX,
  PublicKernelTeardownArtifact: PUBLIC_KERNEL_TEARDOWN_INDEX,
  PublicKernelTailArtifact: PUBLIC_KERNEL_TAIL_INDEX,
  BaseParityArtifact: BASE_PARITY_INDEX,
  RootParityArtifact: ROOT_PARITY_INDEX,
  BaseRollupArtifact: BASE_ROLLUP_INDEX,
  MergeRollupArtifact: MERGE_ROLLUP_INDEX,
  RootRollupArtifact: ROOT_ROLLUP_INDEX,
};

function buildVKTree() {
  const calculator = new MerkleTreeCalculator(VK_TREE_HEIGHT);
  const vkHashes = new Array(2 ** VK_TREE_HEIGHT).fill(Buffer.alloc(32));

  for (const [key, value] of Object.entries(ProtocolCircuitVks)) {
    const index = ProtocolCircuitVkIndexes[key as ProtocolArtifact];
    vkHashes[index] = value.keyAsFields.hash.toBuffer();
  }

  return calculator.computeTree(vkHashes);
}

let vkTree: MerkleTree | undefined;

export function getVKTree() {
  if (!vkTree) {
    vkTree = buildVKTree();
  }
  return vkTree;
}

export function getVKTreeRoot() {
  return Fr.fromBuffer(getVKTree().root);
}

export function getVKIndex(vk: VerificationKeyData | VerificationKeyAsFields | Fr) {
  let hash;
  if (vk instanceof VerificationKeyData) {
    hash = vk.keyAsFields.hash;
  } else if (vk instanceof VerificationKeyAsFields) {
    hash = vk.hash;
  } else {
    hash = vk;
  }

  const index = getVKTree().getIndex(hash.toBuffer());
  if (index < 0) {
    //throw new Error(`VK index for ${hash.toString()} not found in VK tree`);
    return 0; // faked for now
  }
  return index;
}

export function getVKSiblingPath(vkIndex: number) {
  return assertLength<Fr, typeof VK_TREE_HEIGHT>(
    getVKTree()
      .getSiblingPath(vkIndex)
      .map(buf => new Fr(buf)),
    VK_TREE_HEIGHT,
  );
}
