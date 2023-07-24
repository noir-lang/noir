const replaceInFile = require('replace-in-file');
const fs = require('fs');
const path = require('path');

const buildTarget = process.env.BUILD_TARGET;
const dynamic_imports = ['barretenberg_wasm', 'crs', 'random', 'types'];

async function replaceImports() {
  try {
    dynamic_imports.forEach(async item => {
      await replaceInFile({
        files: path.resolve(__dirname, `dest/${buildTarget}/${item}/*`),
        from: new RegExp(`'dynamic\\/${item}';`, 'g'),
        to: `'./${buildTarget}/index.js';`,
      });
    });
    const filePath = path.resolve(__dirname, `dest/${buildTarget}/barretenberg_wasm/${buildTarget}/index.js`);
    // Grab the contents for a hacky check if this has ran twice
    const contents = fs.readFileSync(filePath, 'utf8');
    // hack to allow for shared .wasm files between build targets
    if (contents.includes('../../') && !contents.includes('../../../')) {
      await replaceInFile({
        files: filePath,
        from: /\.\.\/\.\.\//g,
        to: `../../../`,
      });
    }
  } catch (error) {
    console.error('Error occurred:', error);
  }
}

replaceImports();
