import { defineConfig } from 'vite';
import { viteStaticCopy } from 'vite-plugin-static-copy';

export default defineConfig({
  plugins: [
    viteStaticCopy({
      targets: [
        {
          // bb.js WASM files are usually in the dest or dist folder
          src: 'node_modules/@aztec/bb.js/dest/browser/*.wasm',
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
