import * as fs from 'fs';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const NOIR_CONSTANTS_FILE = '../../../../noir-projects/noir-protocol-circuits/crates/types/src/constants.nr';
const TS_CONSTANTS_FILE = '../constants.gen.ts';
const CPP_AZTEC_CONSTANTS_FILE = '../../../../barretenberg/cpp/src/barretenberg/vm/avm_trace/aztec_constants.hpp';
const PIL_AZTEC_CONSTANTS_FILE = '../../../../barretenberg/cpp/pil/avm/constants_gen.pil';
const SOLIDITY_CONSTANTS_FILE = '../../../../l1-contracts/src/core/libraries/ConstantsGen.sol';

// Whitelist of constants that will be copied to aztec_constants.hpp.
// We don't copy everything as just a handful are needed, and updating them breaks the cache and triggers expensive bb builds.
const CPP_CONSTANTS = [
  'TOTAL_FEES_LENGTH',
  'GAS_FEES_LENGTH',
  'GAS_LENGTH',
  'CONTENT_COMMITMENT_LENGTH',
  'GLOBAL_VARIABLES_LENGTH',
  'APPEND_ONLY_TREE_SNAPSHOT_LENGTH',
  'PARTIAL_STATE_REFERENCE_LENGTH',
  'STATE_REFERENCE_LENGTH',
  'HEADER_LENGTH',
  'CALL_CONTEXT_LENGTH',
  'PUBLIC_CONTEXT_INPUTS_LENGTH',
  'PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH',
  'READ_REQUEST_LENGTH',
  'MAX_NOTE_HASH_READ_REQUESTS_PER_CALL',
  'MAX_NULLIFIER_READ_REQUESTS_PER_CALL',
  'MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL',
  'MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL',
  'CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH',
  'MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL',
  'CONTRACT_STORAGE_READ_LENGTH',
  'PUBLIC_CALL_REQUEST_LENGTH',
  'MAX_PUBLIC_DATA_READS_PER_CALL',
  'MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL',
  'NOTE_HASH_LENGTH',
  'MAX_NOTE_HASHES_PER_CALL',
  'NULLIFIER_LENGTH',
  'MAX_NULLIFIERS_PER_CALL',
  'L2_TO_L1_MESSAGE_LENGTH',
  'MAX_L2_TO_L1_MSGS_PER_CALL',
  'LOG_HASH_LENGTH',
  'MAX_UNENCRYPTED_LOGS_PER_CALL',
  'HEADER_LENGTH',
  'GLOBAL_VARIABLES_LENGTH',
  'AZTEC_ADDRESS_LENGTH',
  'START_NOTE_HASH_EXISTS_WRITE_OFFSET',
  'START_NULLIFIER_EXISTS_OFFSET',
  'START_NULLIFIER_NON_EXISTS_OFFSET',
  'START_L1_TO_L2_MSG_EXISTS_WRITE_OFFSET',
  'START_SSTORE_WRITE_OFFSET',
  'START_SLOAD_WRITE_OFFSET',
  'START_EMIT_NOTE_HASH_WRITE_OFFSET',
  'START_EMIT_NULLIFIER_WRITE_OFFSET',
  'START_EMIT_L2_TO_L1_MSG_WRITE_OFFSET',
  'START_EMIT_UNENCRYPTED_LOG_WRITE_OFFSET',
  'SENDER_SELECTOR',
  'ADDRESS_SELECTOR',
  'STORAGE_ADDRESS_SELECTOR',
  'FUNCTION_SELECTOR_SELECTOR',
  'START_GLOBAL_VARIABLES',
  'CHAIN_ID_SELECTOR',
  'VERSION_SELECTOR',
  'BLOCK_NUMBER_SELECTOR',
  'TIMESTAMP_SELECTOR',
  'COINBASE_SELECTOR',
  'FEE_PER_DA_GAS_SELECTOR',
  'FEE_PER_L2_GAS_SELECTOR',
  'END_GLOBAL_VARIABLES',
  'START_SIDE_EFFECT_COUNTER',
  'TRANSACTION_FEE_SELECTOR',
];

const PIL_CONSTANTS = [
  'MAX_NOTE_HASH_READ_REQUESTS_PER_CALL',
  'MAX_NULLIFIER_READ_REQUESTS_PER_CALL',
  'MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL',
  'MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL',
  'MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL',
  'MAX_PUBLIC_DATA_READS_PER_CALL',
  'MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL',
  'MAX_NOTE_HASHES_PER_CALL',
  'MAX_NULLIFIERS_PER_CALL',
  'MAX_L2_TO_L1_MSGS_PER_CALL',
  'MAX_UNENCRYPTED_LOGS_PER_CALL',
  'START_NOTE_HASH_EXISTS_WRITE_OFFSET',
  'START_NULLIFIER_EXISTS_OFFSET',
  'START_NULLIFIER_NON_EXISTS_OFFSET',
  'START_L1_TO_L2_MSG_EXISTS_WRITE_OFFSET',
  'START_SSTORE_WRITE_OFFSET',
  'START_SLOAD_WRITE_OFFSET',
  'START_EMIT_NOTE_HASH_WRITE_OFFSET',
  'START_EMIT_NULLIFIER_WRITE_OFFSET',
  'START_EMIT_L2_TO_L1_MSG_WRITE_OFFSET',
  'START_EMIT_UNENCRYPTED_LOG_WRITE_OFFSET',
  'SENDER_SELECTOR',
  'ADDRESS_SELECTOR',
  'STORAGE_ADDRESS_SELECTOR',
  'FUNCTION_SELECTOR_SELECTOR',
  'START_GLOBAL_VARIABLES',
  'CHAIN_ID_SELECTOR',
  'VERSION_SELECTOR',
  'BLOCK_NUMBER_SELECTOR',
  'TIMESTAMP_SELECTOR',
  'COINBASE_SELECTOR',
  'FEE_PER_DA_GAS_SELECTOR',
  'FEE_PER_L2_GAS_SELECTOR',
  'END_GLOBAL_VARIABLES',
  'START_SIDE_EFFECT_COUNTER',
  'TRANSACTION_FEE_SELECTOR',
];

/**
 * Parsed content.
 */
interface ParsedContent {
  /**
   * Constants of the form "CONSTANT_NAME: number_as_string".
   */
  constants: { [key: string]: string };
  /**
   * GeneratorIndexEnum.
   */
  generatorIndexEnum: { [key: string]: number };
}

/**
 * Processes a collection of constants and generates code to export them as TypeScript constants.
 *
 * @param constants - An object containing key-value pairs representing constants.
 * @returns A string containing code that exports the constants as TypeScript constants.
 */
function processConstantsTS(constants: { [key: string]: string }): string {
  const code: string[] = [];
  Object.entries(constants).forEach(([key, value]) => {
    code.push(`export const ${key} = ${+value > Number.MAX_SAFE_INTEGER ? value + 'n' : value};`);
  });
  return code.join('\n');
}

/**
 * Processes a collection of constants and generates code to export them as cpp constants.
 * Required to ensure consistency between the constants used in pil and used in the vm witness generator.
 *
 * @param constants - An object containing key-value pairs representing constants.
 * @returns A string containing code that exports the constants as cpp constants.
 */
function processConstantsCpp(constants: { [key: string]: string }): string {
  const code: string[] = [];
  Object.entries(constants).forEach(([key, value]) => {
    if (CPP_CONSTANTS.includes(key)) {
      code.push(`#define ${key} ${value}`);
    }
  });
  return code.join('\n');
}

/**
 * Processes a collection of constants and generates code to export them as PIL constants.
 * Required to ensure consistency between the constants used in pil and used in the vm witness generator.
 *
 * @param constants - An object containing key-value pairs representing constants.
 * @returns A string containing code that exports the constants as cpp constants.
 */
function processConstantsPil(constants: { [key: string]: string }): string {
  const code: string[] = [];
  Object.entries(constants).forEach(([key, value]) => {
    if (PIL_CONSTANTS.includes(key)) {
      code.push(`    pol ${key} = ${value};`);
    }
  });
  return code.join('\n');
}
/**
 * Processes an enum and generates code to export it as a TypeScript enum.
 *
 * @param enumName - The name of the enum.
 * @param enumValues - An object containing key-value pairs representing enum values.
 * @returns A string containing code that exports the enum as a TypeScript enum.
 */
function processEnumTS(enumName: string, enumValues: { [key: string]: number }): string {
  const code: string[] = [];

  code.push(`export enum ${enumName} {`);

  Object.entries(enumValues).forEach(([key, value]) => {
    code.push(`  ${key} = ${value},`);
  });

  code.push('}');

  return code.join('\n');
}

/**
 * Processes a collection of constants and generates code to export them as Solidity constants.
 *
 * @param constants - An object containing key-value pairs representing constants.
 * @param prefix - A prefix to add to the constant names.
 * @returns A string containing code that exports the constants as Noir constants.
 */
function processConstantsSolidity(constants: { [key: string]: string }, prefix = ''): string {
  const code: string[] = [];
  Object.entries(constants).forEach(([key, value]) => {
    code.push(`  uint256 internal constant ${prefix}${key} = ${value};`);
  });
  return code.join('\n');
}

/**
 * Generate the constants file in Typescript.
 */
function generateTypescriptConstants({ constants, generatorIndexEnum }: ParsedContent, targetPath: string) {
  const result = [
    '/* eslint-disable */\n// GENERATED FILE - DO NOT EDIT, RUN yarn remake-constants',
    processConstantsTS(constants),
    processEnumTS('GeneratorIndex', generatorIndexEnum),
  ].join('\n');

  fs.writeFileSync(targetPath, result);
}

/**
 * Generate the constants file in C++.
 */
function generateCppConstants({ constants }: ParsedContent, targetPath: string) {
  const resultCpp: string = `// GENERATED FILE - DO NOT EDIT, RUN yarn remake-constants in circuits.js
#pragma once

${processConstantsCpp(constants)}
\n`;

  fs.writeFileSync(targetPath, resultCpp);
}

/**
 * Generate the constants file in PIL.
 */
function generatePilConstants({ constants }: ParsedContent, targetPath: string) {
  const resultPil: string = `// GENERATED FILE - DO NOT EDIT, RUN yarn remake-constants in circuits.js
namespace constants(256);
${processConstantsPil(constants)}
\n`;

  fs.writeFileSync(targetPath, resultPil);
}

/**
 * Generate the constants file in Solidity.
 */
function generateSolidityConstants({ constants }: ParsedContent, targetPath: string) {
  const resultSolidity: string = `// GENERATED FILE - DO NOT EDIT, RUN yarn remake-constants in circuits.js
// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

/**
 * @title Constants Library
 * @author Aztec Labs
 * @notice Library that contains constants used throughout the Aztec protocol
 */
library Constants {
  // Prime field modulus
  uint256 internal constant P =
    21888242871839275222246405745257275088548364400416034343698204186575808495617;

${processConstantsSolidity(constants)}
}\n`;

  fs.writeFileSync(targetPath, resultSolidity);
}

/**
 * Parse the content of the constants file in Noir.
 */
function parseNoirFile(fileContent: string): ParsedContent {
  const constantsExpressions: [string, string][] = [];
  const generatorIndexEnum: { [key: string]: number } = {};

  fileContent.split('\n').forEach(l => {
    const line = l.trim();
    if (!line || line.match(/^\/\/|^\s*\/?\*/)) {
      return;
    }

    const [, name, _type, value] = line.match(/global\s+(\w+)(\s*:\s*\w+)?\s*=\s*(.+?);/) || [];

    if (!name || !value) {
      // eslint-disable-next-line no-console
      console.warn(`Unknown content: ${line}`);
      return;
    }

    const [, indexName] = name.match(/GENERATOR_INDEX__(\w+)/) || [];
    if (indexName) {
      generatorIndexEnum[indexName] = +value;
    } else {
      constantsExpressions.push([name, value]);
    }
  });

  const constants = evaluateExpressions(constantsExpressions);

  return { constants, generatorIndexEnum };
}

/**
 * Converts constants defined as expressions to constants with actual values.
 * @param expressions Ordered list of expressions of the type: "CONSTANT_NAME: expression".
 *   where the expression is a string that can be evaluated to a number.
 *   For example: "CONSTANT_NAME: 2 + 2" or "CONSTANT_NAME: CONSTANT_A * CONSTANT_B".
 * @returns Parsed expressions of the form: "CONSTANT_NAME: number_as_string".
 */
function evaluateExpressions(expressions: [string, string][]): { [key: string]: string } {
  const constants: { [key: string]: string } = {};

  // Create JS expressions. It is not as easy as just evaluating the expression!
  // We basically need to convert everything to BigInts, otherwise things don't fit.
  // However, (1) the bigints need to be initialized from strings; (2) everything needs to
  // be a bigint, even the actual constant values!
  const prelude = expressions
    .map(([name, rhs]) => {
      const guardedRhs = rhs
        // We make some space around the parentheses, so that constant numbers are still split.
        .replace(/\(/g, '( ')
        .replace(/\)/g, ' )')
        // We split the expression into terms...
        .split(' ')
        // ...and then we convert each term to a BigInt if it is a number.
        .map(term => (isNaN(+term) ? term : `BigInt('${term}')`))
        // We join the terms back together.
        .join(' ');
      return `var ${name} = ${guardedRhs};`;
    })
    .join('\n');

  // Extract each value from the expressions. Observe that this will still be a string,
  // so that we can then choose to express it as BigInt or Number depending on the size.
  for (const [name, _] of expressions) {
    constants[name] = eval(prelude + `; BigInt(${name}).toString()`);
  }

  return constants;
}

/**
 * Convert the Noir constants to TypeScript and Solidity.
 */
function main(): void {
  const __dirname = dirname(fileURLToPath(import.meta.url));

  const noirConstantsFile = join(__dirname, NOIR_CONSTANTS_FILE);
  const noirConstants = fs.readFileSync(noirConstantsFile, 'utf-8');
  const parsedContent = parseNoirFile(noirConstants);

  // Typescript
  const tsTargetPath = join(__dirname, TS_CONSTANTS_FILE);
  generateTypescriptConstants(parsedContent, tsTargetPath);

  // Cpp
  const cppTargetPath = join(__dirname, CPP_AZTEC_CONSTANTS_FILE);
  generateCppConstants(parsedContent, cppTargetPath);

  // PIL
  const pilTargetPath = join(__dirname, PIL_AZTEC_CONSTANTS_FILE);
  generatePilConstants(parsedContent, pilTargetPath);

  // Solidity
  const solidityTargetPath = join(__dirname, SOLIDITY_CONSTANTS_FILE);
  fs.mkdirSync(dirname(solidityTargetPath), { recursive: true });
  generateSolidityConstants(parsedContent, solidityTargetPath);
}

main();
