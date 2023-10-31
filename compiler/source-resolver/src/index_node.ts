/// <reference types="node" />

import { initializeResolver, read_file } from './index.js';

initializeResolver((source_id: string) => {
  let fileContent = '';
  try {
    // eslint-disable-next-line @typescript-eslint/no-var-requires
    const fs = require('fs');
    fileContent = fs.readFileSync(source_id, { encoding: 'utf8' }) as string;
  } catch (e) {
    console.log(e);
  }
  return fileContent;
});

export { initializeResolver, read_file };
