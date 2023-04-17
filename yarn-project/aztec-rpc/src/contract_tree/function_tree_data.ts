import { Fr } from '@aztec/foundation';

export function computeFunctionTreeData(functionTree: Fr[], functionIndex: number) {
  let rowSize = Math.ceil(functionTree.length / 2);
  let rowOffset = 0;
  let index = functionIndex;
  const siblingPath: Fr[] = [];
  while (rowSize > 1) {
    const isRight = index & 1;
    siblingPath.push(functionTree[rowOffset + index + (isRight ? -1 : 1)]);
    rowOffset += rowSize;
    rowSize >>= 1;
    index >>= 1;
  }
  return {
    root: functionTree[functionTree.length - 1],
    siblingPath,
  };
}
