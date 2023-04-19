/*
  Copyright (c) 2019 xf00f

  This file is part of web3x and is released under the MIT License.
  https://opensource.org/licenses/MIT
*/

import fs from 'fs';
import { ContractAbiDefinition } from '../../abi/index.js';

/**
 * Extracts the contract ABI (Application Binary Interface) definition and initialization data from a foundry build file.
 * The returned object contains both 'abi' and optional 'initData' properties as parsed from the provided JSON build file.
 * This function is useful for obtaining contract information from compiled sources, like Solidity smart contracts.
 *
 * @param buildFile - The path to the foundry build file containing the ABI and bytecode information.
 * @returns An object containing the contract ABI definition and optional initialization data as properties.
 */
export function getFromFoundry(buildFile: string): {
  /**
   * The contract's Application Binary Interface.
   */
  abi: ContractAbiDefinition;
  /**
   * The bytecode object representing the contract's initial state.
   */
  initData?: string;
} {
  const {
    abi,
    bytecode: { object: initData },
  } = JSON.parse(fs.readFileSync(buildFile).toString());
  return { abi, initData };
}
