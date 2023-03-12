/*
  Copyright (c) 2019 xf00f

  This file is part of web3x and is released under the MIT License.
  https://opensource.org/licenses/MIT
*/

import fs from 'fs';
import { ContractAbiDefinition } from '../../abi/index.js';

export function getFromTruffle(buildFile: string): { abi: ContractAbiDefinition; initData?: string } {
  const { abi, bytecode: initData } = JSON.parse(fs.readFileSync(buildFile).toString());
  return { abi, initData };
}
