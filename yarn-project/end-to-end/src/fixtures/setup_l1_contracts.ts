import {
  type DebugLogger,
  type DeployL1Contracts,
  type L1ContractArtifactsForDeployment,
  deployL1Contracts,
} from '@aztec/aztec.js';
import {
  AvailabilityOracleAbi,
  AvailabilityOracleBytecode,
  GasPortalAbi,
  GasPortalBytecode,
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
import { getCanonicalGasTokenAddress } from '@aztec/protocol-contracts/gas-token';

import { type HDAccount, type PrivateKeyAccount, getContract } from 'viem';
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
    gasToken: {
      contractAbi: PortalERC20Abi,
      contractBytecode: PortalERC20Bytecode,
    },
    gasPortal: {
      contractAbi: GasPortalAbi,
      contractBytecode: GasPortalBytecode,
    },
  };

  const l1Data = await deployL1Contracts(l1RpcUrl, account, foundry, logger, l1Artifacts);
  await initGasBridge(l1Data);

  return l1Data;
};

async function initGasBridge({ walletClient, l1ContractAddresses }: DeployL1Contracts) {
  const gasPortal = getContract({
    address: l1ContractAddresses.gasPortalAddress.toString(),
    abi: GasPortalAbi,
    client: walletClient,
  });

  await gasPortal.write.initialize(
    [
      l1ContractAddresses.registryAddress.toString(),
      l1ContractAddresses.gasTokenAddress.toString(),
      getCanonicalGasTokenAddress(l1ContractAddresses.gasPortalAddress).toString(),
    ],
    {} as any,
  );
}
