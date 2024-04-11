import { CompiledCircuit } from '@noir-lang/types';
// import { main } from './codegen';
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

const cores = 12;

async function fullNoirFromCircuit(circuitName: string): Promise<FullNoir> {
  const circuit: CompiledCircuit = await getCircuit(circuitName);
  const backend: BarretenbergBackend = new BarretenbergBackend(circuit, { threads: cores });
  const noir: Noir = new Noir(circuit, backend);
  return { circuit, backend, noir };
}

type FullNoir = {
  circuit: CompiledCircuit,
  backend: BarretenbergBackend,
  noir: Noir
}

async function start() {
  const simple: FullNoir = await fullNoirFromCircuit('not_odd');
  const outer: FullNoir = await fullNoirFromCircuit('recurse');

  // Generate inner proof artifacts
  console.log("Generating intermediate proof artifacts 1...");
  const innerWitness1 = (await simple.noir.execute({ n: 2 })).witness;
  const innerProof1: ProofData = await simple.backend.generateProof(innerWitness1);
  const artifacts1 = await simple.backend.generateRecursiveProofArtifacts(innerProof1, 1);

  console.log("Generating intermediate proof artifacts 2...");
  const innerWitness2 = (await simple.noir.execute({ n: 4 })).witness;
  const innerProof2: ProofData = await simple.backend.generateProof(innerWitness2);
  const artifacts2 = await simple.backend.generateRecursiveProofArtifacts(innerProof2, 1);

  simple.backend.destroy();

  // Generate and verify outer proof
  const outerInput = {
    verification_key: artifacts1.vkAsFields,
    public_inputs: ["1"], // expect output of inner call to be "true"
    key_hash: artifacts1.vkHash,
    proof1: artifacts1.proofAsFields,
    proof2: artifacts2.proofAsFields
  };
  const outerWitness = (await outer.noir.execute(
    outerInput
  )).witness;
  try {
    console.log("Generating outer proof...");
    const outerProof: ProofData = await outer.backend.generateProof(outerWitness);
    console.log("Verifying outer proof...");
    const res: boolean = await outer.backend.verifyProof(outerProof);
    console.log("Verification", res ? "PASSED" : "failed");
  }
  catch (e) {
    console.log(e);
  }

  outer.backend.destroy();
}

start();
