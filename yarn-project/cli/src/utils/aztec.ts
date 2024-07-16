import { type ContractArtifact, type FunctionArtifact, loadContractArtifact } from '@aztec/aztec.js/abi';
import { type L1ContractArtifactsForDeployment } from '@aztec/aztec.js/ethereum';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';
import { type NoirPackageConfig } from '@aztec/foundation/noir';
import { GasTokenAddress } from '@aztec/protocol-contracts/gas-token';

import TOML from '@iarna/toml';
import { readFile } from 'fs/promises';

import { encodeArgs } from '../encoding.js';

/**
 * Helper type to dynamically import contracts.
 */
interface ArtifactsType {
  [key: string]: ContractArtifact;
}

/**
 * Helper to get an ABI function or throw error if it doesn't exist.
 * @param artifact - Contract's build artifact in JSON format.
 * @param fnName - Function name to be found.
 * @returns The function's ABI.
 */
export function getFunctionArtifact(artifact: ContractArtifact, fnName: string): FunctionArtifact {
  const fn = artifact.functions.find(({ name }) => name === fnName);
  if (!fn) {
    throw Error(`Function ${fnName} not found in contract ABI.`);
  }
  return fn;
}

/**
 * Function to execute the 'deployRollupContracts' command.
 * @param rpcUrl - The RPC URL of the ethereum node.
 * @param chainId - The chain ID of the L1 host.
 * @param privateKey - The private key to be used in contract deployment.
 * @param mnemonic - The mnemonic to be used in contract deployment.
 */
export async function deployAztecContracts(
  rpcUrl: string,
  chainId: number,
  privateKey: string,
  mnemonic: string,
  debugLogger: DebugLogger,
) {
  const {
    InboxAbi,
    InboxBytecode,
    OutboxAbi,
    OutboxBytecode,
    RegistryAbi,
    RegistryBytecode,
    RollupAbi,
    RollupBytecode,
    AvailabilityOracleAbi,
    AvailabilityOracleBytecode,
    GasPortalAbi,
    GasPortalBytecode,
    PortalERC20Abi,
    PortalERC20Bytecode,
  } = await import('@aztec/l1-artifacts');
  const { createEthereumChain, deployL1Contracts } = await import('@aztec/ethereum');
  const { mnemonicToAccount, privateKeyToAccount } = await import('viem/accounts');

  const account = !privateKey
    ? mnemonicToAccount(mnemonic!)
    : privateKeyToAccount(`${privateKey.startsWith('0x') ? '' : '0x'}${privateKey}` as `0x${string}`);
  const chain = createEthereumChain(rpcUrl, chainId);
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
  const { getVKTreeRoot } = await import('@aztec/noir-protocol-circuits-types');

  return await deployL1Contracts(chain.rpcUrl, account, chain.chainInfo, debugLogger, l1Artifacts, {
    l2GasTokenAddress: GasTokenAddress,
    vkTreeRoot: getVKTreeRoot(),
  });
}

/**
 * Gets all contracts available in \@aztec/noir-contracts.js.
 * @returns The contract ABIs.
 */
export async function getExampleContractArtifacts(): Promise<ArtifactsType> {
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore - Importing noir-contracts.js even in devDeps results in a circular dependency error. Need to ignore because this line doesn't cause an error in a dev environment
  const imports = await import('@aztec/noir-contracts.js');
  return Object.fromEntries(Object.entries(imports).filter(([key]) => key.endsWith('Artifact'))) as any;
}

/**
 * Reads a file and converts it to an Aztec Contract ABI.
 * @param fileDir - The directory of the compiled contract ABI.
 * @returns The parsed contract artifact.
 */
export async function getContractArtifact(fileDir: string, log: LogFn) {
  // first check if it's a noir-contracts example
  const artifacts = await getExampleContractArtifacts();
  for (const key of [fileDir, fileDir + 'Artifact', fileDir + 'ContractArtifact']) {
    if (artifacts[key]) {
      return artifacts[key] as ContractArtifact;
    }
  }

  let contents: string;
  try {
    contents = await readFile(fileDir, 'utf8');
  } catch {
    throw Error(`Contract ${fileDir} not found`);
  }

  try {
    return loadContractArtifact(JSON.parse(contents));
  } catch (err) {
    log('Invalid file used. Please try again.');
    throw err;
  }
}

/**
 * Performs necessary checks, conversions & operations to call a contract fn from the CLI.
 * @param contractFile - Directory of the compiled contract ABI.
 * @param functionName - Name of the function to be called.
 * @param _functionArgs - Arguments to call the function with.
 * @param log - Logger instance that will output to the CLI
 * @returns Formatted contract address, function arguments and caller's aztec address.
 */
export async function prepTx(contractFile: string, functionName: string, _functionArgs: string[], log: LogFn) {
  const contractArtifact = await getContractArtifact(contractFile, log);
  const functionArtifact = getFunctionArtifact(contractArtifact, functionName);
  const functionArgs = encodeArgs(_functionArgs, functionArtifact.parameters);

  return { functionArgs, contractArtifact };
}

/**
 * Removes the leading 0x from a hex string. If no leading 0x is found the string is returned unchanged.
 * @param hex - A hex string
 * @returns A new string with leading 0x removed
 */
export const stripLeadingHex = (hex: string) => {
  if (hex.length > 2 && hex.startsWith('0x')) {
    return hex.substring(2);
  }
  return hex;
};

/**
 * Pretty prints Nargo.toml contents to a string
 * @param config - Nargo.toml contents
 * @returns The Nargo.toml contents as a string
 */
export function prettyPrintNargoToml(config: NoirPackageConfig): string {
  const withoutDependencies = Object.fromEntries(Object.entries(config).filter(([key]) => key !== 'dependencies'));

  const partialToml = TOML.stringify(withoutDependencies);
  const dependenciesToml = Object.entries(config.dependencies).map(([name, dep]) => {
    const depToml = TOML.stringify.value(dep);
    return `${name} = ${depToml}`;
  });

  return partialToml + '\n[dependencies]\n' + dependenciesToml.join('\n') + '\n';
}
