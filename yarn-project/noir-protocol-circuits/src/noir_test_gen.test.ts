import {
  AztecAddress,
  CONTRACT_TREE_HEIGHT,
  CircuitsWasm,
  EthAddress,
  FunctionLeafPreimage,
  FunctionSelector,
  NOTE_HASH_TREE_HEIGHT,
  NewContractData,
  computeFunctionTree,
  computeFunctionTreeData,
} from '@aztec/circuits.js';
import { computeContractLeaf, computeFunctionLeaf } from '@aztec/circuits.js/abis';
import { Fr } from '@aztec/foundation/fields';
import { Pedersen, StandardTree } from '@aztec/merkle-tree';
import { MerkleTreeId } from '@aztec/types';

import { default as levelup } from 'levelup';
import memdown from 'memdown';

describe('Data generation for noir tests', () => {
  const selector = new FunctionSelector(1);
  const vkHash = Fr.ZERO;
  const acirHash = new Fr(12341234);
  const contractAddress = AztecAddress.fromField(new Fr(12345));
  const portalContractAddress = EthAddress.fromField(new Fr(23456));

  let functionLeaf: Fr;
  let functionTreeRoot: Fr;

  let wasm: CircuitsWasm;

  beforeAll(async () => {
    wasm = await CircuitsWasm.get();
  });

  it('Computes function leaf', () => {
    const functionLeafPreimage = new FunctionLeafPreimage(selector, false, true, vkHash, acirHash);

    functionLeaf = computeFunctionLeaf(functionLeafPreimage);

    expect(functionLeaf.toString()).toMatchSnapshot();
  });

  it('Computes function tree data', () => {
    const tree = computeFunctionTree(wasm, [functionLeaf]);

    const functionTreeData = computeFunctionTreeData(tree, 0);

    functionTreeRoot = functionTreeData.root;

    expect({
      root: functionTreeData.root.toString(),
      siblingPath: functionTreeData.siblingPath.map(fr => fr.toString()),
    }).toMatchSnapshot();
  });

  it('Computes the contract tree root', async () => {
    const contractLeaf = computeContractLeaf(
      new NewContractData(contractAddress, portalContractAddress, functionTreeRoot),
    );
    const db = levelup((memdown as any)());

    const tree = new StandardTree(
      db,
      new Pedersen(),
      `${MerkleTreeId[MerkleTreeId.CONTRACT_TREE]}`,
      CONTRACT_TREE_HEIGHT,
    );

    await tree.appendLeaves([contractLeaf.toBuffer()]);

    const siblingPath = await tree.getSiblingPath(0n, true);
    expect({
      siblingPath: siblingPath.toFieldArray().map(field => field.toString()),
      root: Fr.fromBuffer(tree.getRoot(true)).toString(),
    }).toMatchSnapshot();
  });

  it('Computes a private data tree', async () => {
    const indexes = new Array(128).fill(null).map((_, i) => BigInt(i));
    const leaves = indexes.map(i => new Fr(i + 1n).toBuffer());

    const db = levelup((memdown as any)());

    const noteHashTree = new StandardTree(
      db,
      new Pedersen(),
      `${MerkleTreeId[MerkleTreeId.NOTE_HASH_TREE]}`,
      NOTE_HASH_TREE_HEIGHT,
    );

    await noteHashTree.appendLeaves(leaves);

    const root = noteHashTree.getRoot(true);
    const siblingPaths = await Promise.all(
      indexes.map(async index => (await noteHashTree.getSiblingPath(index, true)).toFieldArray()),
    );

    expect({
      root: Fr.fromBuffer(root).toString(),
      siblingPaths: siblingPaths.map(path => path.map(field => field.toString())),
    }).toMatchSnapshot();
  });
});
