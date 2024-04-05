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

async function fullNoirFromCircuit(circuitName: string): Promise<FullNoir> {
  const circuit: CompiledCircuit = await getCircuit(circuitName);
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
  // Generate inner proof artifacts
  console.log("Creating Noir from circuit...");
  const simple: FullNoir = await fullNoirFromCircuit('not_odd');

  console.log("Executing binary circuit for witness...");
  const innerInput = { n: 2 };
  const innerWitness = (await simple.noir.execute(innerInput)).witness;
  console.log("Generating intermediate proof...");
  const innerProof: ProofData = await simple.backend.generateProof(innerWitness);

  if (false) { // mess up proof
    console.log("Messing intermediate proof...");
    innerProof.proof[0] += 1;
  }

  console.log("Generating recursive proof artifacts...");
  const { proofAsFields, vkAsFields, vkHash } = await simple.backend.generateRecursiveProofArtifacts(innerProof, 1);
  // console.log({ proofAsFields, vkAsFields, vkHash });
  simple.backend.destroy();

  // Generate and verify outer proof
  console.log("Creating Noir from circuit...");
  const outer: FullNoir = await fullNoirFromCircuit('recurse');
  console.log("Executing binary circuit for witness...");
  const outerInput = {
    verification_key: vkAsFields,
    proof: proofAsFields,
    public_inputs: ["0"],
    key_hash: vkHash
  };
  const outerWitness = (await outer.noir.execute(
    outerInput
  )).witness;

  console.log("Generating outer proof...");
  const outerProof: ProofData = await outer.backend.generateProof(outerWitness);

  console.log("Verifying outer proof...");
  console.log(await outer.backend.verifyProof(outerProof));

  // console.log("Executing lib circuit function to verify inner proof");
  // let res = await main(vkAsFields, proofAsFields, ["7"], vkHash);
  // console.log(res);

  outer.backend.destroy();
}

start();
