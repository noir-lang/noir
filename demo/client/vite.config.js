import { defineConfig } from 'vite';

export default defineConfig({
  build: {
    target: 'esnext',
  },
  optimizeDeps: {
    esbuildOptions: { target: 'esnext' },
    exclude: ['@noir-lang/noir_js', '@noir-lang/noirc_abi', '@noir-lang/acvm_js', '@aztec/bb.js'],
  },
  server: {
    port: 5174,
  },
});
