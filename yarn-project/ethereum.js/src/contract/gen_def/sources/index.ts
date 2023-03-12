/*
  Copyright (c) 2019 xf00f

  This file is part of web3x and is released under the MIT License.
  https://opensource.org/licenses/MIT
*/

import { ContractAbiDefinition } from '../../abi/index.js';
import { ContractConfig } from './config.js';
import { getFromEtherscan } from './source-etherscan.js';
import { getFromFiles } from './source-files.js';
import { getFromFoundry } from './source-foundry.js';
import { getFromTruffle } from './source-truffle.js';

export interface ContractBuildData {
  abi: ContractAbiDefinition;
  initData?: string;
}

const { ETHERSCAN_API_KEY = '', ETHEREUM_HOST = '' } = process.env;

export async function loadDataFromConfig(contract: ContractConfig): Promise<ContractBuildData> {
  switch (contract.source) {
    case 'etherscan':
      return await getFromEtherscan(contract.net, contract.address, ETHERSCAN_API_KEY, ETHEREUM_HOST);
    case 'files':
      return getFromFiles(contract.abiFile, contract.initDataFile);
    case 'truffle':
      return getFromTruffle(contract.buildFile);
    case 'foundry':
      return getFromFoundry(contract.buildFile);
    case 'inline':
      return { abi: contract.abi, initData: contract.initData };
  }
}
