// docs:start:config
export default {
  optimizeDeps: {
    esbuildOptions: { target: 'esnext' },
    exclude: ['@noir-lang/noirc_abi', '@noir-lang/acvm_js'],
  },
  // docs:end:config
  root: 'tests/fixtures/browser',
};
