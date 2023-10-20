import { SignerlessWallet, Wallet } from '@aztec/aztec.js';
import { CircuitsWasm, Fr } from '@aztec/circuits.js';
import { siloNullifier } from '@aztec/circuits.js/abis';
import { DebugLogger } from '@aztec/foundation/log';
import { TestContract } from '@aztec/noir-contracts/types';
import { AztecNode, PXE, TxStatus } from '@aztec/types';

import { setup } from './fixtures/utils.js';

describe('e2e_non_contract_account', () => {
  let aztecNode: AztecNode | undefined;
  let pxe: PXE;
  let nonContractAccountWallet: Wallet;
  let teardown: () => Promise<void>;

  let logger: DebugLogger;

  let contract: TestContract;

  beforeEach(async () => {
    let wallet: Wallet;
    ({ teardown, aztecNode, pxe, wallet, logger } = await setup(1));
    nonContractAccountWallet = new SignerlessWallet(pxe);

    logger(`Deploying L2 contract...`);
    contract = await TestContract.deploy(wallet).send().deployed();
    logger('L2 contract deployed');
  }, 100_000);

  afterEach(() => teardown());

  it('Arbitrary non-contract account can call a private function on a contract', async () => {
    const contractWithNoContractWallet = await TestContract.at(contract.address, nonContractAccountWallet);

    // Send transaction as arbitrary non-contract account
    const nullifier = new Fr(940);
    const receipt = await contractWithNoContractWallet.methods.emit_nullifier(nullifier).send().wait({ interval: 0.1 });
    expect(receipt.status).toBe(TxStatus.MINED);

    const tx = await aztecNode!.getTx(receipt.txHash);
    const expectedSiloedNullifier = siloNullifier(await CircuitsWasm.get(), contract.address, nullifier);
    const siloedNullifier = tx!.newNullifiers[1];

    expect(siloedNullifier.equals(expectedSiloedNullifier)).toBeTruthy();
  }, 120_000);
});
