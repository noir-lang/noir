import { defineConfig, normalizePath } from 'vite';
import { viteStaticCopy } from 'vite-plugin-static-copy';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const clientDir = path.dirname(fileURLToPath(import.meta.url));
const localWasmDir = path.join(clientDir, 'node_modules', '@aztec', 'bb.js', 'dest', 'node', 'barretenberg_wasm');
const workspaceWasmDir = path.join(clientDir, '..', 'node_modules', '@aztec', 'bb.js', 'dest', 'node', 'barretenberg_wasm');
const barretenbergWasmDir = fs.existsSync(localWasmDir) ? localWasmDir : workspaceWasmDir;

export default defineConfig({
  plugins: [
    viteStaticCopy({
      targets: [
        {
          src: normalizePath(path.join(barretenbergWasmDir, '*.wasm')),
          dest: 'assets'
        }
      ]
    })
  ],
  build: {
    target: 'esnext'
  },
  optimizeDeps: {
    esbuildOptions: {
      target: 'esnext'
    },
    exclude: [
      '@noir-lang/noir_js',
      '@noir-lang/noirc_abi',
      '@noir-lang/acvm_js',
      '@aztec/bb.js'
    ]
  }
});
