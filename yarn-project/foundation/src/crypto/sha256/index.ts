import { default as hash } from 'hash.js';

export const sha256 = (data: Buffer) => Buffer.from(hash.sha256().update(data).digest());
