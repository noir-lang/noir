import { defineConfig } from 'vite';
import { viteStaticCopy } from 'vite-plugin-static-copy';

export default defineConfig({
  plugins: [
    viteStaticCopy({
      targets: [
        {
          src: 'node_modules/@aztec/bb.js/dest/node/barretenberg_wasm/*.wasm',
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
