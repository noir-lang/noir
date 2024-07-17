import { expect } from 'chai';
import { join, resolve } from 'path';

import { CompiledCircuit, ProofData } from '@noir-lang/types';
import { Noir } from '@noir-lang/noir_js';
import { BarretenbergBackend } from '@noir-lang/backend_barretenberg';
import { compile, createFileManager } from '@noir-lang/noir_wasm';

// Helper function to get compiled Noir program
async function getCircuit(name: string) {
  const basePath = resolve(join('./test', name));
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
  const noir: Noir = new Noir(circuit);
  return { circuit, backend, noir };
}

// Type to associate related Noir objects
type FullNoir = {
  circuit: CompiledCircuit;
  backend: BarretenbergBackend;
  noir: Noir;
};

async function testBenchmark(name: string) {
  console.log('Testing ', name);
  const p2_n = await fullNoirFromCircuit(name);
  console.log('Execute');
  const { witness } = await p2_n.noir.execute({ x: 1 });
  console.log('genProof from witness');
  const proof = await p2_n.backend.generateProof(witness);
  console.log('genRecProof');
  /*const p2_n_artifacts =*/ await p2_n.backend.generateRecursiveProofArtifacts(proof, 0);
  console.log('Done generating recursive proof artifacts for ', name);
}

// Calculate example sum of two leaf nodes up left branch
// S3
//  S2     9
//        / \
//       /   \
//   S1 4     5
//     / \   / \
//    1   3 #   #

describe('can verify recursive proofs', () => {
  it('does generate circuit with 2^17 gates', async () => {
    await testBenchmark('2^17');
  });

  it.skip('[BUG] -- does NOT generate circuit with 2^18 gates', async () => {
    try {
      await testBenchmark('2^18');
    } catch (error) {
      expect(error).to.be.an('Error');
    }
  });

  // TODO(https://github.com/AztecProtocol/aztec-packages/issues/6672): Reinstate this test.
  it.skip('does recursive proof', async () => {
    const leaf = await fullNoirFromCircuit('sum'); // a + b = c

    // Generate leaf proof artifacts (S1, addition of 1 and 3)

    // Leaf params of left branch
    const leafParams = { a: 1, b: 3 };
    let numPubInputs = 2;

    const leafExecution = await leaf.noir.execute(leafParams);
    console.log('leaf: %d + %d = ', ...Object.values(leafParams), Number(leafExecution.returnValue).toString());
    const innerProof1: ProofData = await leaf.backend.generateProof(leafExecution.witness);
    console.log('\n\ninnerProof1.publicInputs\n', innerProof1.publicInputs);
    console.log('Generating intermediate proof artifacts for leaf calculation...');
    const artifacts1 = await leaf.backend.generateRecursiveProofArtifacts(
      innerProof1,
      numPubInputs + 1, // +1 for public return
    );
    leaf.backend.destroy();

    const pub_inputs: string[] = [
      ...Object.values(leafParams).map((n) => Number(n).toString()),
      Number(leafExecution.returnValue).toString(),
    ];

    const a = leafExecution.returnValue;
    const b = 5; // Sum of leaf branches beneath right node

    // Generate node proof artifacts (S2: verify 1+3=4 proof, add 5)
    const nodeParams = {
      verification_key: artifacts1.vkAsFields,
      public_inputs: pub_inputs, // public, each counted individually
      key_hash: artifacts1.vkHash,
      proof: artifacts1.proofAsFields,
      num: 5,
    };
    numPubInputs = pub_inputs.length;

    const recurseLeaf = await fullNoirFromCircuit('recurseLeaf'); // verify l1 + l2 = n1, then sum n1 + n2
    const nodeExecution = await recurseLeaf.noir.execute(nodeParams);
    console.log('recurseLeaf: %d + %d = ', a, b, Number(nodeExecution.returnValue).toString());
    const innerProof2: ProofData = await recurseLeaf.backend.generateProof(nodeExecution.witness);
    console.log('Generating intermediate proof artifacts recurseLeaf...');
    //TODO: maybe not do magic +16
    const artifacts2 = await recurseLeaf.backend.generateRecursiveProofArtifacts(
      innerProof2,
      numPubInputs + 1 + 16, // +1 for public return +16 for hidden aggregation object
    );
    console.log('innerProof2.publicInputs length = ', innerProof2.publicInputs.length);
    console.log('artifacts2.proof length = ', artifacts2.proofAsFields.length);
    recurseLeaf.backend.destroy();

    // Generate outer proof artifacts (S3: verify 4+5=9)
    const outerParams = {
      verification_key: artifacts2.vkAsFields,
      public_inputs: innerProof2.publicInputs, // 20 = 4 public inputs + 16 aggregation bytes,
      key_hash: artifacts2.vkHash,
      proof: artifacts2.proofAsFields, // 93 = 109 - 16 aggregation bytes in public inputs
    };

    const recurseNode = await fullNoirFromCircuit('recurseNode'); // verify n1 + n2 = root1
    console.log('Executing...');
    const rootExecution = await recurseNode.noir.execute(outerParams);
    console.log('Generating outer proof...');
    const outerProof: ProofData = await recurseNode.backend.generateProof(rootExecution.witness);
    console.log('Verifying outer proof...');
    const resNode: boolean = await recurseNode.backend.verifyProof(outerProof);
    console.log('Verification', resNode ? 'PASSED' : 'failed');
    expect(resNode).to.be.true;

    recurseNode.backend.destroy();
  });
});
