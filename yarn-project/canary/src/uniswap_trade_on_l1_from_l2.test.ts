import { createDebugLogger, createPXEClient, getSandboxAccountsWallets, waitForSandbox } from '@aztec/aztec.js';
import { UniswapSetupContext, uniswapL1L2TestSuite } from '@aztec/end-to-end';

import { createPublicClient, createWalletClient, http } from 'viem';
import { mnemonicToAccount } from 'viem/accounts';
import { foundry } from 'viem/chains';

const { PXE_URL = 'http://localhost:8080', ETHEREUM_HOST = 'http://localhost:8545' } = process.env;
export const MNEMONIC = 'test test test test test test test test test test test junk';
const hdAccount = mnemonicToAccount(MNEMONIC);
// This tests works on forked mainnet, configured on the CI.
const EXPECTED_FORKED_BLOCK = 17514288;

// docs:start:uniswap_setup
const setup = async (): Promise<UniswapSetupContext> => {
  const logger = createDebugLogger('aztec:canary_uniswap');
  const pxe = createPXEClient(PXE_URL);
  await waitForSandbox(pxe);

  const walletClient = createWalletClient({
    account: hdAccount,
    chain: foundry,
    transport: http(ETHEREUM_HOST),
  });
  const publicClient = createPublicClient({
    chain: foundry,
    transport: http(ETHEREUM_HOST),
  });

  const [ownerWallet, sponsorWallet] = await getSandboxAccountsWallets(pxe);

  return { pxe, logger, publicClient, walletClient, ownerWallet, sponsorWallet };
};
// docs:end:uniswap_setup

uniswapL1L2TestSuite(setup, () => Promise.resolve(), EXPECTED_FORKED_BLOCK);
