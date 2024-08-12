import { type AztecAddress } from '@aztec/circuits.js';
import { parseAztecAddress, parseSecretKey, parseTxHash } from '@aztec/cli/utils';

import { Option } from 'commander';
import { readdir, stat } from 'fs/promises';

import { type AliasType, type WalletDB } from '../../storage/wallet_db.js';
import { AccountTypes } from '../accounts.js';

const TARGET_DIR = 'target';

export const ARTIFACT_DESCRIPTION =
  "Path to a compiled Aztec contract's artifact in JSON format. If executed inside a nargo workspace, a package and contract name can be specified as package@contract";

export function aliasedTxHashParser(txHash: string, db?: WalletDB) {
  try {
    return parseTxHash(txHash);
  } catch (err) {
    const prefixed = txHash.includes(':') ? txHash : `transactions:${txHash}`;
    const rawTxHash = db ? db.tryRetrieveAlias(prefixed) : txHash;
    return parseTxHash(rawTxHash);
  }
}

export function aliasedAddressParser(defaultPrefix: AliasType, address: string, db?: WalletDB) {
  if (address.startsWith('0x')) {
    return parseAztecAddress(address);
  } else {
    const prefixed = address.includes(':') ? address : `${defaultPrefix}:${address}`;
    const rawAddress = db ? db.tryRetrieveAlias(prefixed) : address;
    return parseAztecAddress(rawAddress);
  }
}

export function aliasedSecretKeyParser(sk: string, db?: WalletDB) {
  if (sk.startsWith('0x')) {
    return parseSecretKey(sk);
  } else {
    const prefixed = `${sk.startsWith('accounts') ? '' : 'accounts'}:${sk.endsWith(':sk') ? sk : `${sk}:sk`}`;
    const rawSk = db ? db.tryRetrieveAlias(prefixed) : sk;
    return parseSecretKey(rawSk);
  }
}

export function createAliasOption(description: string, hide: boolean) {
  return new Option(`-a, --alias <string>`, description).hideHelp(hide);
}

export function createAccountOption(description: string, hide: boolean, db?: WalletDB) {
  return new Option(`-f, --from <string>`, description)
    .hideHelp(hide)
    .argParser(address => aliasedAddressParser('accounts', address, db));
}

export function createTypeOption(mandatory: boolean) {
  return new Option('-t, --type <string>', 'Type of account to create')
    .choices(AccountTypes)
    .default('schnorr')
    .conflicts('account-or-address')
    .makeOptionMandatory(mandatory);
}

export function createArgsOption(isConstructor: boolean, db?: WalletDB) {
  return new Option('--args [args...]', `${isConstructor ? 'Constructor' : 'Function'}  arguments`)
    .argParser((arg, prev: string[]) => {
      const next = db?.tryRetrieveAlias(arg) || arg;
      prev.push(next);
      return prev;
    })
    .default([]);
}

export function createContractAddressOption(db?: WalletDB) {
  return new Option('-ca, --contract-address <address>', 'Aztec address of the contract.')
    .argParser(address => aliasedAddressParser('contracts', address, db))
    .makeOptionMandatory(true);
}

export function artifactPathParser(filePath: string, db?: WalletDB) {
  if (filePath.includes('@')) {
    const [pkg, contractName] = filePath.split('@');
    return contractArtifactFromWorkspace(pkg, contractName);
  } else if (!new RegExp(/^(\.|\/|[A-Z]:).*\.json$/).test(filePath)) {
    filePath = db ? db.tryRetrieveAlias(`artifacts:${filePath}`) : filePath;
  }
  if (!filePath) {
    throw new Error(
      'This command has to be called from a nargo workspace or contract artifact path should be provided',
    );
  }
  return Promise.resolve(filePath);
}

export async function artifactPathFromPromiseOrAlias(
  artifactPathPromise: Promise<string>,
  contractAddress: AztecAddress,
  db?: WalletDB,
) {
  let artifactPath = await artifactPathPromise;

  if (db && !artifactPath) {
    artifactPath = db.tryRetrieveAlias(`artifacts:${contractAddress.toString()}`);
    if (!artifactPath) {
      throw new Error(`No artifact found for contract address ${contractAddress}, please provide it via the -c option`);
    }
  }
  return artifactPath;
}

export function createArtifactOption(db?: WalletDB) {
  return new Option('-c, --contract-artifact <fileLocation>', ARTIFACT_DESCRIPTION)
    .argParser(filePath => artifactPathParser(filePath, db))
    .makeOptionMandatory(false);
}

async function contractArtifactFromWorkspace(pkg?: string, contractName?: string) {
  const cwd = process.cwd();
  try {
    await stat(`${cwd}/Nargo.toml`);
  } catch (e) {
    throw new Error(
      'Invalid contract artifact argument provided. To use this option, command should be called from a nargo workspace',
    );
  }
  const filesInTarget = await readdir(`${cwd}/${TARGET_DIR}`);
  const bestMatch = filesInTarget.filter(file => {
    if (pkg && contractName) {
      return file === `${pkg}-${contractName}.json`;
    } else {
      return file.endsWith('.json') && (file.includes(pkg || '') || file.includes(contractName || ''));
    }
  });
  if (bestMatch.length === 0) {
    throw new Error('No contract artifacts found in target directory with the specified criteria');
  } else if (bestMatch.length > 1) {
    throw new Error(
      `Multiple contract artifacts found in target directory with the specified criteria ${bestMatch.join(', ')}`,
    );
  }
  return `${cwd}/${TARGET_DIR}/${bestMatch[0]}`;
}

export * from './fees.js';
