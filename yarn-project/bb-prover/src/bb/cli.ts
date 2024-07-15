import { type LogFn } from '@aztec/foundation/log';
import { type ProtocolArtifact, ProtocolCircuitArtifacts } from '@aztec/noir-protocol-circuits-types';

import { Command } from 'commander';
import * as fs from 'fs/promises';

import { generateContractForCircuit, generateKeyForNoirCircuit } from './execute.js';

const { BB_WORKING_DIRECTORY, BB_BINARY_PATH } = process.env;

/**
 * Returns commander program that defines the CLI.
 * @param log - Console logger.
 * @returns The CLI.
 */
export function getProgram(log: LogFn): Command {
  const program = new Command('bb-cli');

  program.description('CLI for interacting with Barretenberg.');

  program
    .command('protocol-circuits')
    .description('Lists the available protocol circuit artifacts')
    .action(() => {
      log(Object.keys(ProtocolCircuitArtifacts).reduce((prev: string, x: string) => prev.concat(`\n${x}`)));
    });

  program
    .command('write-pk')
    .description('Generates the proving key for the specified circuit')
    .requiredOption(
      '-w, --working-directory <string>',
      'A directory to use for storing input/output files',
      BB_WORKING_DIRECTORY,
    )
    .requiredOption('-b, --bb-path <string>', 'The path to the BB binary', BB_BINARY_PATH)
    .requiredOption('-c, --circuit <string>', 'The name of a protocol circuit')
    .action(async options => {
      const compiledCircuit = ProtocolCircuitArtifacts[options.circuit as ProtocolArtifact];
      if (!compiledCircuit) {
        log(`Failed to find circuit ${options.circuit}`);
        return;
      }
      try {
        await fs.access(options.workingDirectory, fs.constants.W_OK);
      } catch (error) {
        log(`Working directory does not exist`);
        return;
      }
      await generateKeyForNoirCircuit(
        options.bbPath,
        options.workingDirectory,
        options.circuit,
        compiledCircuit,
        'pk',
        log,
      );
    });

  program
    .command('write-vk')
    .description('Generates the verification key for the specified circuit')
    .requiredOption(
      '-w, --working-directory <string>',
      'A directory to use for storing input/output files',
      BB_WORKING_DIRECTORY,
    )
    .requiredOption('-b, --bb-path <string>', 'The path to the BB binary', BB_BINARY_PATH)
    .requiredOption('-c, --circuit <string>', 'The name of a protocol circuit')
    .action(async options => {
      const compiledCircuit = ProtocolCircuitArtifacts[options.circuit as ProtocolArtifact];
      if (!compiledCircuit) {
        log(`Failed to find circuit ${options.circuit}`);
        return;
      }
      try {
        await fs.access(options.workingDirectory, fs.constants.W_OK);
      } catch (error) {
        log(`Working directory does not exist`);
        return;
      }
      await generateKeyForNoirCircuit(
        options.bbPath,
        options.workingDirectory,
        options.circuit,
        compiledCircuit,
        'vk',
        log,
      );
    });

  program
    .command('write-contract')
    .description('Generates the verification contract for the specified circuit')
    .requiredOption(
      '-w, --working-directory <string>',
      'A directory to use for storing input/output files',
      BB_WORKING_DIRECTORY,
    )
    .requiredOption('-b, --bb-path <string>', 'The path to the BB binary', BB_BINARY_PATH)
    .requiredOption('-c, --circuit <string>', 'The name of a protocol circuit')
    .requiredOption('-n --contract-name <string>', 'The name of the contract to generate', 'contract.sol')
    .action(async options => {
      const compiledCircuit = ProtocolCircuitArtifacts[options.circuit as ProtocolArtifact];
      if (!compiledCircuit) {
        log(`Failed to find circuit ${options.circuit}`);
        return;
      }
      try {
        await fs.access(options.workingDirectory, fs.constants.W_OK);
      } catch (error) {
        log(`Working directory does not exist`);
        return;
      }

      await generateContractForCircuit(
        options.bbPath,
        options.workingDirectory,
        options.circuit,
        compiledCircuit,
        options.contractName,
        log,
      );
    });

  return program;
}
