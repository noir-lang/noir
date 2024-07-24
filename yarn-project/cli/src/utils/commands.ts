import { FunctionSelector } from '@aztec/aztec.js/abi';
import { AztecAddress } from '@aztec/aztec.js/aztec_address';
import { EthAddress } from '@aztec/aztec.js/eth_address';
import { Fr } from '@aztec/aztec.js/fields';
import { LogId } from '@aztec/aztec.js/log_id';
import { TxHash } from '@aztec/aztec.js/tx_hash';
import { type PXE } from '@aztec/circuit-types';
import { PublicKeys } from '@aztec/circuits.js';
import { type LogFn } from '@aztec/foundation/log';

import { type Command, CommanderError, InvalidArgumentError, Option } from 'commander';
import { lookup } from 'dns/promises';
import { rename, writeFile } from 'fs/promises';

/**
 * If we can successfully resolve 'host.docker.internal', then we are running in a container, and we should treat
 * localhost as being host.docker.internal.
 */
export const getLocalhost = () =>
  lookup('host.docker.internal')
    .then(() => 'host.docker.internal')
    .catch(() => 'localhost');

export const LOCALHOST = await getLocalhost();
export const { ETHEREUM_HOST = `http://${LOCALHOST}:8545`, PRIVATE_KEY, API_KEY, CLI_VERSION } = process.env;

export function addOptions(program: Command, options: Option[]) {
  options.forEach(option => program.addOption(option));
  return program;
}

export const pxeOption = new Option('-u, --rpc-url <string>', 'URL of the PXE')
  .env('PXE_URL')
  .default(`http://${LOCALHOST}:8080`)
  .makeOptionMandatory(true);

export const l1ChainIdOption = new Option('-c, --l1-chain-id <number>', 'Chain ID of the ethereum host')
  .env('L1_CHAIN_ID')
  .default(31337)
  .argParser(value => {
    const parsedValue = Number(value);
    if (isNaN(parsedValue)) {
      throw new Error('Chain ID must be a number.');
    }
    return parsedValue;
  });

export const createPrivateKeyOption = (description: string, mandatory: boolean) =>
  new Option('-pk, --private-key <string>', description)
    .env('PRIVATE_KEY')
    .argParser(parsePrivateKey)
    .makeOptionMandatory(mandatory);

export const logJson = (log: LogFn) => (obj: object) => log(JSON.stringify(obj, null, 2));

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
 * Removes the leading 0x from a hex string. If no leading 0x is found the string is returned unchanged.
 * @param hex - A hex string
 * @returns A new string with leading 0x removed
 */
const stripLeadingHex = (hex: string) => {
  if (hex.length > 2 && hex.startsWith('0x')) {
    return hex.substring(2);
  }
  return hex;
};

export function parseBigint(bigint: string): bigint | undefined {
  return bigint ? BigInt(bigint) : undefined;
}

/**
 * Parses a hex encoded string to an Fr integer
 * @param str - Hex encoded string
 * @returns A integer
 */
export function parseFieldFromHexString(str: string): Fr {
  const hex = stripLeadingHex(str);

  // ensure it's a hex string
  if (!hex.match(/^[0-9a-f]+$/i)) {
    throw new InvalidArgumentError('Invalid hex string');
  }

  // pad it so that we may read it as a buffer.
  // Buffer needs _exactly_ two hex characters per byte
  const padded = hex.length % 2 === 1 ? '0' + hex : hex;
  let buf = Buffer.from(padded, 'hex');
  if (buf.length > Fr.SIZE_IN_BYTES) {
    buf = buf.subarray(buf.length - Fr.SIZE_IN_BYTES);
  }

  const fr = Buffer.alloc(Fr.SIZE_IN_BYTES, 0);
  fr.set(buf, Fr.SIZE_IN_BYTES - buf.length);

  // finally, turn it into an integer
  return Fr.fromBuffer(fr);
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
 * @param publicKey - A public keys object serialised as a string
 * @returns A PublicKeys instance
 * @throws InvalidArgumentError if the input string is not valid.
 */
export function parsePublicKey(publicKey: string): PublicKeys | undefined {
  if (!publicKey) {
    return undefined;
  }
  try {
    return PublicKeys.fromString(publicKey);
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
export function parsePrivateKey(privateKey: string): Fr {
  try {
    return Fr.fromString(privateKey);
  } catch (err) {
    throw new InvalidArgumentError(`Invalid encryption private key: ${privateKey}`);
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
