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
  if (!('program' in compiled)) {
    throw new Error('Compilation failed');
  }
  return compiled.program;
}

const input1 = { n: 2 };
const input2 = { n: 4 };

async function fullNoirFromCircuit(circuitName: string): Promise<FullNoir> {
  const circuit: CompiledCircuit = await getCircuit('not_odd');
  const backend: BarretenbergBackend = new BarretenbergBackend(circuit, { threads: 8 });
  const noir: Noir = new Noir(circuit, backend);
  return { circuit, backend, noir };
}

type FullNoir = {
  circuit: CompiledCircuit,
  backend: BarretenbergBackend,
  noir: Noir
}

async function start() {
  console.log("Creating Noir from circuit...");
  const simple: FullNoir = await fullNoirFromCircuit('not_odd');

  console.log("Executing binary circuit for witness...");
  const witness1 = (await simple.noir.execute(input1)).witness;
  console.log("Generating intermediate proof...");
  const proof1: ProofData = await simple.backend.generateProof(witness1);

  if (1) { // mess up proof
    console.log("Generating intermediate proof...");
    proof1.proof[0] += 1;
  }

  // const witness2 = (await simple.noir.execute(input2)).witness;
  // const proof2: ProofData = await simple.backend.generateProof(witness2);
  console.log("Generating recursive proof artifacts...");
  const { proofAsFields, vkAsFields, vkHash } = await simple.backend.generateRecursiveProofArtifacts(proof1, 1);
  // console.log({ proofAsFields, vkAsFields, vkHash });

  console.log("Executing lib circuit function to verify inner proof");
  let res = await main(vkAsFields, proofAsFields, ["7"], vkHash);
  console.log(res);

  simple.backend.destroy();
}

start();
