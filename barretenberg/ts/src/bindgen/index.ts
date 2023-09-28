import { generateRustCode } from './rust.js';
import { generateTypeScriptCode } from './typescript.js';

const [, , exp = '../exports.json', lang = 'ts'] = process.argv;

function generateCode(exports: string, lang: string) {
  switch (lang) {
    case 'ts':
      return generateTypeScriptCode(exports);
    case 'rust':
      return generateRustCode(exports);
    default:
      throw new Error(`Unknown lang: ${lang}`);
  }
}

console.log(generateCode(exp, lang));
