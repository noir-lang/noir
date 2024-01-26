/* eslint-disable camelcase */
import { CheatCodes, DebugLogger, Fr, Wallet } from '@aztec/aztec.js';
import { AztecLmdbStore } from '@aztec/kv-store';
import { Pedersen, SparseTree, newTree } from '@aztec/merkle-tree';
import { SlowTreeContract } from '@aztec/noir-contracts/SlowTree';

import { setup } from './fixtures/utils.js';

describe('e2e_slow_tree', () => {
  let logger: DebugLogger;
  let wallet: Wallet;
  let teardown: () => Promise<void>;

  let contract: SlowTreeContract;
  let cheatCodes: CheatCodes;

  beforeAll(async () => {
    ({ teardown, logger, wallet, cheatCodes } = await setup());
    contract = await SlowTreeContract.deploy(wallet).send().deployed();
  }, 100_000);

  afterAll(() => teardown());

  it('Messing around with noir slow tree', async () => {
    const depth = 254;
    const slowUpdateTreeSimulator = await newTree(
      SparseTree,
      await AztecLmdbStore.openTmp(),
      new Pedersen(),
      'test',
      depth,
    );
    const getMembershipProof = async (index: bigint, includeUncommitted: boolean) => {
      return {
        index,
        value: Fr.fromBuffer(slowUpdateTreeSimulator.getLeafValue(index, includeUncommitted)!),
        // eslint-disable-next-line camelcase
        sibling_path: (await slowUpdateTreeSimulator.getSiblingPath(index, includeUncommitted)).toFieldArray(),
      };
    };

    const getMembershipCapsule = (proof: { index: bigint; value: Fr; sibling_path: Fr[] }) => {
      return [new Fr(proof.index), proof.value, ...proof.sibling_path];
    };

    const getUpdateProof = async (newValue: bigint, index: bigint) => {
      const beforeProof = await getMembershipProof(index, false);
      const afterProof = await getMembershipProof(index, true);

      return {
        index,
        // eslint-disable-next-line camelcase
        new_value: newValue,
        // eslint-disable-next-line camelcase
        before: { value: beforeProof.value, sibling_path: beforeProof.sibling_path },
        // eslint-disable-next-line camelcase
        after: { value: afterProof.value, sibling_path: afterProof.sibling_path },
      };
    };

    const getUpdateCapsule = (proof: {
      index: bigint;
      new_value: bigint;
      before: { value: Fr; sibling_path: Fr[] };
      after: { value: Fr; sibling_path: Fr[] };
    }) => {
      return [
        new Fr(proof.index),
        new Fr(proof.new_value),
        proof.before.value,
        ...proof.before.sibling_path,
        proof.after.value,
        ...proof.after.sibling_path,
      ];
    };

    const status = async (
      key: bigint,
      _root: { before: bigint; after: bigint; next_change: bigint },
      _leaf: { before: bigint; after: bigint; next_change: bigint },
    ) => {
      const root = await contract.methods.un_read_root(owner).view();
      const leaf = await contract.methods.un_read_leaf_at(owner, key).view();
      expect(root).toEqual(_root);
      expect(leaf).toEqual(_leaf);
    };

    const owner = wallet.getCompleteAddress().address;
    const key = owner.toBigInt();

    await contract.methods.initialize().send().wait();

    const computeNextChange = (ts: bigint) => (ts / 100n + 1n) * 100n;

    let _root = {
      before: Fr.fromBuffer(slowUpdateTreeSimulator.getRoot(true)).toBigInt(),
      after: Fr.fromBuffer(slowUpdateTreeSimulator.getRoot(true)).toBigInt(),
      next_change: 2n ** 120n - 1n,
    };
    let _leaf = { before: 0n, after: 0n, next_change: 0n };
    await status(key, _root, _leaf);
    await wallet.addCapsule(getMembershipCapsule(await getMembershipProof(key, true)));
    await contract.methods.read_at(key).send().wait();

    logger(`Updating tree[${key}] to 1 from public`);
    const t1 = computeNextChange(BigInt(await cheatCodes.eth.timestamp()));
    await contract.methods
      .update_at_public(await getUpdateProof(1n, key))
      .send()
      .wait();
    await slowUpdateTreeSimulator.updateLeaf(new Fr(1).toBuffer(), key);

    // Update below.
    _root = {
      ..._root,
      after: Fr.fromBuffer(slowUpdateTreeSimulator.getRoot(true)).toBigInt(),
      next_change: t1,
    };
    _leaf = { ..._leaf, after: 1n, next_change: t1 };
    await status(key, _root, _leaf);

    const zeroProof = await getMembershipProof(key, false);
    logger(`"Reads" tree[${zeroProof.index}] from the tree, equal to ${zeroProof.value}`);
    await wallet.addCapsule(getMembershipCapsule({ ...zeroProof, value: new Fr(0) }));
    await contract.methods.read_at(key).send().wait();

    // Progress time to beyond the update and thereby commit it to the tree.
    await cheatCodes.aztec.warp((await cheatCodes.eth.timestamp()) + 1000);
    await slowUpdateTreeSimulator.commit();
    logger('--- Progressing time to after the update ---');
    await status(key, _root, _leaf);

    logger(
      `Tries to "read" tree[${zeroProof.index}] from the tree, but is rejected as value is not ${zeroProof.value}`,
    );
    await wallet.addCapsule(getMembershipCapsule({ ...zeroProof, value: new Fr(0) }));
    await expect(contract.methods.read_at(key).simulate()).rejects.toThrowError(
      /Assertion failed: Root does not match expected/,
    );

    logger(`"Reads" tree[${key}], expect to be 1`);
    await wallet.addCapsule(getMembershipCapsule({ ...zeroProof, value: new Fr(1) }));
    await contract.methods.read_at(key).send().wait();

    logger(`Updating tree[${key}] to 4 from private`);
    const t2 = computeNextChange(BigInt(await cheatCodes.eth.timestamp()));
    await wallet.addCapsule(getUpdateCapsule(await getUpdateProof(4n, key)));
    await contract.methods.update_at_private(key, 4n).send().wait();
    await slowUpdateTreeSimulator.updateLeaf(new Fr(4).toBuffer(), key);
    _root = {
      before: Fr.fromBuffer(slowUpdateTreeSimulator.getRoot(false)).toBigInt(),
      after: Fr.fromBuffer(slowUpdateTreeSimulator.getRoot(true)).toBigInt(),
      next_change: t2,
    };
    _leaf = { before: 1n, after: 4n, next_change: t2 };

    await status(key, _root, _leaf);
  }, 200_000);
});
