import { sync as globSync } from 'glob';
import _ from 'lodash';
const { flatten, uniq } = _;

export function glob(cwd: string, patternsOrFiles: string[]): string[] {
  const matches = patternsOrFiles.map((p) => globSync(p, { ignore: 'node_modules/**', absolute: true, cwd }));

  return uniq(flatten(matches));
}
