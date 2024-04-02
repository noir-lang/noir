import { type ArchiverConfig } from '@aztec/archiver';
import { type AztecNodeConfig } from '@aztec/aztec-node';
import { type AccountManager } from '@aztec/aztec.js';
import { type L1ContractAddresses, l1ContractsNames } from '@aztec/ethereum';
import { EthAddress } from '@aztec/foundation/eth-address';
import { type LogFn, createConsoleLogger } from '@aztec/foundation/log';
import { type P2PConfig } from '@aztec/p2p';
import { type GrumpkinScalar, type PXEService, type PXEServiceConfig } from '@aztec/pxe';

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
  return l1ContractsNames.every(cn => {
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
  userLog = createConsoleLogger(),
) => {
  let merged = { ...envVars, ...cliOptions } as T;

  if (hasL1Contracts(envVars)) {
    // create options object for L1 contract addresses
    const l1Contracts: L1ContractAddresses = l1ContractsNames.reduce((acc, cn) => {
      const key = cn as keyof L1ContractAddresses;
      if (cliOptions[key]) {
        return { ...acc, [key]: EthAddress.fromString(cliOptions[key]) };
      } else {
        return { ...acc, [key]: envVars.l1Contracts[key] };
      }
    }, {} as L1ContractAddresses);

    if (contractsRequired && !checkContractAddresses(l1Contracts)) {
      userLog('Deployed L1 contract addresses are required to start the service');
      throw new Error('Deployed L1 contract addresses are required to start the service');
    }

    merged = {
      ...merged,
      l1Contracts,
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
