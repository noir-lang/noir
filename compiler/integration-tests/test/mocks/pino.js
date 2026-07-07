const pino = () => ({
  debug: console.log,
  info: console.log,
  warn: console.warn,
  error: console.error,
  fatal: console.error,

  child: () => {
    return pino();
  },
});

export { pino };
export default pino;
