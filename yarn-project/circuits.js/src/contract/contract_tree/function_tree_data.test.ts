import { Fr } from '@aztec/foundation/fields';

import { computeFunctionTreeData } from './function_tree_data.js';

const getFr = (index: number) => Fr.fromBuffer(Buffer.alloc(32, index));
const Tree = [
  // leaves
  getFr(8),
  getFr(9),
  getFr(10),
  getFr(11),
  getFr(12),
  getFr(13),
  getFr(14),
  getFr(15),
  // 1st hash level
  getFr(4),
  getFr(5),
  getFr(6),
  getFr(7),
  // 2nd hash level
  getFr(2),
  getFr(3),
  // root
  getFr(1),
];

const tests = [
  { index: 0, path: [getFr(9), getFr(5), getFr(3)] },
  { index: 1, path: [getFr(8), getFr(5), getFr(3)] },
  { index: 2, path: [getFr(11), getFr(4), getFr(3)] },
  { index: 3, path: [getFr(10), getFr(4), getFr(3)] },
  { index: 4, path: [getFr(13), getFr(7), getFr(2)] },
  { index: 5, path: [getFr(12), getFr(7), getFr(2)] },
  { index: 6, path: [getFr(15), getFr(6), getFr(2)] },
  { index: 7, path: [getFr(14), getFr(6), getFr(2)] },
];

describe('Compute Function Tree Sibling Path', () => {
  for (let i = 0; i < tests.length; i++) {
    it('should generate the correct sibling path', () => {
      const actual = computeFunctionTreeData(Tree, tests[i].index);
      const expected = {
        root: getFr(1),
        siblingPath: tests[i].path,
      };
      expect(actual).toEqual(expected);
    });
  }
});
