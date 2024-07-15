#!/usr/bin/env node
import { Crs, Barretenberg, RawBuffer } from './index.js';
import createDebug from 'debug';
import { readFileSync, writeFileSync } from 'fs';
import { gunzipSync } from 'zlib';
import { Command } from 'commander';
import { Timer, writeBenchmark } from './benchmark/index.js';
import path from 'path';
createDebug.log = console.error.bind(console);
const debug = createDebug('bb.js');

// Maximum we support in node and the browser is 2^19.
// This is because both node and browser use barretenberg.wasm.
//
// This is not a restriction in the bb binary and one should be
// aware of this discrepancy, when creating proofs in bb versus
// creating the same proofs in the node CLI.
const MAX_CIRCUIT_SIZE = 2 ** 19;
const threads = +process.env.HARDWARE_CONCURRENCY! || undefined;

function getBytecode(bytecodePath: string) {
  const extension = bytecodePath.substring(bytecodePath.lastIndexOf('.') + 1);

  if (extension == 'json') {
    const encodedCircuit = JSON.parse(readFileSync(bytecodePath, 'utf8'));
    const decompressed = gunzipSync(Buffer.from(encodedCircuit.bytecode, 'base64'));
    return decompressed;
  }

  const encodedCircuit = readFileSync(bytecodePath);
  const decompressed = gunzipSync(encodedCircuit);
  return decompressed;
}

async function getGates(bytecodePath: string, honkRecursion: boolean, api: Barretenberg) {
  const { total } = await computeCircuitSize(bytecodePath, honkRecursion, api);
  return total;
}

function getWitness(witnessPath: string) {
  const data = readFileSync(witnessPath);
  const decompressed = gunzipSync(data);
  return decompressed;
}

async function computeCircuitSize(bytecodePath: string, honkRecursion: boolean, api: Barretenberg) {
  debug(`computing circuit size...`);
  const bytecode = getBytecode(bytecodePath);
  const [exact, total, subgroup] = await api.acirGetCircuitSizes(bytecode, honkRecursion);
  return { exact, total, subgroup };
}

async function init(bytecodePath: string, crsPath: string, subgroupSizeOverride = -1, honkRecursion = false) {
  const api = await Barretenberg.new({ threads });

  const circuitSize = await getGates(bytecodePath, honkRecursion, api);
  // TODO(https://github.com/AztecProtocol/barretenberg/issues/811): remove subgroupSizeOverride hack for goblin
  const subgroupSize = Math.max(subgroupSizeOverride, Math.pow(2, Math.ceil(Math.log2(circuitSize))));
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
  return { api, acirComposer, circuitSize, subgroupSize };
}

async function initLite() {
  const api = await Barretenberg.new({ threads: 1 });

  // Plus 1 needed! (Move +1 into Crs?)
  const crs = await Crs.new(1);

  // Load CRS into wasm global CRS state.
  await api.srsInitSrs(new RawBuffer(crs.getG1Data()), crs.numPoints, new RawBuffer(crs.getG2Data()));

  const acirComposer = await api.acirNewAcirComposer(0);
  return { api, acirComposer };
}

export async function proveAndVerify(bytecodePath: string, witnessPath: string, crsPath: string) {
  /* eslint-disable camelcase */
  const acir_test = path.basename(process.cwd());

  const { api, acirComposer, circuitSize, subgroupSize } = await init(bytecodePath, crsPath);
  try {
    debug(`creating proof...`);
    const bytecode = getBytecode(bytecodePath);
    const witness = getWitness(witnessPath);

    const pkTimer = new Timer();
    await api.acirInitProvingKey(acirComposer, bytecode);
    writeBenchmark('pk_construction_time', pkTimer.ms(), { acir_test, threads });
    writeBenchmark('gate_count', circuitSize, { acir_test, threads });
    writeBenchmark('subgroup_size', subgroupSize, { acir_test, threads });

    const proofTimer = new Timer();
    const proof = await api.acirCreateProof(acirComposer, bytecode, witness);
    writeBenchmark('proof_construction_time', proofTimer.ms(), { acir_test, threads });

    debug(`verifying...`);
    const verified = await api.acirVerifyProof(acirComposer, proof);
    debug(`verified: ${verified}`);
    return verified;
  } finally {
    await api.destroy();
  }
  /* eslint-enable camelcase */
}

export async function proveAndVerifyUltraHonk(bytecodePath: string, witnessPath: string, crsPath: string) {
  /* eslint-disable camelcase */
  const { api } = await init(bytecodePath, crsPath, -1, true);
  try {
    const bytecode = getBytecode(bytecodePath);
    const witness = getWitness(witnessPath);

    const verified = await api.acirProveAndVerifyUltraHonk(bytecode, witness);
    return verified;
  } finally {
    await api.destroy();
  }
  /* eslint-enable camelcase */
}

export async function proveAndVerifyMegaHonk(bytecodePath: string, witnessPath: string, crsPath: string) {
  /* eslint-disable camelcase */
  const { api } = await init(bytecodePath, crsPath);
  try {
    const bytecode = getBytecode(bytecodePath);
    const witness = getWitness(witnessPath);

    const verified = await api.acirProveAndVerifyMegaHonk(bytecode, witness);
    return verified;
  } finally {
    await api.destroy();
  }
  /* eslint-enable camelcase */
}

export async function foldAndVerifyProgram(bytecodePath: string, witnessPath: string, crsPath: string) {
  /* eslint-disable camelcase */
  const { api } = await init(bytecodePath, crsPath);
  try {
    const bytecode = getBytecode(bytecodePath);
    const witness = getWitness(witnessPath);

    const verified = await api.acirFoldAndVerifyProgramStack(bytecode, witness);
    return verified;
  } finally {
    await api.destroy();
  }
  /* eslint-enable camelcase */
}

export async function prove(bytecodePath: string, witnessPath: string, crsPath: string, outputPath: string) {
  const { api, acirComposer } = await init(bytecodePath, crsPath);
  try {
    debug(`creating proof...`);
    const bytecode = getBytecode(bytecodePath);
    const witness = getWitness(witnessPath);
    const proof = await api.acirCreateProof(acirComposer, bytecode, witness);
    debug(`done.`);

    if (outputPath === '-') {
      process.stdout.write(proof);
      debug(`proof written to stdout`);
    } else {
      writeFileSync(outputPath, proof);
      debug(`proof written to: ${outputPath}`);
    }
  } finally {
    await api.destroy();
  }
}

export async function gateCount(bytecodePath: string, honkRecursion: boolean) {
  const api = await Barretenberg.new({ threads: 1 });
  try {
    const numberOfGates = await getGates(bytecodePath, honkRecursion, api);

    // Create an 8-byte buffer and write the number into it.
    // Writing number directly to stdout will result in a variable sized
    // input depending on the size.
    const buffer = Buffer.alloc(8);
    buffer.writeBigInt64LE(BigInt(numberOfGates));

    process.stdout.write(buffer);
  } finally {
    await api.destroy();
  }
}

export async function verify(proofPath: string, vkPath: string) {
  const { api, acirComposer } = await initLite();
  try {
    await api.acirLoadVerificationKey(acirComposer, new RawBuffer(readFileSync(vkPath)));
    const verified = await api.acirVerifyProof(acirComposer, readFileSync(proofPath));
    debug(`verified: ${verified}`);
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

    if (outputPath === '-') {
      process.stdout.write(contract);
      debug(`contract written to stdout`);
    } else {
      writeFileSync(outputPath, contract);
      debug(`contract written to: ${outputPath}`);
    }
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

    if (outputPath === '-') {
      process.stdout.write(vk);
      debug(`vk written to stdout`);
    } else {
      writeFileSync(outputPath, vk);
      debug(`vk written to: ${outputPath}`);
    }
  } finally {
    await api.destroy();
  }
}

export async function writePk(bytecodePath: string, crsPath: string, outputPath: string) {
  const { api, acirComposer } = await init(bytecodePath, crsPath);
  try {
    debug('initing proving key...');
    const bytecode = getBytecode(bytecodePath);
    const pk = await api.acirGetProvingKey(acirComposer, bytecode);

    if (outputPath === '-') {
      process.stdout.write(pk);
      debug(`pk written to stdout`);
    } else {
      writeFileSync(outputPath, pk);
      debug(`pk written to: ${outputPath}`);
    }
  } finally {
    await api.destroy();
  }
}

export async function proofAsFields(proofPath: string, vkPath: string, outputPath: string) {
  const { api, acirComposer } = await initLite();

  try {
    debug('serializing proof byte array into field elements');
    const numPublicInputs = readFileSync(vkPath).readUint32BE(8);
    const proofAsFields = await api.acirSerializeProofIntoFields(
      acirComposer,
      readFileSync(proofPath),
      numPublicInputs,
    );
    const jsonProofAsFields = JSON.stringify(proofAsFields.map(f => f.toString()));

    if (outputPath === '-') {
      process.stdout.write(jsonProofAsFields);
      debug(`proofAsFields written to stdout`);
    } else {
      writeFileSync(outputPath, jsonProofAsFields);
      debug(`proofAsFields written to: ${outputPath}`);
    }

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

    if (vkeyOutputPath === '-') {
      process.stdout.write(jsonVKAsFields);
      debug(`vkAsFields written to stdout`);
    } else {
      writeFileSync(vkeyOutputPath, jsonVKAsFields);
      debug(`vkAsFields written to: ${vkeyOutputPath}`);
    }

    debug('done.');
  } finally {
    await api.destroy();
  }
}

export async function proveUltraHonk(bytecodePath: string, witnessPath: string, crsPath: string, outputPath: string) {
  const { api } = await init(bytecodePath, crsPath, -1, true);
  try {
    debug(`creating proof...`);
    const bytecode = getBytecode(bytecodePath);
    const witness = getWitness(witnessPath);
    const proof = await api.acirProveUltraHonk(bytecode, witness);
    debug(`done.`);

    if (outputPath === '-') {
      process.stdout.write(proof);
      debug(`proof written to stdout`);
    } else {
      writeFileSync(outputPath, proof);
      debug(`proof written to: ${outputPath}`);
    }
  } finally {
    await api.destroy();
  }
}

export async function writeVkUltraHonk(bytecodePath: string, crsPath: string, outputPath: string) {
  const { api } = await init(bytecodePath, crsPath, -1, true);
  try {
    const bytecode = getBytecode(bytecodePath);
    debug('initing verification key...');
    const vk = await api.acirWriteVkUltraHonk(bytecode);

    if (outputPath === '-') {
      process.stdout.write(vk);
      debug(`vk written to stdout`);
    } else {
      writeFileSync(outputPath, vk);
      debug(`vk written to: ${outputPath}`);
    }
  } finally {
    await api.destroy();
  }
}

export async function verifyUltraHonk(proofPath: string, vkPath: string) {
  const { api } = await initLite();
  try {
    const verified = await api.acirVerifyUltraHonk(readFileSync(proofPath), new RawBuffer(readFileSync(vkPath)));
    debug(`verified: ${verified}`);
    return verified;
  } finally {
    await api.destroy();
  }
}

export async function proofAsFieldsUltraHonk(proofPath: string, outputPath: string) {
  const { api } = await initLite();
  try {
    debug('outputting proof as vector of fields');
    const proofAsFields = await api.acirProofAsFieldsUltraHonk(readFileSync(proofPath));
    const jsonProofAsFields = JSON.stringify(proofAsFields.map(f => f.toString()));

    if (outputPath === '-') {
      process.stdout.write(jsonProofAsFields);
      debug(`proofAsFieldsUltraHonk written to stdout`);
    } else {
      writeFileSync(outputPath, jsonProofAsFields);
      debug(`proofAsFieldsUltraHonk written to: ${outputPath}`);
    }

    debug('done.');
  } finally {
    await api.destroy();
  }
}

export async function vkAsFieldsUltraHonk(vkPath: string, vkeyOutputPath: string) {
  const { api } = await initLite();

  try {
    debug('serializing vk byte array into field elements');
    const vkAsFields = await api.acirVkAsFieldsUltraHonk(new RawBuffer(readFileSync(vkPath)));
    const jsonVKAsFields = JSON.stringify(vkAsFields.map(f => f.toString()));

    if (vkeyOutputPath === '-') {
      process.stdout.write(jsonVKAsFields);
      debug(`vkAsFieldsUltraHonk written to stdout`);
    } else {
      writeFileSync(vkeyOutputPath, jsonVKAsFields);
      debug(`vkAsFieldsUltraHonk written to: ${vkeyOutputPath}`);
    }

    debug('done.');
  } finally {
    await api.destroy();
  }
}

const program = new Command('bb');

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
  .option('-b, --bytecode-path <path>', 'Specify the bytecode path', './target/program.json')
  .option('-w, --witness-path <path>', 'Specify the witness path', './target/witness.gz')
  .action(async ({ bytecodePath, witnessPath, crsPath }) => {
    handleGlobalOptions();
    const result = await proveAndVerify(bytecodePath, witnessPath, crsPath);
    process.exit(result ? 0 : 1);
  });

program
  .command('prove_and_verify_ultra_honk')
  .description('Generate an UltraHonk proof and verify it. Process exits with success or failure code.')
  .option('-b, --bytecode-path <path>', 'Specify the bytecode path', './target/program.json')
  .option('-w, --witness-path <path>', 'Specify the witness path', './target/witness.gz')
  .action(async ({ bytecodePath, witnessPath, crsPath }) => {
    handleGlobalOptions();
    const result = await proveAndVerifyUltraHonk(bytecodePath, witnessPath, crsPath);
    process.exit(result ? 0 : 1);
  });

program
  .command('prove_and_verify_mega_honk')
  .description('Generate a MegaHonk proof and verify it. Process exits with success or failure code.')
  .option('-b, --bytecode-path <path>', 'Specify the bytecode path', './target/program.json')
  .option('-w, --witness-path <path>', 'Specify the witness path', './target/witness.gz')
  .action(async ({ bytecodePath, witnessPath, crsPath }) => {
    handleGlobalOptions();
    const result = await proveAndVerifyMegaHonk(bytecodePath, witnessPath, crsPath);
    process.exit(result ? 0 : 1);
  });

program
  .command('fold_and_verify_program')
  .description('Accumulate a set of circuits using ClientIvc then verify. Process exits with success or failure code.')
  .option('-b, --bytecode-path <path>', 'Specify the bytecode path', './target/program.json')
  .option('-w, --witness-path <path>', 'Specify the witness path', './target/witness.gz')
  .action(async ({ bytecodePath, witnessPath, crsPath }) => {
    handleGlobalOptions();
    const result = await foldAndVerifyProgram(bytecodePath, witnessPath, crsPath);
    process.exit(result ? 0 : 1);
  });

program
  .command('prove')
  .description('Generate a proof and write it to a file.')
  .option('-b, --bytecode-path <path>', 'Specify the bytecode path', './target/program.json')
  .option('-w, --witness-path <path>', 'Specify the witness path', './target/witness.gz')
  .option('-o, --output-path <path>', 'Specify the proof output path', './proofs/proof')
  .action(async ({ bytecodePath, witnessPath, outputPath, crsPath }) => {
    handleGlobalOptions();
    await prove(bytecodePath, witnessPath, crsPath, outputPath);
  });

program
  .command('gates')
  .description('Print gate count to standard output.')
  .option('-b, --bytecode-path <path>', 'Specify the bytecode path', './target/program.json')
  .option('-hr, --honk-recursion <bool>', 'Specify whether to use UltraHonk recursion', 'false')
  .action(async ({ bytecodePath: bytecodePath, honkRecursion: honkRecursion }) => {
    handleGlobalOptions();
    await gateCount(bytecodePath, honkRecursion);
  });

program
  .command('verify')
  .description('Verify a proof. Process exists with success or failure code.')
  .requiredOption('-p, --proof-path <path>', 'Specify the path to the proof')
  .requiredOption('-k, --vk <path>', 'path to a verification key. avoids recomputation.')
  .action(async ({ proofPath, vk }) => {
    handleGlobalOptions();
    const result = await verify(proofPath, vk);
    process.exit(result ? 0 : 1);
  });

program
  .command('contract')
  .description('Output solidity verification key contract.')
  .option('-b, --bytecode-path <path>', 'Specify the bytecode path', './target/program.json')
  .option('-o, --output-path <path>', 'Specify the path to write the contract', './target/contract.sol')
  .requiredOption('-k, --vk-path <path>', 'Path to a verification key. avoids recomputation.')
  .action(async ({ outputPath, vkPath }) => {
    handleGlobalOptions();
    await contract(outputPath, vkPath);
  });

program
  .command('write_vk')
  .description('Output verification key.')
  .option('-b, --bytecode-path <path>', 'Specify the bytecode path', './target/program.json')
  .option('-o, --output-path <path>', 'Specify the path to write the key')
  .action(async ({ bytecodePath, outputPath, crsPath }) => {
    handleGlobalOptions();
    await writeVk(bytecodePath, crsPath, outputPath);
  });

program
  .command('write_pk')
  .description('Output proving key.')
  .option('-b, --bytecode-path <path>', 'Specify the bytecode path', './target/program.json')
  .requiredOption('-o, --output-path <path>', 'Specify the path to write the key')
  .action(async ({ bytecodePath, outputPath, crsPath }) => {
    handleGlobalOptions();
    await writePk(bytecodePath, crsPath, outputPath);
  });

program
  .command('proof_as_fields')
  .description('Return the proof as fields elements')
  .requiredOption('-p, --proof-path <path>', 'Specify the proof path')
  .requiredOption('-k, --vk-path <path>', 'Path to verification key.')
  .requiredOption('-o, --output-path <path>', 'Specify the JSON path to write the proof fields')
  .action(async ({ proofPath, vkPath, outputPath }) => {
    handleGlobalOptions();
    await proofAsFields(proofPath, vkPath, outputPath);
  });

program
  .command('vk_as_fields')
  .description('Return the verification key represented as fields elements. Also return the verification key hash.')
  .requiredOption('-k, --vk-path <path>', 'Path to verification key.')
  .requiredOption('-o, --output-path <path>', 'Specify the JSON path to write the verification key fields and key hash')
  .action(async ({ vkPath, outputPath }) => {
    handleGlobalOptions();
    await vkAsFields(vkPath, outputPath);
  });

program
  .command('prove_ultra_honk')
  .description('Generate a proof and write it to a file.')
  .option('-b, --bytecode-path <path>', 'Specify the bytecode path', './target/program.json')
  .option('-w, --witness-path <path>', 'Specify the witness path', './target/witness.gz')
  .option('-o, --output-path <path>', 'Specify the proof output path', './proofs/proof')
  .action(async ({ bytecodePath, witnessPath, outputPath, crsPath }) => {
    handleGlobalOptions();
    await proveUltraHonk(bytecodePath, witnessPath, crsPath, outputPath);
  });

program
  .command('write_vk_ultra_honk')
  .description('Output verification key.')
  .option('-b, --bytecode-path <path>', 'Specify the bytecode path', './target/program.json')
  .requiredOption('-o, --output-path <path>', 'Specify the path to write the key')
  .action(async ({ bytecodePath, outputPath, crsPath }) => {
    handleGlobalOptions();
    await writeVkUltraHonk(bytecodePath, crsPath, outputPath);
  });

program
  .command('verify_ultra_honk')
  .description('Verify a proof. Process exists with success or failure code.')
  .requiredOption('-p, --proof-path <path>', 'Specify the path to the proof')
  .requiredOption('-k, --vk <path>', 'path to a verification key. avoids recomputation.')
  .action(async ({ proofPath, vk }) => {
    handleGlobalOptions();
    const result = await verifyUltraHonk(proofPath, vk);
    process.exit(result ? 0 : 1);
  });

program
  .command('proof_as_fields_honk')
  .description('Return the proof as fields elements')
  .requiredOption('-p, --proof-path <path>', 'Specify the proof path')
  .requiredOption('-o, --output-path <path>', 'Specify the JSON path to write the proof fields')
  .action(async ({ proofPath, outputPath }) => {
    handleGlobalOptions();
    await proofAsFieldsUltraHonk(proofPath, outputPath);
  });

program
  .command('vk_as_fields_ultra_honk')
  .description('Return the verification key represented as fields elements.')
  .requiredOption('-k, --vk-path <path>', 'Path to verification key.')
  .requiredOption('-o, --output-path <path>', 'Specify the JSON path to write the verification key fields.')
  .action(async ({ vkPath, outputPath }) => {
    handleGlobalOptions();
    await vkAsFieldsUltraHonk(vkPath, outputPath);
  });

program.name('bb.js').parse(process.argv);
