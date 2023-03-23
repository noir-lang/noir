// import { InMemoryP2PCLient } from './memory_p2p_client.js';
// import { MockBlockSource } from './mocks.js';

export * from './client/index.js';

/**
 * Main function of P2P in-memory client that runs at init.
 */
// async function main() {
//   // TODO: replace with actual rollup source that gets instantiated with env variables
//   const rollupSource = new MockBlockSource();
//   const p2pClient = new InMemoryP2PCLient(rollupSource);
//   await p2pClient.start();

//   const shutdown = async () => {
//     await p2pClient.stop();
//     process.exit(0);
//   };

//   process.once('SIGINT', shutdown);
//   process.once('SIGTERM', shutdown);
// }

// main().catch(err => console.log('ERROR in main p2p function: ', err));
