/*
  Copyright (c) 2019 xf00f

  This file is part of web3x and is released under the MIT License.
  https://opensource.org/licenses/MIT
*/

import { ContractAbiDefinition } from '../../abi/index.js';

/**
 * Represents a file-based source for contract ABI and initialization data.
 * Provides a convenient way to specify the contract name, ABI file location,
 * and optional initialization data file location when generating contract interfaces.
 */
interface FileSource {
  /**
   * The origin of contract configuration data.
   */
  source: 'files';
  /**
   * The unique identifier for the contract configuration.
   */
  name: string;
  /**
   * Path to the contract's ABI file.
   */
  abiFile: string;
  /**
   * Optional file containing initialization data for the contract.
   */
  initDataFile?: string;
}

/**
 * Represents an Etherscan source configuration for a smart contract.
 * Provides necessary information to fetch the ABI and contract details from the Etherscan API.
 */
interface EtherscanSource {
  /**
   * The source from which the contract definition is obtained.
   */
  source: 'etherscan';
  /**
   * The unique identifier for the contract.
   */
  name: string;
  /**
   * The Ethereum network identifier.
   */
  net: string;
  /**
   * The Ethereum contract address.
   */
  address: string;
}

/**
 * Represents a Truffle-based source configuration for contract information.
 * Provides properties to identify the contract name and associated build file
 * in order to extract ABI and other necessary data for further web3x usage.
 */
interface TruffleSource {
  /**
   * The origin of contract information.
   */
  source: 'truffle';
  /**
   * The unique identifier for the contract.
   */
  name: string;
  /**
   * The path to the build file containing contract information.
   */
  buildFile: string;
}

/**
 * Represents a Foundry build file source for contract configuration.
 * Provides necessary details to locate and use the Foundry generated build files for smart contracts.
 */
interface FoundrySource {
  /**
   * The origin of the contract configuration data.
   */
  source: 'foundry';
  /**
   * The unique identifier for a contract.
   */
  name: string;
  /**
   * The path to the build file containing contract information.
   */
  buildFile: string;
}

/**
 * Represents an inline contract source configuration.
 * Provides a convenient way to directly specify the contract ABI and optional initialization data in the configuration object.
 */
interface InlineSource {
  /**
   * The origin of contract ABI and initialization data.
   */
  source: 'inline';
  /**
   * The name identifier for the contract.
   */
  name: string;
  /**
   * The contract's Application Binary Interface (ABI) definition.
   */
  abi: ContractAbiDefinition;
  /**
   * Initialization data for contract deployment.
   */
  initData?: string;
}

/**
 * Union type representing various contract configuration sources including file, Etherscan, Truffle, Foundry, and inline.
 */
export type ContractConfig = FileSource | EtherscanSource | TruffleSource | InlineSource | FoundrySource;

/**
 * Represents a configuration object for web3x contract generation.
 * Provides options to specify import paths, output paths, and various contract source types for generating contract interfaces.
 */
export interface Config {
  /**
   * The path to import contracts from.
   */
  importPath?: string;
  /**
   * The destination path for generated output files.
   */
  outputPath?: string;
  /**
   * An array of contract configurations for various sources.
   */
  contracts: ContractConfig[];
}
