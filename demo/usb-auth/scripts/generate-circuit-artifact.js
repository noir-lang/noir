import { writeFile } from 'node:fs/promises';
import { compileCircuitFromFiles } from '../src/compile-circuit.js';

const circuit = await compileCircuitFromFiles(new URL('../', import.meta.url));
const moduleSource = `const circuit = ${JSON.stringify(circuit, null, 2)};\n\nexport default circuit;\n`;

await writeFile(new URL('../src/circuit-artifact.js', import.meta.url), moduleSource, 'utf8');
console.log('Wrote src/circuit-artifact.js');
