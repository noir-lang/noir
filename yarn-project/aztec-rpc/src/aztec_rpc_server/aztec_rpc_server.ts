import { AcirSimulator, encodeArguments } from '@aztec/acir-simulator';
import { AztecNode } from '@aztec/aztec-node';
import {
  AztecAddress,
  CONTRACT_TREE_HEIGHT,
  CircuitsWasm,
  ContractDeploymentData,
  EcdsaSignature,
  EthAddress,
  FUNCTION_TREE_HEIGHT,
  FunctionData,
  MembershipWitness,
  PrivateCallStackItem,
  TxContext,
  TxRequest,
  UInt8Vector,
  computeFunctionTree,
} from '@aztec/circuits.js';
import { hashVK } from '@aztec/circuits.js/abis';
import { Fr, Point, createDebugLogger } from '@aztec/foundation';
import { FunctionTreeInfo, KernelProver } from '@aztec/kernel-prover';
import { ContractAbi, FunctionType } from '@aztec/noir-contracts';
import { Tx, TxHash } from '@aztec/types';
import { generateFunctionSelector } from '../abi_coder/index.js';
import { AztecRPCClient, DeployedContract } from '../aztec_rpc_client/index.js';
import { ContractDao, toContractDao } from '../contract_database/contract_dao.js';
import { ContractTree } from '../contract_tree/index.js';
import { Database } from '../database/database.js';
import { TxDao } from '../database/tx_dao.js';
import { KeyStore } from '../key_store/index.js';
import { Synchroniser } from '../synchroniser/index.js';
import { TxReceipt, TxStatus } from '../tx/index.js';
import { computeFunctionTreeData } from './function_tree_data.js';

/**
 * Implements a remote Aztec RPC client provider.
 * Combines our major components into one API.
 */
export class AztecRPCServer implements AztecRPCClient {
  private synchroniser: Synchroniser;

  constructor(
    private keyStore: KeyStore,
    acirSimulator: AcirSimulator,
    private kernelProver: KernelProver,
    private node: AztecNode,
    private db: Database,
    private log = createDebugLogger('aztec:rpc_server'),
  ) {
    this.synchroniser = new Synchroniser(node, db, acirSimulator);
  }

  public async start() {
    const accounts = await this.keyStore.getAccounts();
    for (const account of accounts) {
      await this.initAccountState(account);
    }
    this.synchroniser.start();
    this.log(`Started. ${accounts.length} initial accounts.`);
  }

  public async stop() {
    await this.synchroniser.stop();
    this.log('Stopped.');
  }

  public async addAccount() {
    const accountAddress = await this.keyStore.addAccount();
    await this.initAccountState(accountAddress);
    return accountAddress;
  }

  public async addContracts(contracts: DeployedContract[]) {
    const contractDaos = contracts.map(c => toContractDao(c.abi, c.address, c.portalContract));
    await Promise.all(contractDaos.map(c => this.db.addContract(c)));
  }

  public async getAccounts(): Promise<AztecAddress[]> {
    const accounts = this.synchroniser.getAccounts();
    return await Promise.all(accounts.map(a => a.getAddress()));
  }

  public getAccountPublicKey(address: AztecAddress): Promise<Point> {
    const account = this.ensureAccount(address);
    return Promise.resolve(account.getPublicKey());
  }

  public async getStorageAt(contract: AztecAddress, storageSlot: Fr) {
    const txAuxData = await this.db.getTxAuxData(contract, storageSlot);
    return txAuxData.map(d => d.notePreimage.items.map(item => item.value));
  }

  /**
   * Is an L2 contract deployed at this address?
   * @param contractAddress - The contract data address.
   * @returns Whether the contract was deployed.
   */
  public async isContractDeployed(contractAddress: AztecAddress): Promise<boolean> {
    return !!(await this.node.getContractData(contractAddress));
  }

  public async createDeploymentTxRequest(
    abi: ContractAbi,
    args: any[],
    portalContract: EthAddress,
    contractAddressSalt = Fr.random(),
    from?: AztecAddress,
  ) {
    const wasm = await CircuitsWasm.get();
    const fromAddress = this.ensureAccountOrDefault(from);

    const constructorAbi = abi.functions.find(f => f.name === 'constructor');
    if (!constructorAbi) {
      throw new Error('Cannot find constructor in the ABI.');
    }

    if (!constructorAbi.verificationKey) {
      throw new Error('Missing verification key for the constructor.');
    }

    const flatArgs = encodeArguments(constructorAbi, args);
    const contractTree = await ContractTree.new(abi, flatArgs, portalContract, contractAddressSalt, fromAddress);

    const functionData = new FunctionData(
      generateFunctionSelector(constructorAbi.name, constructorAbi.parameters),
      true,
      true,
    );

    const constructorVkHash = await hashVK(wasm, Buffer.from(constructorAbi.verificationKey, 'hex'));

    const functionTreeRoot = await contractTree.getFunctionTreeRoot();

    const contractDeploymentData = new ContractDeploymentData(
      Fr.fromBuffer(constructorVkHash),
      functionTreeRoot,
      contractAddressSalt,
      portalContract,
    );

    const txContext = new TxContext(false, false, true, contractDeploymentData);

    const contract = contractTree.contract;
    await this.db.addContract(contract);

    return new TxRequest(
      fromAddress,
      contract.address,
      functionData,
      flatArgs,
      Fr.random(), // nonce
      txContext,
      Fr.ZERO, // chainId
    );
  }

  public async createTxRequest(functionName: string, args: any[], to: AztecAddress, from?: AztecAddress) {
    const fromAddress = this.ensureAccountOrDefault(from);

    const contract = await this.db.getContract(to);
    if (!contract) {
      throw new Error('Unknown contract.');
    }

    const functionDao = contract.functions.find(f => f.name === functionName);
    if (!functionDao) {
      throw new Error('Unknown function.');
    }

    const flatArgs = encodeArguments(functionDao, args);

    const functionData = new FunctionData(
      functionDao.selector,
      functionDao.functionType === FunctionType.SECRET,
      false,
    );

    const txContext = new TxContext(
      false,
      false,
      false,
      new ContractDeploymentData(Fr.ZERO, Fr.ZERO, Fr.ZERO, new EthAddress(Buffer.alloc(EthAddress.SIZE_IN_BYTES))),
    );

    return new TxRequest(
      fromAddress,
      to,
      functionData,
      flatArgs,
      Fr.random(), // nonce
      txContext,
      Fr.ZERO, // chainId
    );
  }

  public signTxRequest(txRequest: TxRequest) {
    this.ensureAccount(txRequest.from);
    return this.keyStore.signTxRequest(txRequest);
  }

  public async createTx(txRequest: TxRequest, signature: EcdsaSignature) {
    const accountState = this.ensureAccount(txRequest.from);

    const { executionResult, oldRoots, contract, acirHash } = await accountState.simulate(txRequest);

    this.log(`Executing Prover...`);
    const { publicInputs } = await this.kernelProver.prove(
      txRequest,
      signature,
      executionResult,
      oldRoots,
      (callStackItem: PrivateCallStackItem) => {
        return this.getFunctionTreeInfo(contract, callStackItem);
      },
      (committment: Buffer) => {
        return this.getContractSiblingPath(committment);
      },
      acirHash,
    );

    this.log(`Proof completed!`);

    const contractAddress = txRequest.to;
    const unverifiedData = accountState.createUnverifiedData(contractAddress, executionResult.preimages.newNotes);
    const tx = new Tx(publicInputs, new UInt8Vector(Buffer.alloc(0)), unverifiedData);

    const [toContract, newContract] = txRequest.functionData.isConstructor
      ? [undefined, contractAddress]
      : [contractAddress, undefined];
    const dao = new TxDao(await tx.getTxHash(), undefined, undefined, txRequest.from, toContract, newContract, '');
    await this.db.addTx(dao);

    return tx;
  }

  private async getFunctionTreeInfo(contract: ContractDao, callStackItem: PrivateCallStackItem) {
    return await this.computeFunctionTreeInfo(contract, callStackItem);
  }

  private async computeFunctionTreeInfo(contract: ContractDao, callStackItem: PrivateCallStackItem) {
    const wasm = await CircuitsWasm.get();
    const tree = new ContractTree(contract, wasm);
    const root = await tree.getFunctionTreeRoot();
    const functionIndex =
      contract.functions.findIndex(f => f.selector.equals(callStackItem.functionData.functionSelector)) - 1;
    if (functionIndex < 0) {
      return {
        root,
        membershipWitness: MembershipWitness.makeEmpty(FUNCTION_TREE_HEIGHT, 0),
      } as FunctionTreeInfo;
    }

    const leaves = await tree.getFunctionLeaves();
    const functionTree = await computeFunctionTree(wasm, leaves);
    const functionTreeData = computeFunctionTreeData(functionTree, functionIndex);
    const membershipWitness = new MembershipWitness<typeof FUNCTION_TREE_HEIGHT>(
      FUNCTION_TREE_HEIGHT,
      functionIndex,
      functionTreeData.siblingPath,
    );
    return {
      root,
      membershipWitness,
    } as FunctionTreeInfo;
  }

  /**
   * Send a transaction.
   * @param tx - The transaction
   * @returns A hash of the transaction, used to identify it.
   */
  public async sendTx(tx: Tx): Promise<TxHash> {
    await this.node.sendTx(tx);
    return tx.getTxHash();
  }

  public async viewTx(functionName: string, args: any[], to: AztecAddress, from?: AztecAddress) {
    const txRequest = await this.createTxRequest(functionName, args, to, from);
    const accountState = this.ensureAccount(txRequest.from);
    const { executionResult } = await accountState.simulate(txRequest);

    // TODO - Return typed result based on the function abi.
    return executionResult.preimages;
  }

  /**
   * Fetchs a transaction receipt for a tx
   * @param txHash - The transaction hash
   * @returns A recipt of the transaction
   */
  public async getTxReceipt(txHash: TxHash): Promise<TxReceipt> {
    const localTx = await this.synchroniser.getTxByHash(txHash);
    const partialReceipt = {
      txHash: txHash,
      blockHash: localTx?.blockHash,
      blockNumber: localTx?.blockNumber,
      from: localTx?.from,
      to: localTx?.to,
      contractAddress: localTx?.contractAddress,
      error: '',
    };

    if (localTx?.blockHash) {
      return {
        ...partialReceipt,
        status: TxStatus.MINED,
      };
    }

    const pendingTx = await this.node.getPendingTxByHash(txHash);
    if (pendingTx) {
      return {
        ...partialReceipt,
        status: TxStatus.PENDING,
      };
    }

    // if the transaction mined it will be removed from the pending pool and there is a race condition here as the synchroniser will not have the tx as mined yet, so it will appear dropped
    // until the synchroniser picks this up

    const accountState = this.synchroniser.getAccount(localTx.from);
    if (accountState && !(await accountState.isSynchronised())) {
      // there is a pending L2 block, which means the transaction will not be in the tx pool but may be awaiting mine on L1
      return {
        ...partialReceipt,
        status: TxStatus.PENDING,
      };
    }

    // TODO we should refactor this once the node can store transactions. At that point we should query the node and not deal with block heights.

    return {
      ...partialReceipt,
      status: TxStatus.DROPPED,
      error: 'Tx dropped by P2P node.',
    };
  }

  private async initAccountState(address: AztecAddress) {
    const accountPrivateKey = await this.keyStore.getAccountPrivateKey(address);
    await this.synchroniser.addAccount(accountPrivateKey);
    this.log(`Account added: ${address.toString()}`);
  }

  private ensureAccountOrDefault(account?: AztecAddress) {
    const address = account || this.synchroniser.getAccounts()[0]?.getAddress();
    if (!address) {
      throw new Error('No accounts available in the key store.');
    }

    this.ensureAccount(address);

    return address;
  }

  private ensureAccount(account: AztecAddress) {
    const accountState = this.synchroniser.getAccount(account);
    if (!accountState) {
      throw new Error(`Unknown account: ${account.toShortString()}.`);
    }

    return accountState;
  }

  private async getContractSiblingPath(committment: Buffer) {
    const index = await this.node.findContractIndex(committment);
    if (index === undefined) {
      throw new Error('Failed to find contract');
    }
    const siblingPath = await this.node.getContractPath(index);
    return new MembershipWitness<typeof CONTRACT_TREE_HEIGHT>(
      CONTRACT_TREE_HEIGHT,
      Number(index),
      siblingPath.data.map(x => new Fr(x.readBigInt64BE())),
    );
  }
}
