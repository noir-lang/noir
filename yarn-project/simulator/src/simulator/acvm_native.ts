import { randomBytes } from '@aztec/foundation/crypto';
import { type NoirCompiledCircuit } from '@aztec/types/noir';

import { type WitnessMap } from '@noir-lang/types';
import * as proc from 'child_process';
import fs from 'fs/promises';

import { type SimulationProvider } from './simulation_provider.js';

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
 * @param pathToAcvm - The path to the ACVm binary
 * @returns The completed partial witness outputted from the circuit
 */
export async function executeNativeCircuit(
  inputWitness: WitnessMap,
  bytecode: Buffer,
  workingDirectory: string,
  pathToAcvm: string,
) {
  const bytecodeFilename = 'bytecode';
  const witnessFilename = 'input_witness.toml';

  // convert the witness map to TOML format
  let witnessMap = '';
  inputWitness.forEach((value: string, key: number) => {
    witnessMap = witnessMap.concat(`${key} = '${value}'\n`);
  });

  // In case the directory is still around from some time previously, remove it
  await fs.rm(workingDirectory, { recursive: true, force: true });
  // Create the new working directory
  await fs.mkdir(workingDirectory, { recursive: true });
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
    `--print`,
  ];
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
        reject(errorBuffer.toString('utf-8'));
      }
    });
  });

  try {
    const output = await processPromise;
    return parseIntoWitnessMap(output);
  } finally {
    // Clean up the working directory before we leave
    await fs.rm(workingDirectory, { recursive: true, force: true });
  }
}

export class NativeACVMSimulator implements SimulationProvider {
  constructor(private workingDirectory: string, private pathToAcvm: string) {}
  async simulateCircuit(input: WitnessMap, compiledCircuit: NoirCompiledCircuit): Promise<WitnessMap> {
    // Execute the circuit on those initial witness values

    // Decode the bytecode from base64 since the acvm does not know about base64 encoding
    const decodedBytecode = Buffer.from(compiledCircuit.bytecode, 'base64');

    // Provide a unique working directory so we don't get clashes with parallel executions
    const directory = `${this.workingDirectory}/${randomBytes(32).toString('hex')}`;
    // Execute the circuit
    const _witnessMap = await executeNativeCircuit(input, decodedBytecode, directory, this.pathToAcvm);

    return _witnessMap;
  }
}
