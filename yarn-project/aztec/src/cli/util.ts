import { ArchiverConfig } from '@aztec/archiver';
import { AztecNodeConfig } from '@aztec/aztec-node';
import { AccountManager } from '@aztec/aztec.js';
import { L1ContractAddresses } from '@aztec/ethereum';
import { EthAddress } from '@aztec/foundation/eth-address';
import { LogFn } from '@aztec/foundation/log';
import { P2PConfig } from '@aztec/p2p';
import { GrumpkinScalar, PXEService, PXEServiceConfig } from '@aztec/pxe';

/**
 * Checks if the object has l1Contracts property
 * @param obj - The object to check
 * @returns True if the object has l1Contracts property
 */
function hasL1Contracts(obj: any): obj is {
  /** the deployed L1 contract addresses */
  l1Contracts: unknown;
} {
  return 'l1Contracts' in obj;
}

/**
 * Checks if all contract addresses set in config.
 * @param contracts - L1 Contract Addresses object
 * @returns true if all contract addresses are not zero
 */
const checkContractAddresses = (contracts: L1ContractAddresses) => {
  return ['rollupAddress', 'inboxAddress', 'outboxAddress', 'contractDeploymentEmitterAddress'].every(cn => {
    const key = cn as keyof L1ContractAddresses;
    return contracts[key] && contracts[key] !== EthAddress.ZERO;
  });
};

export const installSignalHandlers = (logFn: LogFn, cb?: Array<() => Promise<void>>) => {
  const shutdown = async () => {
    logFn('Shutting down...');
    if (cb) {
      await Promise.all(cb);
    }
    process.exit(0);
  };
  process.removeAllListeners('SIGINT');
  process.removeAllListeners('SIGTERM');
  process.once('SIGINT', shutdown);
  process.once('SIGTERM', shutdown);
};

/**
 * Parses a string of options into a key-value map.
 * @param options - String of options in the format "option1=value1,option2=value2".
 * @returns Key-value map of options.
 */
export const parseModuleOptions = (options: string): Record<string, string> => {
  if (!options?.length) {
    return {};
  }
  const optionsArray = options.split(',');
  return optionsArray.reduce((acc, option) => {
    const [key, value] = option.split('=');
    return { ...acc, [key]: value };
  }, {});
};

export const mergeEnvVarsAndCliOptions = <T extends AztecNodeConfig | PXEServiceConfig | P2PConfig | ArchiverConfig>(
  envVars: AztecNodeConfig | PXEServiceConfig | P2PConfig | ArchiverConfig,
  cliOptions: Record<string, string>,
  contractsRequired = false,
) => {
  if (contractsRequired && !cliOptions.rollupAddress) {
    throw new Error('Rollup contract address is required to start the service');
  }
  const cliOptionsContracts: L1ContractAddresses = {
    rollupAddress: cliOptions.rollupAddress ? EthAddress.fromString(cliOptions.rollupAddress) : EthAddress.ZERO,
    registryAddress: cliOptions.registryAddress ? EthAddress.fromString(cliOptions.registryAddress) : EthAddress.ZERO,
    inboxAddress: cliOptions.inboxAddress ? EthAddress.fromString(cliOptions.inboxAddress) : EthAddress.ZERO,
    outboxAddress: cliOptions.outboxAddress ? EthAddress.fromString(cliOptions.outboxAddress) : EthAddress.ZERO,
    contractDeploymentEmitterAddress: cliOptions.contractDeploymentEmitterAddress
      ? EthAddress.fromString(cliOptions.contractDeploymentEmitterAddress)
      : EthAddress.ZERO,
    availabilityOracleAddress: cliOptions.availabilityOracleAddress
      ? EthAddress.fromString(cliOptions.availabilityOracleAddress)
      : EthAddress.ZERO,
  };

  if (
    hasL1Contracts(envVars) &&
    contractsRequired &&
    (!checkContractAddresses(cliOptionsContracts) || !checkContractAddresses(envVars.l1Contracts))
  ) {
    throw new Error('Deployed L1 contract addresses are required to start the service');
  }

  let merged = { ...envVars, ...cliOptions } as T;

  if (hasL1Contracts(envVars)) {
    merged = {
      ...merged,
      l1Contracts: {
        ...(envVars.l1Contracts && { ...envVars.l1Contracts }),
        ...cliOptionsContracts,
      },
    } as T;
  }

  return merged;
};

/**
 * Creates logs for the initial accounts
 * @param accounts - The initial accounts
 * @param pxe - A PXE instance to get the registered accounts
 * @returns A string array containing the initial accounts details
 */
export async function createAccountLogs(
  accounts: {
    /**
     * The account object
     */
    account: AccountManager;
    /**
     * The private key of the account
     */
    privateKey: GrumpkinScalar;
  }[],
  pxe: PXEService,
) {
  const registeredAccounts = await pxe.getRegisteredAccounts();
  const accountLogStrings = [`Initial Accounts:\n\n`];
  for (const account of accounts) {
    const completeAddress = account.account.getCompleteAddress();
    if (registeredAccounts.find(a => a.equals(completeAddress))) {
      accountLogStrings.push(` Address: ${completeAddress.address.toString()}\n`);
      accountLogStrings.push(` Partial Address: ${completeAddress.partialAddress.toString()}\n`);
      accountLogStrings.push(` Private Key: ${account.privateKey.toString()}\n`);
      accountLogStrings.push(` Public Key: ${completeAddress.publicKey.toString()}\n\n`);
    }
  }
  return accountLogStrings;
}
