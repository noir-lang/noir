import { default as levelup } from 'levelup';
import { Hasher } from '../hasher.js';
import { standardBasedTreeTestSuite } from '../test/standard_based_test_suite.js';
import { treeTestSuite } from '../test/test_suite.js';
import { StandardTree } from './standard_tree.js';
import { newTree } from '../new_tree.js';
import { loadTree } from '../load_tree.js';

const createDb = async (levelUp: levelup.LevelUp, hasher: Hasher, name: string, depth: number) => {
  return await newTree(StandardTree, levelUp, hasher, name, depth);
};

const createFromName = async (levelUp: levelup.LevelUp, hasher: Hasher, name: string) => {
  return await loadTree(StandardTree, levelUp, hasher, name);
};

treeTestSuite('StandardTree', createDb, createFromName);
standardBasedTreeTestSuite('StandardTree', createDb);
