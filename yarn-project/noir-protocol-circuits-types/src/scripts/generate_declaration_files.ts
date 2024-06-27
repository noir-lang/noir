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
  const name = file.replace('.json', '');
  await writeFile(join(target, `${name}.d.json.ts`), content);
}
