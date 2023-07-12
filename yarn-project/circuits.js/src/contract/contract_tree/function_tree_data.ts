import { Fr } from '@aztec/foundation/fields';

/**
 * Computes the root and sibling path of a given function tree and index.
 * The function takes in an array of Fr elements representing the function tree and an integer function index.
 * It returns an object containing the root element of the tree and an array of sibling path elements.
 *
 * @param functionTree - The array of Fr elements representing the function tree.
 * @param functionIndex - The integer index of the desired function in the tree.
 * @returns An object containing the root element (Fr) of the tree and an array of sibling path elements (Fr[]).
 */
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
