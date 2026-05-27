import { defineConfig } from 'vite';
import { viteStaticCopy } from 'vite-plugin-static-copy';

export default defineConfig({
  plugins: [
    // NoirJS needs the barretenberg WASM files to be served
    viteStaticCopy({
      targets: [
        {
          src: 'node_modules/@aztec/bb.js/dist/*.wasm',
          dest: 'assets'
        }
      ]
    })
  ],
  optimizeDeps: {
    exclude: ['@noir-lang/backend_barretenberg', '@noir-lang/noir_js']
  }
});
