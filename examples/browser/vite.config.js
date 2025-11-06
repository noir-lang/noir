// docs:start:config
export default {
  optimizeDeps: {
    esbuildOptions: { target: 'esnext' },
    exclude: ['@noir-lang/noirc_abi', '@noir-lang/acvm_js', '@aztec/bb.js'],
    include: ['pino', 'buffer'],
  },
  // docs:end:config
  root: '.',
};
