let loadedModule;

async function loadModule() {
  if (!loadedModule) {
    loadedModule = await import('../dest/node/index.js');
  }
  return loadedModule;
}

module.exports = { loadModule };
