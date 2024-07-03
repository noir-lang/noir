import { fileURLToPath } from '@aztec/foundation/url';

import { readdir, writeFile } from 'fs/promises';
import { join } from 'path';

const content = `\
import { type NoirCompiledCircuit } from '@aztec/types/noir';
const circuit: NoirCompiledCircuit;
export = circuit;
`;

const target = fileURLToPath(new URL('../../artifacts', import.meta.url).href);
const files = await readdir(target);
for (const file of files) {
  // guard against running this script twice without cleaning the artifacts/ dir first
  if (!file.endsWith('.json')) {
    continue;
  }
  const name = file.replace('.json', '');
  await writeFile(join(target, `${name}.d.json.ts`), content);
}
