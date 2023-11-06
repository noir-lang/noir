import { readFileSync } from 'node:fs';

import NoirVersion from './noir-version.json' assert { type: 'json' };

// read package.json at runtime instead of compile time so that we keep rootDir as-is in tsconfig
const pkg = JSON.parse(readFileSync(new URL('../package.json', import.meta.url), 'utf-8'));

export const AztecNrVersion = `aztec-packages-v${pkg.version}`;
export const NoirWasmVersion = pkg.dependencies['@noir-lang/noir_wasm'];
export const NoirTag = NoirVersion.tag;
export const NoirCommit = NoirVersion.commit;

export { NoirVersion };
