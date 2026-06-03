import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { nodePolyfills } from 'vite-plugin-node-polyfills';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

// Workspace root — two levels up from demo/usb-auth.
// Vite must be allowed to serve @aztec/bb.js worker scripts that live here.
const workspaceRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

export default defineConfig({
  plugins: [
    react(),
    nodePolyfills({
      include: ['buffer'],
      globals: { Buffer: true, global: true, process: false },
      protocolImports: false,
    }),
  ],
  server: {
    fs: {
      // Allow Vite to serve files from the monorepo root so that
      // @aztec/bb.js worker scripts (main.worker.js, thread.worker.js)
      // in the workspace-root node_modules can be fetched.
      allow: [workspaceRoot],
    },
  },
  optimizeDeps: {
    // Keep bb.js out of esbuild pre-bundling — it embeds WASM as data URIs
    // and spawns workers; esbuild cannot handle either pattern.
    exclude: ['@aztec/bb.js'],
    include: ['@noir-lang/noir_js'],
  },
  build: {
    rollupOptions: {
      external: ['@aztec/bb.js'],
    },
  },
});
