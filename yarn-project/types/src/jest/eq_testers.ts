import { L2Block } from '../l2_block.js';

/**
 * Checks if two objects are the same L2Block.
 *
 * Sometimes we might be comparing two L2Block instances that represent the same block but one of them might not have
 * calculated and filled its `blockHash` property (which is computed on demand). This function ensures both objects
 * are really the same L2Block.
 *
 * @param a - An object
 * @param b - Another object
 * @returns True if both a and b are the same L2Block
 */
export function equalL2Blocks(a: any, b: any) {
  const aAsL2Block = a && a instanceof L2Block ? a : undefined;
  const bAsL2Block = b && b instanceof L2Block ? b : undefined;

  if (aAsL2Block && bAsL2Block) {
    // we got two L2Block instances, so we can compare them
    // use a custom comparator because the blockHash property is lazily computed and one instance might not have it
    return aAsL2Block.toBuffer().equals(bAsL2Block.toBuffer());
  } else if (aAsL2Block || bAsL2Block) {
    // one value is an L2block and the other isn't. Definitely not equal.
    return false;
  } else {
    // we don't know what they are, tell Jest to keep looking
    return undefined;
  }
}
