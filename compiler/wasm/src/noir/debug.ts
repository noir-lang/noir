import { inflate } from 'pako';

/** Decompresses and decodes the debug symbols */
export function inflateDebugSymbols(debugSymbols: string) {
  return JSON.parse(inflate(Buffer.from(debugSymbols, 'base64'), { to: 'string', raw: true }));
}
