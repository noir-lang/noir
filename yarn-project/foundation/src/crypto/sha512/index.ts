import { default as hash } from 'hash.js';

import { GrumpkinScalar } from '../../fields/fields.js';
import { type Bufferable, serializeToBuffer } from '../../serialize/serialize.js';

export const sha512 = (data: Buffer) => Buffer.from(hash.sha512().update(data).digest());

/**
 * @dev We don't truncate in this function (unlike in sha256ToField) because this function is used in situations where
 * we don't care only about collision resistance but we need the output to be uniformly distributed as well. This is
 * because we use it as a pseudo-random function.
 */
export const sha512ToGrumpkinScalar = (data: Bufferable[]) => {
  const buffer = serializeToBuffer(data);
  return GrumpkinScalar.fromBufferReduce(sha512(buffer));
};
