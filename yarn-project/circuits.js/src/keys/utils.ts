import { GeneratorIndex } from '../constants.gen.js';
import { type KeyGenerator, type KeyPrefix } from './key_types.js';

export function getKeyGenerator(prefix: KeyPrefix): KeyGenerator {
  // We get enum key by capitalizing key prefix and concatenating it with 'SK_M'
  const enumKey = `${prefix.toUpperCase()}SK_M`;
  return GeneratorIndex[enumKey as keyof typeof GeneratorIndex] as KeyGenerator;
}
