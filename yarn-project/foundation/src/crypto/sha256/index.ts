import { default as hash } from 'hash.js';

import { Fr } from '../../fields/fields.js';
import { truncateAndPad } from '../../serialize/free_funcs.js';

export const sha256 = (data: Buffer) => Buffer.from(hash.sha256().update(data).digest());

export const sha256Trunc = (data: Buffer) => truncateAndPad(sha256(data));

export const sha256ToField = (data: Buffer) => Fr.fromBuffer(sha256Trunc(data));
