import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { type Command } from 'commander';

export function injectCommands(program: Command, log: LogFn, debugLogger: DebugLogger) {
  program
    .command('inspect-contract')
    .description('Shows list of external callable functions for a contract')
    .argument(
      '<contractArtifactFile>',
      `A compiled Noir contract's artifact in JSON format or name of a contract artifact exported by @aztec/noir-contracts.js`,
    )
    .action(async (contractArtifactFile: string) => {
      const { inspectContract } = await import('./inspect_contract.js');
      await inspectContract(contractArtifactFile, debugLogger, log);
    });

  // Helper for users to decode hex strings into structs if needed.
  program
    .command('parse-parameter-struct')
    .description("Helper for parsing an encoded string into a contract's parameter struct.")
    .argument('<encodedString>', 'The encoded hex string')
    .requiredOption(
      '-c, --contract-artifact <fileLocation>',
      "A compiled Aztec.nr contract's ABI in JSON format or name of a contract ABI exported by @aztec/noir-contracts.js",
    )
    .requiredOption('-p, --parameter <parameterName>', 'The name of the struct parameter to decode into')
    .action(async (encodedString, options) => {
      const { parseParameterStruct } = await import('./parse_parameter_struct.js');
      await parseParameterStruct(encodedString, options.contractArtifact, options.parameter, log);
    });

  return program;
}
