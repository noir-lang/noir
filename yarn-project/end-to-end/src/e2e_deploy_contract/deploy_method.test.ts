import { AztecAddress, type DebugLogger, type PXE, type Wallet } from '@aztec/aztec.js';
import { CounterContract, StatefulTestContract } from '@aztec/noir-contracts.js';
import { TestContract } from '@aztec/noir-contracts.js/Test';
import { TokenContract } from '@aztec/noir-contracts.js/Token';

import { DeployTest } from './deploy_test.js';

describe('e2e_deploy_contract deploy method', () => {
  const t = new DeployTest('deploy method');

  let pxe: PXE;
  let logger: DebugLogger;
  let wallet: Wallet;

  const ignoredArg = AztecAddress.random();

  beforeAll(async () => {
    ({ pxe, logger, wallet } = await t.setup());
  });

  afterAll(() => t.teardown());

  it('publicly deploys and initializes a contract', async () => {
    const owner = wallet.getAddress();
    logger.debug(`Deploying stateful test contract`);
    const contract = await StatefulTestContract.deploy(wallet, owner, owner, 42).send().deployed();
    expect(await contract.methods.summed_values(owner).simulate()).toEqual(42n);
    logger.debug(`Calling public method on stateful test contract at ${contract.address.toString()}`);
    await contract.methods.increment_public_value(owner, 84).send().wait();
    expect(await contract.methods.get_public_value(owner).simulate()).toEqual(84n);
  });

  it('publicly universally deploys and initializes a contract', async () => {
    const owner = wallet.getAddress();
    const opts = { universalDeploy: true };
    const contract = await StatefulTestContract.deploy(wallet, owner, owner, 42).send(opts).deployed();
    expect(await contract.methods.summed_values(owner).simulate()).toEqual(42n);
    await contract.methods.increment_public_value(owner, 84).send().wait();
    expect(await contract.methods.get_public_value(owner).simulate()).toEqual(84n);
  });

  it('publicly deploys and calls a public function from the constructor', async () => {
    const owner = wallet.getAddress();
    const token = await TokenContract.deploy(wallet, owner, 'TOKEN', 'TKN', 18).send().deployed();
    expect(await token.methods.is_minter(owner).simulate()).toEqual(true);
  });

  it('publicly deploys and initializes via a public function', async () => {
    const owner = wallet.getAddress();
    logger.debug(`Deploying contract via a public constructor`);
    const contract = await StatefulTestContract.deployWithOpts(
      { wallet, method: 'public_constructor' },
      owner,
      ignoredArg,
      42,
    )
      .send()
      .deployed();
    expect(await contract.methods.get_public_value(owner).simulate()).toEqual(42n);
    logger.debug(`Calling a private function to ensure the contract was properly initialized`);
    const outgoingViewer = owner;
    await contract.methods.create_note(owner, outgoingViewer, 30).send().wait();
    expect(await contract.methods.summed_values(owner).simulate()).toEqual(30n);
  });

  it('deploys a contract with a default initializer not named constructor', async () => {
    logger.debug(`Deploying contract with a default initializer named initialize`);
    const opts = { skipClassRegistration: true, skipPublicDeployment: true };
    // Emitting the outgoing logs to the same address as owner to avoid having to set up another account.
    const contract = await CounterContract.deploy(wallet, 10, wallet.getAddress(), wallet.getAddress())
      .send(opts)
      .deployed();
    logger.debug(`Calling a function to ensure the contract was properly initialized`);
    // Emitting the outgoing logs to the same address as owner to avoid having to set up another account.
    await contract.methods.increment(wallet.getAddress(), wallet.getAddress()).send().wait();
    expect(await contract.methods.get_counter(wallet.getAddress()).simulate()).toEqual(11n);
  });

  it('publicly deploys a contract with no constructor', async () => {
    logger.debug(`Deploying contract with no constructor`);
    const contract = await TestContract.deploy(wallet).send().deployed();
    logger.debug(`Call a public function to check that it was publicly deployed`);
    const receipt = await contract.methods.emit_unencrypted(42).send().wait();
    const logs = await pxe.getUnencryptedLogs({ txHash: receipt.txHash });
    expect(logs.logs[0].log.data.toString('hex').replace(/^0+/, '')).toEqual('2a');
  });

  it('refuses to deploy a contract with no constructor and no public deployment', async () => {
    logger.debug(`Deploying contract with no constructor and skipping public deploy`);
    const opts = { skipPublicDeployment: true, skipClassRegistration: true };
    await expect(TestContract.deploy(wallet).prove(opts)).rejects.toThrow(/no function calls needed/i);
  });

  it.skip('publicly deploys and calls a public function in the same batched call', async () => {
    // TODO(@spalladino): Requires being able to read a nullifier on the same tx it was emitted.
  });

  it.skip('publicly deploys and calls a public function in a tx in the same block', async () => {
    // TODO(@spalladino): Requires being able to read a nullifier on the same block it was emitted.
  });
});
