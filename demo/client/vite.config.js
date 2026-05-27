import { defineConfig } from 'vite';
import { viteStaticCopy } from 'vite-plugin-static-copy';
import { nodePolyfills } from 'vite-plugin-node-polyfills';

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
    }),
    nodePolyfills({
      include: ['buffer', 'crypto', 'stream', 'path', 'os', 'util'],
      globals: {
        Buffer: true,
        global: true,
        process: true,
      },
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
