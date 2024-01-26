import { MerkleTreeId } from '@aztec/circuit-types';
import {
  AztecAddress,
  CONTRACT_TREE_HEIGHT,
  EthAddress,
  FunctionLeafPreimage,
  FunctionSelector,
  NOTE_HASH_TREE_HEIGHT,
  NewContractData,
  computeFunctionTreeData,
} from '@aztec/circuits.js';
import {
  computeContractLeaf,
  computeFunctionLeaf,
  computeFunctionTree,
  computeFunctionTreeRoot,
} from '@aztec/circuits.js/abis';
import { Fr } from '@aztec/foundation/fields';
import { AztecLmdbStore } from '@aztec/kv-store';
import { Pedersen, StandardTree } from '@aztec/merkle-tree';

describe('Data generation for noir tests', () => {
  const defaultContract = {
    address: AztecAddress.fromField(new Fr(12345)),
    portalContractAddress: EthAddress.fromField(new Fr(23456)),
    functions: [
      new FunctionLeafPreimage(FunctionSelector.fromField(new Fr(1010101)), false, true, Fr.ZERO, new Fr(1111)),
      new FunctionLeafPreimage(FunctionSelector.fromField(new Fr(2020202)), true, true, Fr.ZERO, new Fr(2222)),
      new FunctionLeafPreimage(FunctionSelector.fromField(new Fr(3030303)), false, false, Fr.ZERO, new Fr(3333)),
      new FunctionLeafPreimage(FunctionSelector.fromField(new Fr(4040404)), true, false, Fr.ZERO, new Fr(4444)),
    ],
    toString: () => 'defaultContract',
    functionTreeRoot: Fr.ZERO,
  };

  const parentContract = {
    address: AztecAddress.fromField(new Fr(667788)),
    portalContractAddress: EthAddress.fromField(new Fr(990011)),
    functions: [
      new FunctionLeafPreimage(FunctionSelector.fromField(new Fr(334455)), false, true, Fr.ZERO, new Fr(345345)),
    ],
    toString: () => 'parentContract',
    functionTreeRoot: Fr.ZERO,
  };

  const contracts = [[defaultContract], [parentContract]];

  test.each(contracts)('Computes function tree data for %s', contract => {
    const leaves = contract.functions.map(f => computeFunctionLeaf(f));

    const tree = computeFunctionTree(leaves);
    contract.functionTreeRoot = computeFunctionTreeRoot(tree);

    leaves.forEach((leaf, index) => {
      const functionTreeData = computeFunctionTreeData(tree, index);
      expect(functionTreeData.root).toEqual(contract.functionTreeRoot);
      expect({
        index,
        leaf: leaf.toString(),
        siblingPath: functionTreeData.siblingPath.map(fr => fr.toString()),
        root: functionTreeData.root.toString(),
      }).toMatchSnapshot();
    });
  });

  test('Computes contract tree data', async () => {
    const leaves = contracts.map(([contract]) => {
      const contractLeaf = computeContractLeaf(
        new NewContractData(contract.address, contract.portalContractAddress, contract.functionTreeRoot),
      );
      return contractLeaf.toBuffer();
    });

    const db = await AztecLmdbStore.openTmp();
    const tree = new StandardTree(
      db,
      new Pedersen(),
      `${MerkleTreeId[MerkleTreeId.CONTRACT_TREE]}`,
      CONTRACT_TREE_HEIGHT,
    );

    await tree.appendLeaves(leaves);
    const siblingPaths = await Promise.all(contracts.map((_, index) => tree.getSiblingPath(BigInt(index), true)));

    expect({
      siblingPaths: siblingPaths.map(siblingPath => siblingPath.toFieldArray().map(field => field.toString())),
      root: Fr.fromBuffer(tree.getRoot(true)).toString(),
    }).toMatchSnapshot();
  });

  it('Computes a note hash tree', async () => {
    const indexes = new Array(128).fill(null).map((_, i) => BigInt(i));
    const leaves = indexes.map(i => new Fr(i + 1n).toBuffer());

    const db = await AztecLmdbStore.openTmp();

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
