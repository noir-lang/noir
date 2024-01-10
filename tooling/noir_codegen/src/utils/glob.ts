import { sync as globSync } from 'glob';

export function glob(cwd: string, patternsOrFiles: string[]): string[] {
  const matches = patternsOrFiles.map((p) => globSync(p, { ignore: 'node_modules/**', absolute: true, cwd }));
  return [...new Set(matches.flat())];
}
