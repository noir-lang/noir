import { type DebugLogger, type L1ContractArtifactsForDeployment, deployL1Contracts } from '@aztec/aztec.js';
import {
  AvailabilityOracleAbi,
  AvailabilityOracleBytecode,
  FeeJuicePortalAbi,
  FeeJuicePortalBytecode,
  InboxAbi,
  InboxBytecode,
  OutboxAbi,
  OutboxBytecode,
  PortalERC20Abi,
  PortalERC20Bytecode,
  RegistryAbi,
  RegistryBytecode,
  RollupAbi,
  RollupBytecode,
} from '@aztec/l1-artifacts';
import { getVKTreeRoot } from '@aztec/noir-protocol-circuits-types';
import { FeeJuiceAddress } from '@aztec/protocol-contracts/fee-juice';

import { type HDAccount, type PrivateKeyAccount } from 'viem';
import { foundry } from 'viem/chains';

export { deployAndInitializeTokenAndBridgeContracts } from '../shared/cross_chain_test_harness.js';

export const setupL1Contracts = async (
  l1RpcUrl: string,
  account: HDAccount | PrivateKeyAccount,
  logger: DebugLogger,
) => {
  const l1Artifacts: L1ContractArtifactsForDeployment = {
    registry: {
      contractAbi: RegistryAbi,
      contractBytecode: RegistryBytecode,
    },
    inbox: {
      contractAbi: InboxAbi,
      contractBytecode: InboxBytecode,
    },
    outbox: {
      contractAbi: OutboxAbi,
      contractBytecode: OutboxBytecode,
    },
    availabilityOracle: {
      contractAbi: AvailabilityOracleAbi,
      contractBytecode: AvailabilityOracleBytecode,
    },
    rollup: {
      contractAbi: RollupAbi,
      contractBytecode: RollupBytecode,
    },
    feeJuice: {
      contractAbi: PortalERC20Abi,
      contractBytecode: PortalERC20Bytecode,
    },
    feeJuicePortal: {
      contractAbi: FeeJuicePortalAbi,
      contractBytecode: FeeJuicePortalBytecode,
    },
  };

  const l1Data = await deployL1Contracts(l1RpcUrl, account, foundry, logger, l1Artifacts, {
    l2FeeJuiceAddress: FeeJuiceAddress,
    vkTreeRoot: getVKTreeRoot(),
  });

  return l1Data;
};
