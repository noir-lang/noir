import { MerkleTreeId } from '@aztec/circuit-types';
import {
  AztecAddress,
  EthAddress,
  FunctionSelector,
  NOTE_HASH_TREE_HEIGHT,
  computeContractAddressFromInstance,
  computeContractClassId,
  computeContractClassIdPreimage,
  computeInitializationHashFromEncodedArgs,
  computePartialAddress,
  computePrivateFunctionsTree,
  computeSaltedInitializationHash,
} from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';
import { setupCustomSnapshotSerializers } from '@aztec/foundation/testing';
import { openTmpStore } from '@aztec/kv-store/utils';
import { Pedersen, StandardTree } from '@aztec/merkle-tree';
import { type ContractClass, type ContractInstance } from '@aztec/types/contracts';

describe('Data generation for noir tests', () => {
  setupCustomSnapshotSerializers(expect);

  type FixtureContractData = Omit<ContractClass, 'version' | 'publicFunctions'> &
    Pick<ContractInstance, 'publicKeysHash' | 'portalContractAddress' | 'salt'> &
    Pick<ContractClass, 'privateFunctions'> & { toString: () => string };

  const defaultContract: FixtureContractData = {
    artifactHash: new Fr(12345),
    portalContractAddress: EthAddress.fromField(new Fr(23456)),
    packedBytecode: Buffer.from([3, 4, 5, 6, 7]),
    publicKeysHash: new Fr(45678),
    salt: new Fr(56789),
    privateFunctions: [
      { selector: FunctionSelector.fromField(new Fr(1010101)), vkHash: new Fr(0) },
      { selector: FunctionSelector.fromField(new Fr(2020202)), vkHash: new Fr(0) },
    ],
    toString: () => 'defaultContract',
  };

  const parentContract: FixtureContractData = {
    artifactHash: new Fr(1212),
    portalContractAddress: EthAddress.fromField(new Fr(2323)),
    packedBytecode: Buffer.from([3, 4, 3, 4]),
    publicKeysHash: new Fr(4545),
    salt: new Fr(5656),
    privateFunctions: [{ selector: FunctionSelector.fromField(new Fr(334455)), vkHash: new Fr(0) }],
    toString: () => 'parentContract',
  };

  const constructorSelector = new FunctionSelector(999);

  const contracts = [[defaultContract], [parentContract]];

  const format = (obj: object) => JSON.stringify(obj, null, 2).replaceAll('"', '');

  test.each(contracts)('Computes contract info for %s', contract => {
    const contractClass: ContractClass = { ...contract, publicFunctions: [], version: 1 };
    const contractClassId = computeContractClassId(contractClass);
    const initializationHash = computeInitializationHashFromEncodedArgs(constructorSelector, []);
    const { artifactHash, privateFunctionsRoot, publicBytecodeCommitment } =
      computeContractClassIdPreimage(contractClass);
    const deployer = AztecAddress.ZERO;
    const instance: ContractInstance = { ...contract, version: 1, initializationHash, contractClassId, deployer };
    const address = computeContractAddressFromInstance(instance);
    const saltedInitializationHash = computeSaltedInitializationHash(instance);
    const partialAddress = computePartialAddress(instance);

    /* eslint-disable camelcase */
    expect(
      format({
        contract_address_salt: contract.salt.toString(),
        artifact_hash: artifactHash.toString(),
        public_bytecode_commitment: publicBytecodeCommitment.toString(),
        private_functions_root: privateFunctionsRoot.toString(),
        address: `AztecAddress { inner: ${address.toString()} }`,
        partial_address: `PartialAddress { inner: ${partialAddress.toString()} }`,
        portal_contract_address: `EthAddress { inner: ${contract.portalContractAddress.toString()} }`,
        contract_class_id: `ContractClassId { inner: ${contractClassId.toString()} }`,
        public_keys_hash: `PublicKeysHash { inner: ${contract.publicKeysHash.toString()} }`,
        salted_initialization_hash: `SaltedInitializationHash { inner: ${saltedInitializationHash.toString()} }`,
        deployer: `AztecAddress { inner: ${deployer.toString()} }`,
      }),
    ).toMatchSnapshot();
    /* eslint-enable camelcase */
  });

  test.each(contracts)('Computes function tree for %s', contract => {
    const tree = computePrivateFunctionsTree(contract.privateFunctions);
    expect(
      tree.leaves.map((leaf, index) => ({
        index,
        leaf: leaf.toString('hex'),
        siblingPath: tree.getSiblingPath(index).map(b => b.toString('hex')),
      })),
    ).toMatchSnapshot();
  });

  it('Computes a note hash tree', async () => {
    const indexes = new Array(128).fill(null).map((_, i) => BigInt(i));
    const leaves = indexes.map(i => new Fr(i + 1n));

    const db = openTmpStore();

    const noteHashTree = new StandardTree(
      db,
      new Pedersen(),
      `${MerkleTreeId[MerkleTreeId.NOTE_HASH_TREE]}`,
      NOTE_HASH_TREE_HEIGHT,
      0n,
      Fr,
    );

    await noteHashTree.appendLeaves(leaves);

    const root = noteHashTree.getRoot(true);
    const siblingPaths = await Promise.all(
      indexes.map(async index => (await noteHashTree.getSiblingPath(index, true)).toFields()),
    );

    expect({
      root: Fr.fromBuffer(root).toString(),
      siblingPaths: siblingPaths.map(path => path.map(field => field.toString())),
    }).toMatchSnapshot();
  });
});
