import { CompiledCircuit } from '@noir-lang/types';
// import { main } from './codegen';
import { setTimeout } from "timers/promises";

import { Noir } from '@noir-lang/noir_js';
import { BarretenbergBackend } from '@noir-lang/backend_barretenberg';
import { join, resolve } from 'path';
import { compile, createFileManager } from '@noir-lang/noir_wasm';

import { ProofData } from '@noir-lang/types';

// Helper function to get compiled Noir program
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

// Helper function to create Noir objects
async function fullNoirFromCircuit(circuitName: string): Promise<FullNoir> {
  const circuit: CompiledCircuit = await getCircuit(circuitName);
  const backend: BarretenbergBackend = new BarretenbergBackend(circuit, { threads: cores });
  const noir: Noir = new Noir(circuit, backend);
  return { circuit, backend, noir };
}

// Type to associate related Noir objects
type FullNoir = {
  circuit: CompiledCircuit,
  backend: BarretenbergBackend,
  noir: Noir
}

// Calculate example sum of two leaf nodes up left branch
// S3
//  S2     9
//        / \
//       /   \
//   S1 4     5
//     / \   / \
//    1   3 #   #


async function start() {
  // Create Noir objects for each circuit
  const leaf: FullNoir = await fullNoirFromCircuit('sum'); // a + b = c
  const recurseLeaf: FullNoir = await fullNoirFromCircuit('recurseLeaf'); // verify l1 + l2 = n1, then sum n1 + n2
  const recurseNode: FullNoir = await fullNoirFromCircuit('recurseNode'); // verify n1 + n2 = root1

  // Generate leaf proof artifacts (S1, addition of 1 and 3)

  // Leaf params of left branch
  const leafParams = { a: 1, b: 3 };
  let numPubInputs = 2;

  let { witness, returnValue } = await leaf.noir.execute(leafParams);
  console.log("leaf: %d + %d = ", ...Object.values(leafParams), Number(returnValue).toString());
  const innerProof1: ProofData = await leaf.backend.generateProof(witness);
  console.log("Generating intermediate proof artifacts for leaf calculation...");
  const artifacts1 = await leaf.backend.generateRecursiveProofArtifacts(
    innerProof1,
    numPubInputs + 1 // +1 for public return
  );

  let pub_inputs: string[] = [
    ...(Object.values(leafParams).map(n => Number(n).toString())),
    Number(returnValue).toString()
  ];

  const a = returnValue;
  const b = 5; // Sum of leaf branches beneath right node

  // Generate node proof artifacts (S2: verify 1+3=4 proof, add 5)
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
  console.log("artifacts2.proof length = ", artifacts2.proofAsFields.length);

  pub_inputs.push(returnValue.toString()); // leaf returns sum
  pub_inputs.push(returnValue.toString()); // node also coded to return same value

  // Generate outer proof artifacts (S3: verify 4+5=9)
  const outerParams = {
    verification_key: artifacts2.vkAsFields,
    public_inputs: pub_inputs,
    key_hash: artifacts2.vkHash,
    proof: artifacts2.proofAsFields // the proof size of a function that verifies another proof was expected to be 109 bytes, but was still 93
  };

  console.log("Executing...");
  ({ witness, returnValue } = await recurseNode.noir.execute(outerParams));
  console.log("Generating outer proof...");
  const outerProof: ProofData = await recurseNode.backend.generateProof(witness);
  console.log("Verifying outer proof...");
  const resNode: boolean = await recurseNode.backend.verifyProof(outerProof);
  console.log("Verification", resNode ? "PASSED" : "failed");

  // Cleanup
  recurseNode.backend.destroy();
  recurseLeaf.backend.destroy();
  leaf.backend.destroy();
}

start();
