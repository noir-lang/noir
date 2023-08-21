#!/usr/bin/env node
import { Crs, BarretenbergApiAsync, newBarretenbergApiAsync, RawBuffer } from './index.js';
import createDebug from 'debug';
import { readFileSync, writeFileSync } from 'fs';
import { gunzipSync } from 'zlib';
import { Command } from 'commander';

createDebug.log = console.error.bind(console);
const debug = createDebug('bb.js');

// Maximum we support.
const MAX_CIRCUIT_SIZE = 2 ** 19;

function getBytecode(bytecodePath: string) {
  const encodedCircuit = readFileSync(bytecodePath, 'utf-8');
  const buffer = Buffer.from(encodedCircuit, 'base64');
  const decompressed = gunzipSync(buffer);
  return decompressed;
}

async function getGates(bytecodePath: string, api: BarretenbergApiAsync) {
  const { total } = await computeCircuitSize(bytecodePath, api);
  return total;
}

function getWitness(witnessPath: string) {
  const data = readFileSync(witnessPath);
  const decompressed = gunzipSync(data);
  return decompressed;
}

async function computeCircuitSize(bytecodePath: string, api: BarretenbergApiAsync) {
  debug(`computing circuit size...`);
  const bytecode = getBytecode(bytecodePath);
  const [exact, total, subgroup] = await api.acirGetCircuitSizes(bytecode);
  return { exact, total, subgroup };
}

async function init(bytecodePath: string, crsPath: string) {
  const api = await newBarretenbergApiAsync();

  const circuitSize = await getGates(bytecodePath, api);
  const subgroupSize = Math.pow(2, Math.ceil(Math.log2(circuitSize)));
  if (subgroupSize > MAX_CIRCUIT_SIZE) {
    throw new Error(`Circuit size of ${subgroupSize} exceeds max supported of ${MAX_CIRCUIT_SIZE}`);
  }

  debug(`circuit size: ${circuitSize}`);
  debug(`subgroup size: ${subgroupSize}`);
  debug('loading crs...');
  // Plus 1 needed! (Move +1 into Crs?)
  const crs = await Crs.new(subgroupSize + 1, crsPath);

  // Important to init slab allocator as first thing, to ensure maximum memory efficiency.
  await api.commonInitSlabAllocator(subgroupSize);

  // Load CRS into wasm global CRS state.
  // TODO: Make RawBuffer be default behavior, and have a specific Vector type for when wanting length prefixed.
  await api.srsInitSrs(new RawBuffer(crs.getG1Data()), crs.numPoints, new RawBuffer(crs.getG2Data()));

  const acirComposer = await api.acirNewAcirComposer(subgroupSize);
  return { api, acirComposer, circuitSize: subgroupSize };
}

async function initLite() {
  const api = await newBarretenbergApiAsync(1);

  // Plus 1 needed! (Move +1 into Crs?)
  const crs = await Crs.new(1);

  // Load CRS into wasm global CRS state.
  await api.srsInitSrs(new RawBuffer(crs.getG1Data()), crs.numPoints, new RawBuffer(crs.getG2Data()));

  const acirComposer = await api.acirNewAcirComposer(0);
  return { api, acirComposer };
}

export async function proveAndVerify(bytecodePath: string, witnessPath: string, crsPath: string, isRecursive: boolean) {
  const { api, acirComposer } = await init(bytecodePath, crsPath);
  try {
    debug(`creating proof...`);
    const bytecode = getBytecode(bytecodePath);
    const witness = getWitness(witnessPath);
    const proof = await api.acirCreateProof(acirComposer, bytecode, witness, isRecursive);

    debug(`verifying...`);
    const verified = await api.acirVerifyProof(acirComposer, proof, isRecursive);
    process.stdout.write(`${verified}`);
    return verified;
  } finally {
    await api.destroy();
  }
}

export async function prove(
  bytecodePath: string,
  witnessPath: string,
  crsPath: string,
  isRecursive: boolean,
  outputPath: string,
) {
  const { api, acirComposer } = await init(bytecodePath, crsPath);
  try {
    debug(`creating proof...`);
    const bytecode = getBytecode(bytecodePath);
    const witness = getWitness(witnessPath);
    const proof = await api.acirCreateProof(acirComposer, bytecode, witness, isRecursive);
    debug(`done.`);

    process.stdout.write(proof);
    writeFileSync(outputPath, proof);

    debug(`proof written to: ${outputPath}`);
  } finally {
    await api.destroy();
  }
}

export async function gateCount(bytecodePath: string) {
  const api = await newBarretenbergApiAsync(1);
  try {
    process.stdout.write(`${await getGates(bytecodePath, api)}`);
  } finally {
    await api.destroy();
  }
}

export async function verify(proofPath: string, isRecursive: boolean, vkPath: string) {
  const { api, acirComposer } = await initLite();
  try {
    await api.acirLoadVerificationKey(acirComposer, new RawBuffer(readFileSync(vkPath)));
    const verified = await api.acirVerifyProof(acirComposer, readFileSync(proofPath), isRecursive);

    process.stdout.write(`${verified}`);
    return verified;
  } finally {
    await api.destroy();
  }
}

export async function contract(outputPath: string, vkPath: string) {
  const { api, acirComposer } = await initLite();
  try {
    await api.acirLoadVerificationKey(acirComposer, new RawBuffer(readFileSync(vkPath)));
    const contract = await api.acirGetSolidityVerifier(acirComposer);

    process.stdout.write(contract);
    writeFileSync(outputPath, contract);

    debug(`contract written to: ${outputPath}`);
  } finally {
    await api.destroy();
  }
}

export async function writeVk(bytecodePath: string, crsPath: string, outputPath: string) {
  const { api, acirComposer } = await init(bytecodePath, crsPath);
  try {
    debug('initing proving key...');
    const bytecode = getBytecode(bytecodePath);
    await api.acirInitProvingKey(acirComposer, bytecode);

    debug('initing verification key...');
    const vk = await api.acirGetVerificationKey(acirComposer);

    process.stdout.write(vk);
    writeFileSync(outputPath, vk);

    debug(`vk written to: ${outputPath}`);
  } finally {
    await api.destroy();
  }
}

export async function proofAsFields(proofPath: string, numInnerPublicInputs: number, outputPath: string) {
  const { api, acirComposer } = await initLite();

  try {
    debug('serializing proof byte array into field elements');
    const proofAsFields = await api.acirSerializeProofIntoFields(
      acirComposer,
      readFileSync(proofPath),
      numInnerPublicInputs,
    );
    const jsonProofAsFields = JSON.stringify(proofAsFields.map(f => f.toString()));

    process.stdout.write(jsonProofAsFields);
    writeFileSync(outputPath, jsonProofAsFields);

    debug('done.');
  } finally {
    await api.destroy();
  }
}

export async function vkAsFields(vkPath: string, vkeyOutputPath: string) {
  const { api, acirComposer } = await initLite();

  try {
    debug('serializing vk byte array into field elements');
    await api.acirLoadVerificationKey(acirComposer, new RawBuffer(readFileSync(vkPath)));
    const [vkAsFields, vkHash] = await api.acirSerializeVerificationKeyIntoFields(acirComposer);
    const output = [vkHash, ...vkAsFields].map(f => f.toString());
    const jsonVKAsFields = JSON.stringify(output);

    process.stdout.write(jsonVKAsFields);
    writeFileSync(vkeyOutputPath, jsonVKAsFields);

    debug('done.');
  } finally {
    await api.destroy();
  }
}

const program = new Command();

program.option('-v, --verbose', 'enable verbose logging', false);
program.option('-c, --crs-path <path>', 'set crs path', './crs');

function handleGlobalOptions() {
  if (program.opts().verbose) {
    createDebug.enable('bb.js*');
  }
}

program
  .command('prove_and_verify')
  .description('Generate a proof and verify it. Process exits with success or failure code.')
  .option('-b, --bytecode-path <path>', 'Specify the bytecode path', './target/main.bytecode')
  .option('-w, --witness-path <path>', 'Specify the witness path', './target/witness.tr')
  .option('-r, --recursive', 'prove and verify using recursive prover and verifier', false)
  .action(async ({ bytecodePath, witnessPath, recursive, crsPath }) => {
    handleGlobalOptions();
    const result = await proveAndVerify(bytecodePath, witnessPath, crsPath, recursive);
    process.exit(result ? 0 : 1);
  });

program
  .command('prove')
  .description('Generate a proof and write it to a file.')
  .option('-b, --bytecode-path <path>', 'Specify the bytecode path', './target/main.bytecode')
  .option('-w, --witness-path <path>', 'Specify the witness path', './target/witness.tr')
  .option('-r, --recursive', 'prove using recursive prover', false)
  .option('-o, --output-path <path>', 'Specify the proof output path', './proofs/proof')
  .action(async ({ bytecodePath, witnessPath, recursive, outputPath, crsPath }) => {
    handleGlobalOptions();
    await prove(bytecodePath, witnessPath, crsPath, recursive, outputPath);
  });

program
  .command('gates')
  .description('Print gate count to standard output.')
  .option('-b, --bytecode-path <path>', 'Specify the bytecode path', './target/main.bytecode')
  .action(async ({ bytecodePath: bytecodePath }) => {
    handleGlobalOptions();
    await gateCount(bytecodePath);
  });

program
  .command('verify')
  .description('Verify a proof. Process exists with success or failure code.')
  .requiredOption('-p, --proof-path <path>', 'Specify the path to the proof')
  .option('-r, --recursive', 'prove using recursive prover', false)
  .requiredOption('-k, --vk <path>', 'path to a verification key. avoids recomputation.')
  .action(async ({ proofPath, recursive, vk }) => {
    handleGlobalOptions();
    const result = await verify(proofPath, recursive, vk);
    process.exit(result ? 0 : 1);
  });

program
  .command('contract')
  .description('Output solidity verification key contract.')
  .option('-b, --bytecode-path <path>', 'Specify the bytecode path', './target/main.bytecode')
  .option('-o, --output-path <path>', 'Specify the path to write the contract', '-')
  .requiredOption('-k, --vk <path>', 'path to a verification key. avoids recomputation.')
  .action(async ({ outputPath, vk }) => {
    handleGlobalOptions();
    await contract(outputPath, vk);
  });

program
  .command('write_vk')
  .description('Output verification key.')
  .option('-b, --bytecode-path <path>', 'Specify the bytecode path', './target/main.bytecode')
  .requiredOption('-o, --output-path <path>', 'Specify the path to write the key')
  .action(async ({ bytecodePath, outputPath, crsPath }) => {
    handleGlobalOptions();
    await writeVk(bytecodePath, crsPath, outputPath);
  });

program
  .command('proof_as_fields')
  .description('Return the proof as fields elements')
  .requiredOption('-p, --proof-path <path>', 'Specify the proof path')
  .requiredOption('-n, --num-public-inputs <number>', 'Specify the number of public inputs')
  .requiredOption('-o, --output-path <path>', 'Specify the JSON path to write the proof fields')
  .action(async ({ proofPath, numPublicInputs, outputPath }) => {
    handleGlobalOptions();
    await proofAsFields(proofPath, numPublicInputs, outputPath);
  });

program
  .command('vk_as_fields')
  .description('Return the verification key represented as fields elements. Also return the verification key hash.')
  .requiredOption('-i, --input-path <path>', 'Specifies the vk path (output from write_vk)')
  .requiredOption('-o, --output-path <path>', 'Specify the JSON path to write the verification key fields and key hash')
  .action(async ({ inputPath, outputPath }) => {
    handleGlobalOptions();
    await vkAsFields(inputPath, outputPath);
  });

program.name('bb.js').parse(process.argv);
