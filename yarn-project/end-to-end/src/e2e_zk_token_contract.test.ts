// import { AztecNode } from '@aztec/aztec-node';
// import { AztecAddress, AztecRPCServer, Contract, ContractDeployer, Fr } from '@aztec/aztec.js';
// import { EthAddress } from '@aztec/ethereum.js/eth_address';
// import { EthereumRpc } from '@aztec/ethereum.js/eth_rpc';
// import { WalletProvider } from '@aztec/ethereum.js/provider';
// import { createDebugLogger } from '@aztec/foundation';
// import { ZkTokenContractAbi } from '@aztec/noir-contracts/examples';
// import { createAztecNode } from './create_aztec_node.js';
// import { createAztecRpcServer } from './create_aztec_rpc_client.js';
// import { createProvider, deployRollupContract, deployYeeterContract } from './deploy_l1_contracts.js';

// const ETHEREUM_HOST = 'http://localhost:8545';
// const MNEMONIC = 'test test test test test test test test test test test junk';

// const logger = createDebugLogger('aztec:e2e_zk_token_contract');

// describe('e2e_zk_token_contract', () => {
//   let provider: WalletProvider;
//   let node: AztecNode;
//   let aztecRpcServer: AztecRPCServer;
//   let rollupAddress: EthAddress;
//   let yeeterAddress: EthAddress;
//   let accounts: AztecAddress[];
//   let contract: Contract;

//   beforeAll(async () => {
//     provider = createProvider(ETHEREUM_HOST, MNEMONIC, 1);
//     const ethRpc = new EthereumRpc(provider);
//     logger('Deploying contracts...');
//     rollupAddress = await deployRollupContract(provider, ethRpc);
//     yeeterAddress = await deployYeeterContract(provider, ethRpc);
//     logger('Deployed contracts...');
//   });

//   beforeEach(async () => {
//     node = await createAztecNode(rollupAddress, yeeterAddress, ETHEREUM_HOST, provider.getPrivateKey(0)!);
//     aztecRpcServer = await createAztecRpcServer(1, node);
//     accounts = await aztecRpcServer.getAccounts();
//   });

//   afterEach(async () => {
//     await node.stop();
//     await aztecRpcServer.stop();
//   });

//   const expectStorageSlot = async (accountIdx: number, expectedBalance: bigint) => {
//     // We only generate 1 note in each test. Balance is the first field of the only note.
//     // TBD - how to calculate storage slot?
//     const storageSlot = Fr.ZERO;
//     const [[balance]] = await aztecRpcServer.getStorageAt(contract.address, storageSlot);
//     logger(`Account ${accountIdx} balance: ${balance}`);
//     expect(balance).toBe(expectedBalance);
//   };

//   const expectBalance = async (accountIdx: number, expectedBalance: bigint) => {
//     const balance = await contract.methods.getBalance().call({ from: accounts[accountIdx] });
//     logger(`Account ${accountIdx} balance: ${balance}`);
//     expect(balance).toBe(expectedBalance);
//   };

//   const deployContract = async (initialBalance = 0n) => {
//     const deployer = new ContractDeployer(ZkTokenContractAbi, aztecRpcServer);
//     const receipt = await deployer.deploy(initialBalance).send().getReceipt();
//     return new Contract(receipt.contractAddress!, ZkTokenContractAbi, aztecRpcServer);
//   };

//   /**
//    * Milestone 1.3
//    * https://hackmd.io/AG5rb9DyTRu3y7mBptWauA
//    */
//   it.skip('should deploy zk token contract with initial token minted to the account', async () => {
//     const initialBalance = 987n;
//     await deployContract(initialBalance);
//     await expectStorageSlot(0, initialBalance);
//     await expectStorageSlot(1, 0n);
//   });

//   /**
//    * Milestone 1.4
//    */
//   it.skip('should call mint and increase balance', async () => {
//     const mintAmount = 65n;

//     await deployContract();

//     await expectStorageSlot(0, 0n);
//     await expectStorageSlot(1, 0n);

//     const receipt = await contract.methods.mint(mintAmount).send({ from: accounts[1] }).getReceipt();
//     expect(receipt.status).toBe(true);

//     await expectStorageSlot(0, 0n);
//     await expectStorageSlot(1, mintAmount);
//   });

//   /**
//    * Milestone 1.5
//    */
//   it.skip('should call transfer and increase balance of another account', async () => {
//     const initialBalance = 987n;
//     const transferAmount = 654n;

//     await deployContract(initialBalance);

//     await expectBalance(0, initialBalance);
//     await expectBalance(1, 0n);

//     const receipt = await contract.methods.transfer(accounts[1]).send({ from: accounts[0] }).getReceipt();
//     expect(receipt.status).toBe(true);

//     await expectBalance(0, initialBalance - transferAmount);
//     await expectBalance(1, transferAmount);
//   });
// });
