import { type GeneratorIndex } from '../constants.gen.js';

export type KeyGenerator = GeneratorIndex.NSK_M | GeneratorIndex.IVSK_M | GeneratorIndex.OVSK_M | GeneratorIndex.TSK_M;
export type KeyPrefix = 'n' | 'iv' | 'ov' | 't';
export const KEY_PREFIXES: KeyPrefix[] = ['n', 'iv', 'ov', 't'];
