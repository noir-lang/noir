/*
  Copyright (c) 2019 xf00f

  This file is part of web3x and is released under the MIT License.
  https://opensource.org/licenses/MIT
*/

import { ContractAbiDefinition } from '../../abi/index.js';

interface FileSource {
  source: 'files';
  name: string;
  abiFile: string;
  initDataFile?: string;
}

interface EtherscanSource {
  source: 'etherscan';
  name: string;
  net: string;
  address: string;
}

interface TruffleSource {
  source: 'truffle';
  name: string;
  buildFile: string;
}

interface FoundrySource {
  source: 'foundry';
  name: string;
  buildFile: string;
}

interface InlineSource {
  source: 'inline';
  name: string;
  abi: ContractAbiDefinition;
  initData?: string;
}

export type ContractConfig = FileSource | EtherscanSource | TruffleSource | InlineSource | FoundrySource;

export interface Config {
  importPath?: string;
  outputPath?: string;
  contracts: ContractConfig[];
}
