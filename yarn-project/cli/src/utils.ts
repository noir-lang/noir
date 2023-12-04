import { AztecAddress, EthAddress, Fr, FunctionSelector, GrumpkinScalar, PXE, Point, TxHash } from '@aztec/aztec.js';
import { L1ContractArtifactsForDeployment, createEthereumChain, deployL1Contracts } from '@aztec/ethereum';
import { ContractArtifact } from '@aztec/foundation/abi';
import { DebugLogger, LogFn } from '@aztec/foundation/log';
import {
  ContractDeploymentEmitterAbi,
  ContractDeploymentEmitterBytecode,
  InboxAbi,
  InboxBytecode,
  OutboxAbi,
  OutboxBytecode,
  RegistryAbi,
  RegistryBytecode,
  RollupAbi,
  RollupBytecode,
} from '@aztec/l1-artifacts';
import { LogId } from '@aztec/types';

import { CommanderError, InvalidArgumentError } from 'commander';
import { readFile, rename, writeFile } from 'fs/promises';
import { mnemonicToAccount, privateKeyToAccount } from 'viem/accounts';

import { encodeArgs } from './encoding.js';

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
export function getFunctionArtifact(artifact: ContractArtifact, fnName: string) {
  const fn = artifact.functions.find(({ name }) => name === fnName);
  if (!fn) {
    throw Error(`Function ${fnName} not found in contract ABI.`);
  }
  return fn;
}

/**
 * Function to execute the 'deployRollupContracts' command.
 * @param rpcUrl - The RPC URL of the ethereum node.
 * @param apiKey - The api key of the ethereum node endpoint.
 * @param privateKey - The private key to be used in contract deployment.
 * @param mnemonic - The mnemonic to be used in contract deployment.
 */
export async function deployAztecContracts(
  rpcUrl: string,
  apiKey: string,
  privateKey: string,
  mnemonic: string,
  debugLogger: DebugLogger,
) {
  const account = !privateKey ? mnemonicToAccount(mnemonic!) : privateKeyToAccount(`0x${privateKey}`);
  const chain = createEthereumChain(rpcUrl, apiKey);
  const l1Artifacts: L1ContractArtifactsForDeployment = {
    contractDeploymentEmitter: {
      contractAbi: ContractDeploymentEmitterAbi,
      contractBytecode: ContractDeploymentEmitterBytecode,
    },
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
    rollup: {
      contractAbi: RollupAbi,
      contractBytecode: RollupBytecode,
    },
  };
  return await deployL1Contracts(chain.rpcUrl, account, chain.chainInfo, debugLogger, l1Artifacts);
}

/**
 * Gets all contracts available in \@aztec/noir-contracts.
 * @returns The contract ABIs.
 */
export async function getExampleContractArtifacts() {
  const artifacts: ArtifactsType = await import('@aztec/noir-contracts/artifacts');
  return artifacts;
}

/**
 * Reads a file and converts it to an Aztec Contract ABI.
 * @param fileDir - The directory of the compiled contract ABI.
 * @returns The parsed contract artifact.
 */
export async function getContractArtifact(fileDir: string, log: LogFn) {
  // first check if it's a noir-contracts example
  let contents: string;
  const artifacts = await getExampleContractArtifacts();
  if (artifacts[fileDir]) {
    return artifacts[fileDir] as ContractArtifact;
  }

  try {
    contents = await readFile(fileDir, 'utf8');
  } catch {
    throw Error(`Contract ${fileDir} not found`);
  }

  // if not found, try reading as path directly
  let contractArtifact: ContractArtifact;
  try {
    contractArtifact = JSON.parse(contents) as ContractArtifact;
  } catch (err) {
    log('Invalid file used. Please try again.');
    throw err;
  }
  return contractArtifact;
}

/**
 * Utility to select a TX sender either from user input
 * or from the first account that is found in a PXE instance.
 * @param pxe - The PXE instance that will be checked for an account.
 * @param _from - The user input.
 * @returns An Aztec address. Will throw if one can't be found in either options.
 */
export async function getTxSender(pxe: PXE, _from?: string) {
  let from: AztecAddress;
  if (_from) {
    try {
      from = AztecAddress.fromString(_from);
    } catch {
      throw new InvalidArgumentError(`Invalid option 'from' passed: ${_from}`);
    }
  } else {
    const accounts = await pxe.getRegisteredAccounts();
    if (!accounts.length) {
      throw new Error('No accounts found in PXE instance.');
    }
    from = accounts[0].address;
  }
  return from;
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
 * Parses a hex encoded string to an Fr integer to be used as salt
 * @param str - Hex encoded string
 * @returns A integer to be used as salt
 */
export function parseSaltFromHexString(str: string): Fr {
  const hex = stripLeadingHex(str);

  // ensure it's a hex string
  if (!hex.match(/^[0-9a-f]+$/i)) {
    throw new InvalidArgumentError('Invalid hex string');
  }

  // pad it so that we may read it as a buffer.
  // Buffer needs _exactly_ two hex characters per byte
  const padded = hex.length % 2 === 1 ? '0' + hex : hex;

  // finally, turn it into an integer
  return Fr.fromBuffer(Buffer.from(padded, 'hex'));
}

/**
 * Parses an AztecAddress from a string.
 * @param address - A serialized Aztec address
 * @returns An Aztec address
 * @throws InvalidArgumentError if the input string is not valid.
 */
export function parseAztecAddress(address: string): AztecAddress {
  try {
    return AztecAddress.fromString(address);
  } catch {
    throw new InvalidArgumentError(`Invalid address: ${address}`);
  }
}

/**
 * Parses an Ethereum address from a string.
 * @param address - A serialized Ethereum address
 * @returns An Ethereum address
 * @throws InvalidArgumentError if the input string is not valid.
 */
export function parseEthereumAddress(address: string): EthAddress {
  try {
    return EthAddress.fromString(address);
  } catch {
    throw new InvalidArgumentError(`Invalid address: ${address}`);
  }
}

/**
 * Parses an AztecAddress from a string.
 * @param address - A serialized Aztec address
 * @returns An Aztec address
 * @throws InvalidArgumentError if the input string is not valid.
 */
export function parseOptionalAztecAddress(address: string): AztecAddress | undefined {
  if (!address) {
    return undefined;
  }
  return parseAztecAddress(address);
}

/**
 * Parses an optional log ID string into a LogId object.
 *
 * @param logId - The log ID string to parse.
 * @returns The parsed LogId object, or undefined if the log ID is missing or empty.
 */
export function parseOptionalLogId(logId: string): LogId | undefined {
  if (!logId) {
    return undefined;
  }
  return LogId.fromString(logId);
}

/**
 * Parses a selector from a string.
 * @param selector - A serialized selector.
 * @returns A selector.
 * @throws InvalidArgumentError if the input string is not valid.
 */
export function parseOptionalSelector(selector: string): FunctionSelector | undefined {
  if (!selector) {
    return undefined;
  }
  try {
    return FunctionSelector.fromString(selector);
  } catch {
    throw new InvalidArgumentError(`Invalid selector: ${selector}`);
  }
}

/**
 * Parses a string into an integer or returns undefined if the input is falsy.
 *
 * @param value - The string to parse into an integer.
 * @returns The parsed integer, or undefined if the input string is falsy.
 * @throws If the input is not a valid integer.
 */
export function parseOptionalInteger(value: string): number | undefined {
  if (!value) {
    return undefined;
  }
  const parsed = Number(value);
  if (!Number.isInteger(parsed)) {
    throw new InvalidArgumentError('Invalid integer.');
  }
  return parsed;
}

/**
 * Parses a TxHash from a string.
 * @param txHash - A transaction hash
 * @returns A TxHash instance
 * @throws InvalidArgumentError if the input string is not valid.
 */
export function parseTxHash(txHash: string): TxHash {
  try {
    return TxHash.fromString(txHash);
  } catch {
    throw new InvalidArgumentError(`Invalid transaction hash: ${txHash}`);
  }
}

/**
 * Parses an optional TxHash from a string.
 * Calls parseTxHash internally.
 * @param txHash - A transaction hash
 * @returns A TxHash instance, or undefined if the input string is falsy.
 * @throws InvalidArgumentError if the input string is not valid.
 */
export function parseOptionalTxHash(txHash: string): TxHash | undefined {
  if (!txHash) {
    return undefined;
  }
  return parseTxHash(txHash);
}

/**
 * Parses a public key from a string.
 * @param publicKey - A public key
 * @returns A Point instance
 * @throws InvalidArgumentError if the input string is not valid.
 */
export function parsePublicKey(publicKey: string): Point {
  try {
    return Point.fromString(publicKey);
  } catch (err) {
    throw new InvalidArgumentError(`Invalid public key: ${publicKey}`);
  }
}

/**
 * Parses a partial address from a string.
 * @param address - A partial address
 * @returns A Fr instance
 * @throws InvalidArgumentError if the input string is not valid.
 */
export function parsePartialAddress(address: string): Fr {
  try {
    return Fr.fromString(address);
  } catch (err) {
    throw new InvalidArgumentError(`Invalid partial address: ${address}`);
  }
}

/**
 * Parses a private key from a string.
 * @param privateKey - A string
 * @returns A private key
 * @throws InvalidArgumentError if the input string is not valid.
 */
export function parsePrivateKey(privateKey: string): GrumpkinScalar {
  try {
    const value = GrumpkinScalar.fromString(privateKey);
    // most likely a badly formatted key was passed
    if (value.isZero()) {
      throw new Error('Private key must not be zero');
    }

    return value;
  } catch (err) {
    throw new InvalidArgumentError(`Invalid private key: ${privateKey}`);
  }
}

/**
 * Parses a field from a string.
 * @param field - A string representing the field.
 * @returns A field.
 * @throws InvalidArgumentError if the input string is not valid.
 */
export function parseField(field: string): Fr {
  try {
    const isHex = field.startsWith('0x') || field.match(new RegExp(`^[0-9a-f]{${Fr.SIZE_IN_BYTES * 2}}$`, 'i'));
    if (isHex) {
      return Fr.fromString(field);
    }

    if (['true', 'false'].includes(field)) {
      return new Fr(field === 'true');
    }

    const isNumber = +field || field === '0';
    if (isNumber) {
      return new Fr(BigInt(field));
    }

    const isBigInt = field.endsWith('n');
    if (isBigInt) {
      return new Fr(BigInt(field.replace(/n$/, '')));
    }

    return new Fr(BigInt(field));
  } catch (err) {
    throw new InvalidArgumentError(`Invalid field: ${field}`);
  }
}

/**
 * Parses an array of strings to Frs.
 * @param fields - An array of strings representing the fields.
 * @returns An array of Frs.
 */
export function parseFields(fields: string[]): Fr[] {
  return fields.map(parseField);
}

/**
 * Updates a file in place atomically.
 * @param filePath - Path to file
 * @param contents - New contents to write
 */
export async function atomicUpdateFile(filePath: string, contents: string) {
  const tmpFilepath = filePath + '.tmp';
  try {
    await writeFile(tmpFilepath, contents, {
      // let's crash if the tmp file already exists
      flag: 'wx',
    });
    await rename(tmpFilepath, filePath);
  } catch (e) {
    if (e instanceof Error && 'code' in e && e.code === 'EEXIST') {
      const commanderError = new CommanderError(
        1,
        e.code,
        `Temporary file already exists: ${tmpFilepath}. Delete this file and try again.`,
      );
      commanderError.nestedError = e.message;
      throw commanderError;
    } else {
      throw e;
    }
  }
}
