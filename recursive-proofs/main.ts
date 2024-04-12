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
  const leaf: FullNoir = await fullNoirFromCircuit('sum');

  const leafParams = { a: 1, b: 3 };
  let numPubInputs = 2;

  // Generate leaf proof artifacts
  let { witness, returnValue } = await leaf.noir.execute(leafParams);
  console.log("leaf: %d + %d = ", ...Object.values(leafParams), Number(returnValue).toString());
  const innerProof1: ProofData = await leaf.backend.generateProof(witness);
  console.log("Generating intermediate proof artifacts leaf...");
  const artifacts1 = await leaf.backend.generateRecursiveProofArtifacts(
    innerProof1,
    numPubInputs + 1 // +1 for public return
  );


  let pub_inputs: string[] = [
    ...(Object.values(leafParams).map(n => Number(n).toString())),
    Number(returnValue).toString()
  ];

  let recurseLeaf: FullNoir = await fullNoirFromCircuit('recurseLeaf');

  const a = returnValue;
  const b = 5;

  const nodeParams = {
    verification_key: artifacts1.vkAsFields,
    public_inputs: pub_inputs, // public, each counted individually
    key_hash: artifacts1.vkHash,
    proof: artifacts1.proofAsFields,
    num: 5
  };
  numPubInputs = pub_inputs.length;

  ({ witness, returnValue } = await recurseLeaf.noir.execute(nodeParams));
  console.log("recurseLeaf: %d + %d = ", a, b, Number(returnValue).toString());
  const innerProof2: ProofData = await recurseLeaf.backend.generateProof(witness);
  console.log("Generating intermediate proof artifacts recurseLeaf...");
  const artifacts2 = await recurseLeaf.backend.generateRecursiveProofArtifacts(
    innerProof2,
    numPubInputs + 1 + 16 // +1 for public return +16 for hidden aggregation object
  );
  console.log("artifacts2 generated.");

  // Generate and verify outer proof
  const outerParams = {
    verification_key: artifacts2.vkAsFields,
    public_inputs: [1, 3, 4, returnValue].map.toString(), // returns proven sum
    key_hash: artifacts2.vkHash,
    proof: artifacts2.proofAsFields
  };

  const recurseNode: FullNoir = await fullNoirFromCircuit('recurseNode');
  ({ witness, returnValue } = await recurseNode.noir.execute(outerParams));
  console.log("Generating outer proof...");
  const outerProof: ProofData = await recurseNode.backend.generateProof(witness);
  console.log("Verifying outer proof...");
  const resNode: boolean = await recurseNode.backend.verifyProof(outerProof);
  console.log("Verification", resNode ? "PASSED" : "failed");

  recurseNode.backend.destroy();
  recurseLeaf.backend.destroy();
  leaf.backend.destroy();
}

start();
