/*
  Copyright (c) 2019 xf00f

  This file is part of web3x and is released under the MIT License.
  https://opensource.org/licenses/MIT
*/

import fs from 'fs';
import { ContractAbiDefinition } from '../../abi/index.js';

/**
 * Extracts Contract ABI (Application Binary Interface) and initialization data from a Truffle build file.
 * The function reads the specified JSON build file and returns an object containing the ABI and initData.
 * This can be utilized for deploying and interacting with smart contracts through web3x library.
 *
 * @param buildFile - The path to the Truffle build file in JSON format.
 * @returns An object containing the contract ABI and optional initData (bytecode).
 */
export function getFromTruffle(buildFile: string): {
  /**
   * The JSON representation of a smart contract's interface.
   */
  abi: ContractAbiDefinition;
  /**
   * The initialization bytecode for contract deployment.
   */
  initData?: string;
} {
  const { abi, bytecode: initData } = JSON.parse(fs.readFileSync(buildFile).toString());
  return { abi, initData };
}
