import { AztecAddress, AztecRPCClient, Contract, ContractDeployer, Fr } from '@aztec/aztec.js';
import { ZkTokenContractAbi } from '@aztec/noir-contracts/examples';
import { createAztecRPCClient } from './create_aztec_rpc_client.js';

describe('e2e_zk_token_contract', () => {
  let arc: AztecRPCClient;
  let accounts: AztecAddress[];
  let contract: Contract;

  const expectStorageSlot = async (accountIdx: number, expectedBalance: bigint) => {
    // We only generate 1 note in each test. Balance is the first field of the only note.
    // TBD - how to calculate storage slot?
    const storageSlot = Fr.ZERO;
    const [[balance]] = await arc.getStorageAt(contract.address, storageSlot);
    console.log(`Account ${accountIdx} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  const expectBalance = async (accountIdx: number, expectedBalance: bigint) => {
    const balance = await contract.methods.getBalance().call({ from: accounts[accountIdx] });
    console.log(`Account ${accountIdx} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  const deployContract = async (initialBalance = 0n) => {
    const deployer = new ContractDeployer(ZkTokenContractAbi, arc);
    const receipt = await deployer.deploy(initialBalance).send().getReceipt();
    return new Contract(receipt.contractAddress!, ZkTokenContractAbi, arc);
  };

  beforeEach(async () => {
    arc = await createAztecRPCClient(2);
    accounts = await arc.getAccounts();
  });

  /**
   * Milestone 1.3
   * https://hackmd.io/AG5rb9DyTRu3y7mBptWauA
   */
  it.skip('should deploy zk token contract with initial token minted to the account', async () => {
    const initialBalance = 987n;
    await deployContract(initialBalance);
    await expectStorageSlot(0, initialBalance);
    await expectStorageSlot(1, 0n);
  });

  /**
   * Milestone 1.4
   */
  it.skip('should call mint and increase balance', async () => {
    const mintAmount = 65n;

    await deployContract();

    await expectStorageSlot(0, 0n);
    await expectStorageSlot(1, 0n);

    const receipt = await contract.methods.mint(mintAmount).send({ from: accounts[1] }).getReceipt();
    expect(receipt.status).toBe(true);

    await expectStorageSlot(0, 0n);
    await expectStorageSlot(1, mintAmount);
  });

  /**
   * Milestone 1.5
   */
  it.skip('should call transfer and increase balance of another account', async () => {
    const initialBalance = 987n;
    const transferAmount = 654n;

    await deployContract(initialBalance);

    await expectBalance(0, initialBalance);
    await expectBalance(1, 0n);

    const receipt = await contract.methods.transfer(accounts[1]).send({ from: accounts[0] }).getReceipt();
    expect(receipt.status).toBe(true);

    await expectBalance(0, initialBalance - transferAmount);
    await expectBalance(1, transferAmount);
  });
});
