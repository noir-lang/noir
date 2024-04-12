import { randomBytes } from '@aztec/foundation/crypto';
import { createDebugLogger } from '@aztec/foundation/log';
import { Timer } from '@aztec/foundation/timer';
import { type NoirCompiledCircuit } from '@aztec/types/noir';

import { type WitnessMap } from '@noir-lang/types';
import * as proc from 'child_process';
import fs from 'fs/promises';

import { type SimulationProvider } from './simulation_provider.js';

const logger = createDebugLogger('aztec:acvm-native');

export enum ACVM_RESULT {
  SUCCESS,
  FAILURE,
}

export type ACVMSuccess = {
  status: ACVM_RESULT.SUCCESS;
  duration: number;
  witness: Map<number, string>;
};

export type ACVMFailure = {
  status: ACVM_RESULT.FAILURE;
  reason: string;
};

export type ACVMResult = ACVMSuccess | ACVMFailure;

/**
 * Parses a TOML format witness map string into a Map structure
 * @param outputString - The witness map in TOML format
 * @returns The parsed witness map
 */
function parseIntoWitnessMap(outputString: string) {
  const lines = outputString.split('\n');
  return new Map<number, string>(
    lines
      .filter((line: string) => line.length)
      .map((line: string) => {
        const pair = line.replaceAll(' ', '').split('=');
        return [Number(pair[0]), pair[1].replaceAll('"', '')];
      }),
  );
}

/**
 *
 * @param inputWitness - The circuit's input witness
 * @param bytecode - The circuit bytecode
 * @param workingDirectory - A directory to use for temporary files by the ACVM
 * @param pathToAcvm - The path to the ACVM binary
 * @param outputFilename - If specified, the output will be stored as a file, encoded using Bincode
 * @returns The completed partial witness outputted from the circuit
 */
export async function executeNativeCircuit(
  inputWitness: WitnessMap,
  bytecode: Buffer,
  workingDirectory: string,
  pathToAcvm: string,
  outputFilename?: string,
): Promise<ACVMResult> {
  const bytecodeFilename = 'bytecode';
  const witnessFilename = 'input_witness.toml';

  // convert the witness map to TOML format
  let witnessMap = '';
  inputWitness.forEach((value: string, key: number) => {
    witnessMap = witnessMap.concat(`${key} = '${value}'\n`);
  });

  try {
    // Check that the directory exists
    await fs.access(workingDirectory);
  } catch (error) {
    return { status: ACVM_RESULT.FAILURE, reason: `Working directory ${workingDirectory} does not exist` };
  }

  try {
    // Write the bytecode and input witness to the working directory
    await fs.writeFile(`${workingDirectory}/${bytecodeFilename}`, bytecode);
    await fs.writeFile(`${workingDirectory}/${witnessFilename}`, witnessMap);

    // Execute the ACVM using the given args
    const args = [
      `execute`,
      `--working-directory`,
      `${workingDirectory}`,
      `--bytecode`,
      `${bytecodeFilename}`,
      `--input-witness`,
      `${witnessFilename}`,
      '--print',
      '--output-witness',
      'output-witness',
    ];

    logger.debug(`Calling ACVM with ${args.join(' ')}`);

    const processPromise = new Promise<string>((resolve, reject) => {
      let outputWitness = Buffer.alloc(0);
      let errorBuffer = Buffer.alloc(0);
      const acvm = proc.spawn(pathToAcvm, args);
      acvm.stdout.on('data', data => {
        outputWitness = Buffer.concat([outputWitness, data]);
      });
      acvm.stderr.on('data', data => {
        errorBuffer = Buffer.concat([errorBuffer, data]);
      });
      acvm.on('close', code => {
        if (code === 0) {
          resolve(outputWitness.toString('utf-8'));
        } else {
          logger.error(`From ACVM: ${errorBuffer.toString('utf-8')}`);
          reject(errorBuffer.toString('utf-8'));
        }
      });
    });

    const duration = new Timer();
    const output = await processPromise;
    if (outputFilename) {
      const outputWitnessFileName = `${workingDirectory}/output-witness.gz`;
      await fs.copyFile(outputWitnessFileName, outputFilename);
    }
    const witness = parseIntoWitnessMap(output);
    return { status: ACVM_RESULT.SUCCESS, witness, duration: duration.ms() };
  } catch (error) {
    return { status: ACVM_RESULT.FAILURE, reason: `${error}` };
  }
}

export class NativeACVMSimulator implements SimulationProvider {
  constructor(private workingDirectory: string, private pathToAcvm: string, private witnessFilename?: string) {}
  async simulateCircuit(input: WitnessMap, compiledCircuit: NoirCompiledCircuit): Promise<WitnessMap> {
    // Execute the circuit on those initial witness values

    // Decode the bytecode from base64 since the acvm does not know about base64 encoding
    const decodedBytecode = Buffer.from(compiledCircuit.bytecode, 'base64');

    // Provide a unique working directory so we don't get clashes with parallel executions
    const directory = `${this.workingDirectory}/${randomBytes(8).toString('hex')}`;

    await fs.mkdir(directory, { recursive: true });

    // Execute the circuit
    const result = await executeNativeCircuit(input, decodedBytecode, directory, this.pathToAcvm, this.witnessFilename);

    await fs.rm(directory, { force: true, recursive: true });

    if (result.status == ACVM_RESULT.FAILURE) {
      throw new Error(`Failed to generate witness: ${result.reason}`);
    }

    return result.witness;
  }
}
