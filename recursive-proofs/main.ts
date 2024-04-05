import { CompiledCircuit } from '@noir-lang/types';
import { main } from './codegen';
import { setTimeout } from "timers/promises";

import { Noir } from '@noir-lang/noir_js';
import { BarretenbergBackend } from '@noir-lang/backend_barretenberg';
import { join, resolve } from 'path';
import { compile, createFileManager } from '@noir-lang/noir_wasm';

import { ProofData } from '@noir-lang/types';

async function getCircuit(name: string) {
  const basePath = resolve(join('./circuits', name));
  const fm = createFileManager(basePath);
  const compiled = await compile(fm, basePath);
  console.log(compiled.program.abi);
  if (!('program' in compiled)) {
    throw new Error('Compilation failed');
  }
  return compiled.program;
}

const input1 = { n: 2 };
const input2 = { n: 4 };

async function start() {
  let simpleCircuit: CompiledCircuit = await getCircuit('not_odd');
  let simpleBackend: BarretenbergBackend = new BarretenbergBackend(simpleCircuit, { threads: 8 });
  let simpleNoir: Noir = new Noir(simpleCircuit, simpleBackend);

  const witness1 = (await simpleNoir.execute(input1)).witness;
  let proof1: ProofData = await simpleBackend.generateProof(witness1);

  const witness2 = (await simpleNoir.execute(input2)).witness;
  let proof2: ProofData = await simpleBackend.generateProof(witness2);

  // let res = await main(/* intermediate proofs */);
  // console.log(res);

  simpleBackend.destroy();
}

start();
