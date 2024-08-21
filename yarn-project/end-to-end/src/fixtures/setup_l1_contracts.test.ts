import { type DebugLogger, createDebugLogger } from '@aztec/foundation/log';

import { type Anvil } from '@viem/anvil';
import { type PrivateKeyAccount } from 'viem';
import { privateKeyToAccount } from 'viem/accounts';

import { setupL1Contracts, startAnvil } from './utils.js';

describe('deploy_l1_contracts', () => {
  let anvil: Anvil;
  let rpcUrl: string;
  let privateKey: PrivateKeyAccount;
  let logger: DebugLogger;

  beforeAll(async () => {
    logger = createDebugLogger('aztec:setup_l1_contracts');
    privateKey = privateKeyToAccount('0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba');

    ({ anvil, rpcUrl } = await startAnvil());
  });

  afterAll(async () => {
    await anvil.stop();
  });

  const deploy = (salt: number | undefined) => setupL1Contracts(rpcUrl, privateKey, logger, { salt });

  it('deploys without salt', async () => {
    await deploy(undefined);
  });

  it('deploys with salt on different addresses', async () => {
    const first = await deploy(42);
    const second = await deploy(43);

    expect(first.l1ContractAddresses).not.toEqual(second.l1ContractAddresses);
  });

  it('deploys twice with salt on same addresses', async () => {
    const first = await deploy(44);
    const second = await deploy(44);

    expect(first.l1ContractAddresses).toEqual(second.l1ContractAddresses);
  });
});
