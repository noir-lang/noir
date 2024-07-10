// docs:start:create_account_imports
import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { Fr, GrumpkinScalar, createPXEClient } from '@aztec/aztec.js';
// docs:end:create_account_imports
// docs:start:import_contract
import { Contract } from '@aztec/aztec.js';
// docs:end:import_contract
// docs:start:import_token_contract
import { TokenContract, TokenContractArtifact } from '@aztec/noir-contracts.js/Token';

// docs:end:import_token_contract

describe('docs_examples', () => {
  it('deploys and interacts with a token contract', async () => {
    // docs:start:full_deploy
    // docs:start:define_account_vars
    const PXE_URL = process.env.PXE_URL || 'http://localhost:8080';
    const secretKey = Fr.random();
    const signingPrivateKey = GrumpkinScalar.random();
    const pxe = createPXEClient(PXE_URL);
    // docs:end:define_account_vars

    // docs:start:create_wallet
    const wallet = await getSchnorrAccount(pxe, secretKey, signingPrivateKey).waitSetup();
    // docs:end:create_wallet

    // docs:start:deploy_contract
    const deployedContract = await TokenContract.deploy(
      wallet, // wallet instance
      wallet.getAddress(), // account
      'TokenName', // constructor arg1
      'TokenSymbol', // constructor arg2
      18,
    )
      .send()
      .deployed();
    // docs:end:deploy_contract

    // docs:start:get_contract
    const contract = await Contract.at(deployedContract.address, TokenContractArtifact, wallet);
    // docs:end:get_contract
    // docs:end:full_deploy

    // docs:start:send_transaction
    const _tx = await contract.methods.mint_public(wallet.getAddress(), 1).send().wait();
    // docs:end:send_transaction

    // docs:start:simulate_function
    const balance = await contract.methods.balance_of_public(wallet.getAddress()).simulate();
    expect(balance).toEqual(1n);
    // docs:end:simulate_function
  });
});
