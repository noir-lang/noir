/*
  Copyright (c) 2019 xf00f

  This file is part of web3x and is released under the MIT License.
  https://opensource.org/licenses/MIT
*/

import { EthAddress } from '@aztec/foundation';
import { EthereumRpc } from '../../../eth_rpc/index.js';
import { bufferToHex } from '../../../hex_string/index.js';
import { JsonRpcProvider } from '../../../provider/json_rpc_provider.js';
import { ContractAbiDefinition } from '../../abi/index.js';

/**
 * Get the API host for a given Ethereum network.
 * The function maps an input network string to its corresponding etherscan.io API host URL.
 * Supported networks are 'mainnet', 'kovan', and 'ropsten'. Throws an error if an unknown network is provided.
 *
 * @param net - The Ethereum network string (e.g., 'mainnet', 'kovan', or 'ropsten').
 * @returns The etherscan.io API host URL for the specified network.
 */
function getApiHost(net: string) {
  switch (net) {
    case 'mainnet':
      return 'api.etherscan.io';
    case 'kovan':
      return 'api-kovan.etherscan.io';
    case 'ropsten':
      return 'api-ropsten.etherscan.io';
    default:
      throw new Error(`Unknown network ${net}`);
  }
}

/**
 * Fetches the Contract Application Binary Interface (ABI) for a given Ethereum contract address from Etherscan API.
 * The ABI is essential for interacting with smart contracts and decoding transactions in Ethereum.
 *
 * @param net - The Ethereum network identifier, such as 'mainnet', 'kovan', or 'ropsten'.
 * @param address - The Ethereum contract address to fetch the ABI for.
 * @param apiKey - The Etherscan API key for accessing their services.
 * @returns A Promise that resolves to the ContractAbiDefinition containing the fetched ABI.
 * @throws An Error if the network is unknown, or fetching the ABI fails.
 */
async function getAbi(net: string, address: string, apiKey: string): Promise<ContractAbiDefinition> {
  const host = getApiHost(net);
  const abiUrl = `http://${host}/api?module=contract&action=getabi&address=${address}&format=raw&apikey=${apiKey}`;
  const response = await fetch(abiUrl);
  const abi = await response.json();
  if (abi.status === '0') {
    throw new Error(`Failed to fetch abi from etherscan: ${abi.result}`);
  }
  return abi;
}

// async function getInitData(address: string, ethHost: string) {
// const host = getHost(net);
// const response: string = await fetch(`https://${host}/address/${address}`).then(r => r.text());
// console.log(response);
// const initCodeMd = response.match(/<div id='verifiedbytecode2'>([0-9a-f]+)</);
// if (!initCodeMd) {
//   return;
// }
// const initCode = '0x' + initCodeMd![1];
// const ctorParamsMd = response.match(
//   /last bytes of the Contract Creation Code above.*?margin-top: 5px;'>([0-9a-f]+)</,
// );
// if (ctorParamsMd) {
//   const ctorParams = ctorParamsMd![1];
//   if (!initCode.endsWith(ctorParams)) {
//     throw new Error('Expected ctor params to be appended to end of init code.');
//   }
//   return initCode.slice(0, -ctorParams.length);
// }
// return initCode;
// }
/**
 * Retrieves the Contract Application Binary Interface (ABI) and bytecode for a given Ethereum contract address
 * from the Etherscan API, using the specified network and API key. The ABI is essential for interacting with
 * smart contracts and decoding transactions in Ethereum. If an Ethereum host is provided, this function will
 * also fetch the contract's bytecode.
 *
 * @param net - The Ethereum network identifier, such as 'mainnet', 'kovan', or 'ropsten'.
 * @param address - The Ethereum contract address to fetch the ABI for.
 * @param apiKey - The Etherscan API key for accessing their services.
 * @param ethHost - The Ethereum host URL, if available, to fetch the bytecode of the contract.
 * @returns A Promise that resolves to an object containing the fetched ABI and optionally the bytecode.
 * @throws An Error if the network is unknown, or fetching the ABI fails.
 */
export async function getFromEtherscan(net: string, address: string, apiKey: string, ethHost: string) {
  const abi = await getAbi(net, address, apiKey);

  if (ethHost) {
    const ethRpc = new EthereumRpc(new JsonRpcProvider(ethHost));
    const initData = bufferToHex(await ethRpc.getCode(EthAddress.fromString(address)));
    return { abi, initData };
  } else {
    console.log(`No ETHEREUM_HOST env var specified, will not include bytecode for ${address}.`);
  }

  return { abi };
}
