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
  const recursive: FullNoir = await fullNoirFromCircuit('recurse');

  // Generate artifacts recursively
  let verificationProof = {
    verification_key: Array<string>(114).fill("0x00"),
    proof: Array<string>(93).fill("0x00"),
    public_inputs: ["0x06"],
    key_hash: "0x00" as string, // circuit should skip `verify_proof` when key_hash is 0.
  };
  let sum = 1;
  const nodes = [3, 7];
  for (const [i, n] of nodes.entries()) {
    console.log("\n\nExecuting (with key_hash %s, sum %d, n %d)", verificationProof.key_hash, sum, n);
    verificationProof.public_inputs = ["4"];
    const { witness, returnValue } = await recursive.noir.execute({
      ...verificationProof,
      sum,
      n
    });
    console.log("RESULT:", returnValue, witness);
    sum = Number(returnValue) as number;
    console.log("Generating proof data...");
    const pd: ProofData = await recursive.backend.generateProof(witness);
    console.log(pd);
    if (i < nodes.length - 1) {
      console.log("Generating recursive proof artifacts (depth %d)...", i);
      ({
        proofAsFields: verificationProof.proof,
        vkAsFields: verificationProof.verification_key,
        vkHash: verificationProof.key_hash
      } = await recursive.backend.generateRecursiveProofArtifacts(
        pd,
        Number(verificationProof.public_inputs[0])
      ));
    }
    else {
      console.log("Verifying final proof...");
      try {
        const res: boolean = await recursive.backend.verifyProof(pd);
        console.log("Verification", res ? "PASSED" : "failed");
      }
      catch (e) {
        console.log(e);
      }

    }
  }

  recursive.backend.destroy();
}

start();
