/*
  Copyright (c) 2019 xf00f

  This file is part of web3x and is released under the MIT License.
  https://opensource.org/licenses/MIT
*/

import fs from 'fs';
import { ContractAbiDefinition } from '../../abi/index.js';

/**
 * Reads ABI and initData from specified files, and returns an object containing ContractAbiDefinition and initData as properties.
 * This function is used to load necessary data for a smart contract from local JSON files.
 *
 * @param abiFile - The file path to the JSON file containing the ABI (Application Binary Interface) of the smart contract.
 * @param initDataFile - Optional. The file path to the JSON file containing the initialization data for the smart contract.
 * @returns An object with properties 'abi' containing the ContractAbiDefinition and optional 'initData' containing the initialization data.
 */
export function getFromFiles(abiFile: string, initDataFile?: string) {
  const abi: ContractAbiDefinition = JSON.parse(fs.readFileSync(abiFile).toString());

  if (initDataFile) {
    const initData = fs.readFileSync(initDataFile).toString();
    return { abi, initData };
  }

  return { abi };
}
