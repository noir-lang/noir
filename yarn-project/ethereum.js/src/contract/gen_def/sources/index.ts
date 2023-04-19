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

/**
 * Represents contract build data for a smart contract.
 * Contains the ABI definition and optional initialization data required for deploying and interacting with the contract.
 */
export interface ContractBuildData {
  /**
   * The Application Binary Interface (ABI) for a smart contract.
   */
  abi: ContractAbiDefinition;
  /**
   * Initial deployment data for the contract.
   */
  initData?: string;
}

const { ETHERSCAN_API_KEY = '', ETHEREUM_HOST = '' } = process.env;

/**
 * Loads contract build data (ABI and initialization data) from the provided configuration object.
 * Supports various sources such as Etherscan, files, Truffle, Foundry, and inline configuration.
 * Throws an error if the source is not supported or if there's an issue with loading the data.
 *
 * @param contract - The ContractConfig object containing source and other required details.
 * @returns A Promise that resolves to a ContractBuildData object containing ABI and optional initData.
 */
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
