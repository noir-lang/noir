import { AztecRPCServer } from '@aztec/aztec-rpc';
import {
  Account,
  AccountContract,
  CompleteAddress,
  Entrypoint,
  FunctionCall,
  NodeInfo,
  buildPayload,
  buildTxExecutionRequest,
  hashPayload,
} from '@aztec/aztec.js';
import { GrumpkinPrivateKey, GrumpkinScalar } from '@aztec/circuits.js';
import { Schnorr } from '@aztec/circuits.js/barretenberg';
import { ContractAbi } from '@aztec/foundation/abi';
import { PrivateTokenContract, SchnorrHardcodedAccountContractAbi } from '@aztec/noir-contracts/types';

import { setup } from '../fixtures/utils.js';

// docs:start:account-contract
const PRIVATE_KEY = GrumpkinScalar.fromString('0xd35d743ac0dfe3d6dbe6be8c877cb524a00ab1e3d52d7bada095dfc8894ccfa');

/** Account contract implementation that authenticates txs using Schnorr signatures. */
class SchnorrHardcodedKeyAccountContract implements AccountContract {
  constructor(private privateKey: GrumpkinPrivateKey = PRIVATE_KEY) {}

  getContractAbi(): ContractAbi {
    // Return the ABI of the SchnorrHardcodedAccount contract.
    return SchnorrHardcodedAccountContractAbi;
  }

  getDeploymentArgs(): Promise<any[]> {
    // This contract does not require any arguments in its constructor.
    return Promise.resolve([]);
  }

  getEntrypoint(completeAddress: CompleteAddress, nodeInfo: NodeInfo): Promise<Entrypoint> {
    const privateKey = this.privateKey;
    const address = completeAddress.address;

    // Create a new Entrypoint object, whose responsibility is to turn function calls from the user
    // into a tx execution request ready to be simulated and sent.
    return Promise.resolve({
      async createTxExecutionRequest(calls: FunctionCall[]) {
        // Assemble the EntrypointPayload out of the requested calls
        const { payload, packedArguments: callsPackedArguments } = await buildPayload(calls);

        // Hash the request payload and sign it using Schnorr
        const message = await hashPayload(payload);
        const signer = await Schnorr.new();
        const signature = signer.constructSignature(message, privateKey).toBuffer();

        // Collect the payload and its signature as arguments to the entrypoint
        const args = [payload, signature];

        // Capture the entrypoint function
        const entrypointMethod = SchnorrHardcodedAccountContractAbi.functions.find(f => f.name === 'entrypoint')!;

        // Assemble and return the tx execution request
        return buildTxExecutionRequest(address, entrypointMethod, args, callsPackedArguments, nodeInfo);
      },
    });
  }
}
// docs:end:account-contract

describe('guides/writing_an_account_contract', () => {
  let context: Awaited<ReturnType<typeof setup>>;

  beforeEach(async () => {
    context = await setup(0);
  }, 60_000);

  afterEach(async () => {
    await context.aztecNode?.stop();
    if (context.aztecRpcServer instanceof AztecRPCServer) {
      await context.aztecRpcServer.stop();
    }
  });

  it('works', async () => {
    const { aztecRpcServer: rpc, logger } = context;
    // docs:start:account-contract-deploy
    const encryptionPrivateKey = GrumpkinScalar.random();
    const account = new Account(rpc, encryptionPrivateKey, new SchnorrHardcodedKeyAccountContract());
    const wallet = await account.waitDeploy();
    const address = wallet.getCompleteAddress().address;
    // docs:end:account-contract-deploy
    logger(`Deployed account contract at ${address}`);

    // docs:start:account-contract-works
    const token = await PrivateTokenContract.deploy(wallet, 100, address).send().deployed();
    logger(`Deployed token contract at ${token.address}`);

    await token.methods.mint(50, address).send().wait();
    const balance = await token.methods.getBalance(address).view();
    logger(`Balance of wallet is now ${balance}`);
    // docs:end:account-contract-works
    expect(balance).toEqual(150n);

    // docs:start:account-contract-fails
    const wrongKey = GrumpkinScalar.random();
    const wrongAccountContract = new SchnorrHardcodedKeyAccountContract(wrongKey);
    const wrongAccount = new Account(rpc, encryptionPrivateKey, wrongAccountContract, wallet.getCompleteAddress());
    const wrongWallet = await wrongAccount.getWallet();
    const tokenWithWrongWallet = await PrivateTokenContract.at(token.address, wrongWallet);

    try {
      await tokenWithWrongWallet.methods.mint(200, address).simulate();
    } catch (err) {
      logger(`Failed to send tx: ${err}`);
    }
    // docs:end:account-contract-fails
  }, 60_000);
});
