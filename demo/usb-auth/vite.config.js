import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { nodePolyfills } from 'vite-plugin-node-polyfills';

const HEAVY_EXTERNALS = ['@aztec/bb.js', '@aztec/bb.js/browser'];

export default defineConfig({
  plugins: [
    react(),
    nodePolyfills({
      include: ['buffer'],
      globals: { Buffer: true, global: true, process: false },
      protocolImports: false,
    }),
  ],
  optimizeDeps: {
    exclude: HEAVY_EXTERNALS,
    include: ['@noir-lang/noir_js'],
  },
  resolve: {
    alias: { pino: 'pino/browser.js' },
  },
  build: {
    rollupOptions: { external: HEAVY_EXTERNALS },
  },
});
