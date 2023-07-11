import { mnemonicToAccount, privateKeyToAccount } from 'viem/accounts';
import fs from 'fs';
import { createEthereumChain, deployL1Contracts } from '@aztec/ethereum';
import { DebugLogger, Logger } from '@aztec/foundation/log';
import { ContractAbi } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/aztec.js';
import { encodeArgs } from './cli_encoder.js';

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
  await deployL1Contracts(chain.rpcUrl, account, chain.chainInfo, debugLogger);
}

/**
 * Reads a file and converts it to an Aztec Contract ABI.
 * @param fileDir - The directory of the compiled contract ABI.
 * @returns The parsed ContractABI.
 */
export function getContractAbi(fileDir: string, log: Logger) {
  const contents = fs.readFileSync(fileDir, 'utf8');
  let contractAbi: ContractAbi;
  try {
    contractAbi = JSON.parse(contents) as ContractAbi;
  } catch (err) {
    log('Invalid file used. Please try again.');
    throw err;
  }
  return contractAbi;
}

/**
 * Performs necessary checks, conversions & operations to call a contract fn from the CLI.
 * @param contractFile - Directory of the compiled contract ABI.
 * @param _contractAddress - Aztec Address of the contract.
 * @param functionName - Name of the function to be called.
 * @param _origin - The caller's address.
 * @param _functionArgs - Arguments to call the function with.
 * @param log - Logger instance that will output to the CLI
 * @returns Formatted contract address, function arguments and caller's aztec address.
 */
export function prepTx(
  contractFile: string,
  _contractAddress: string,
  functionName: string,
  _origin: string,
  _functionArgs: string[],
  log: Logger,
) {
  let contractAddress;
  try {
    contractAddress = AztecAddress.fromString(_contractAddress);
  } catch {
    throw new Error(`Unable to parse contract address ${_contractAddress}.`);
  }
  const contractAbi = getContractAbi(contractFile, log);
  const functionAbi = contractAbi.functions.find(({ name }) => name === functionName);
  if (!functionAbi) {
    throw new Error(`Function ${functionName} not found on contract ABI.`);
  }

  const functionArgs = encodeArgs(_functionArgs, functionAbi.parameters);
  let origin;
  if (_origin) {
    try {
      origin = AztecAddress.fromString(_origin);
    } catch {
      throw new Error(`Unable to parse caller address ${_origin}.`);
    }
  }

  return { contractAddress, functionArgs, origin, contractAbi };
}
