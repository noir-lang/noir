/*
  Copyright (c) 2019 xf00f

  This file is part of web3x and is released under the MIT License.
  https://opensource.org/licenses/MIT
*/

import { EthAddress } from '../../../eth_address/index.js';
import { EthereumRpc } from '../../../eth_rpc/index.js';
import { bufferToHex } from '../../../hex_string/index.js';
import { JsonRpcProvider } from '../../../provider/json_rpc_provider.js';
import { ContractAbiDefinition } from '../../abi/index.js';

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
