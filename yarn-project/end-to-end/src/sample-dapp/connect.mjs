// docs:start:all
import { createAztecRpcClient } from '@aztec/aztec.js';

const { SANDBOX_URL = 'http://localhost:8080' } = process.env;

async function main() {
  const client = createAztecRpcClient(SANDBOX_URL);
  const { chainId } = await client.getNodeInfo();
  console.log(`Connected to chain ${chainId}`);
}

main().catch(err => {
  console.error(`Error in app: ${err}`);
  process.exit(1);
});
// docs:end:all
