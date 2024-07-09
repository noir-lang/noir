/* eslint-disable no-console */
import type * as AztecAccountsSchnorr from '@aztec/accounts/schnorr';
import type * as AztecAccountsSingleKey from '@aztec/accounts/single_key';
import type * as AztecAccountsTesting from '@aztec/accounts/testing';
import * as AztecJs from '@aztec/aztec.js';
import { TokenContractArtifact } from '@aztec/noir-contracts.js/Token';
import { contractArtifactToBuffer } from '@aztec/types/abi';

import { type Server } from 'http';
import Koa from 'koa';
import serve from 'koa-static';
import path, { dirname } from 'path';
import { type Browser, type Page, launch } from 'puppeteer';

declare global {
  /**
   * Helper interface to declare aztec.js within browser context.
   */
  interface Window {
    /**
     * The aztec.js library.
     */
    AztecJs: { Buffer: typeof Buffer } & typeof AztecJs &
      typeof AztecAccountsSingleKey &
      typeof AztecAccountsTesting &
      typeof AztecAccountsSchnorr;
  }
}

const __filename = AztecJs.fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const privKey = AztecJs.GrumpkinScalar.random();

export const browserTestSuite = (
  setup: () => Promise<{
    /**
     *  The webserver instance.
     */
    server: Server;
    /**
     * The webserver URL.
     */
    webServerURL: string;
    /**
     *  The PXE webserver instance.
     */
    pxeServer: Server | undefined;
    /**
     * The url of the PXE
     */
    pxeURL: string;
  }>,
  pageLogger: AztecJs.DebugLogger,
) =>
  describe('e2e_aztec.js_browser', () => {
    const initialBalance = 33n;
    const transferAmount = 3n;

    let contractAddress: AztecJs.AztecAddress;

    let app: Koa;
    let testClient: AztecJs.PXE;
    let server: Server;
    let webServerURL: string;
    let pxeServer: Server | undefined;
    let pxeURL: string;

    let browser: Browser;
    let page: Page;

    beforeAll(async () => {
      ({ server, pxeURL, pxeServer, webServerURL } = await setup());
      testClient = AztecJs.createPXEClient(pxeURL);
      await AztecJs.waitForPXE(testClient);

      app = new Koa();
      app.use(serve(path.resolve(__dirname, './web')));

      browser = await launch({
        executablePath: process.env.CHROME_BIN,
        headless: true,
        args: [
          '--no-sandbox',
          '--headless',
          '--disable-gpu',
          '--disable-dev-shm-usage',
          '--disable-software-rasterizer',
          '--remote-debugging-port=9222',
        ],
      });
      page = await browser.newPage();
      page.on('console', msg => {
        pageLogger.info(msg.text());
      });
      page.on('pageerror', err => {
        pageLogger.error(err.toString());
      });
      await page.goto(`${webServerURL}/index.html`);
      while (!(await page.evaluate(() => !!window.AztecJs))) {
        pageLogger.verbose('Waiting for window.AztecJs...');
        await AztecJs.sleep(1000);
      }
    });

    afterAll(async () => {
      await browser.close();
      server.close();
      pxeServer?.close();
    });

    it('Loads Aztec.js in the browser', async () => {
      const generatePublicKeyExists = await page.evaluate(() => {
        const { generatePublicKey } = window.AztecJs;
        return typeof generatePublicKey === 'function';
      });
      expect(generatePublicKeyExists).toBe(true);
    });

    it('Creates an account', async () => {
      const result = await page.evaluate(
        async (rpcUrl, secretKeyString) => {
          const { Fr, createPXEClient, getUnsafeSchnorrAccount } = window.AztecJs;
          const pxe = createPXEClient(rpcUrl!);
          const secretKey = Fr.fromString(secretKeyString);
          const account = getUnsafeSchnorrAccount(pxe, secretKey);
          await account.waitSetup();
          const completeAddress = account.getCompleteAddress();
          const addressString = completeAddress.address.toString();
          console.log(`Created Account: ${addressString}`);
          return addressString;
        },
        pxeURL,
        privKey.toString(),
      );
      const accounts = await testClient.getRegisteredAccounts();
      const stringAccounts = accounts.map(acc => acc.address.toString());
      expect(stringAccounts.includes(result)).toBeTruthy();
    });

    it('Deploys Token contract', async () => {
      await deployTokenContract();
    });

    it('Can access CompleteAddress class in browser', async () => {
      const result: string = await page.evaluate(() => {
        const completeAddress = window.AztecJs.CompleteAddress.fromString(
          '0x1b554ab89034d16274f0043eb2d4d1104f3d6e2b995a9b8f492dfcd881b8469522f7fcddfa3ce3e8f0cc8e82d7b94cdd740afa3e77f8e4a63ea78a239432dcab0471657de2b6216ade6c506d28fbc22ba8b8ed95c871ad9f3e3984e90d9723a7111223493147f6785514b1c195bb37a2589f22a6596d30bb2bb145fdc9ca8f1e273bbffd678edce8fe30e0deafc4f66d58357c06fd4a820285294b9746c3be9509115c96e962322ffed6522f57194627136b8d03ac7469109707f5e44190c4840c49773308a13d740a7f0d4f0e6163b02c5a408b6f965856b6a491002d073d5b00d3d81beb009873eb7116327cf47c612d5758ef083d4fda78e9b63980b2a7622f567d22d2b02fe1f4ad42db9d58a36afd1983e7e2909d1cab61cafedad6193a0a7c585381b10f4666044266a02405bf6e01fa564c8517d4ad5823493abd31de',
        );
        // NOTE: browser does not know how to serialize CompleteAddress for return, so return a string
        // otherwise returning a CompleteAddress makes result undefined.
        return completeAddress.toString();
      });
      expect(result).toBe(
        '0x1b554ab89034d16274f0043eb2d4d1104f3d6e2b995a9b8f492dfcd881b8469522f7fcddfa3ce3e8f0cc8e82d7b94cdd740afa3e77f8e4a63ea78a239432dcab0471657de2b6216ade6c506d28fbc22ba8b8ed95c871ad9f3e3984e90d9723a7111223493147f6785514b1c195bb37a2589f22a6596d30bb2bb145fdc9ca8f1e273bbffd678edce8fe30e0deafc4f66d58357c06fd4a820285294b9746c3be9509115c96e962322ffed6522f57194627136b8d03ac7469109707f5e44190c4840c49773308a13d740a7f0d4f0e6163b02c5a408b6f965856b6a491002d073d5b00d3d81beb009873eb7116327cf47c612d5758ef083d4fda78e9b63980b2a7622f567d22d2b02fe1f4ad42db9d58a36afd1983e7e2909d1cab61cafedad6193a0a7c585381b10f4666044266a02405bf6e01fa564c8517d4ad5823493abd31de',
      );
    });

    it("Gets the owner's balance", async () => {
      const result = await page.evaluate(
        async (rpcUrl, contractAddress, TokenContractArtifact) => {
          const {
            Contract,
            AztecAddress,
            createPXEClient: createPXEClient,
            getDeployedTestAccountsWallets,
          } = window.AztecJs;
          const pxe = createPXEClient(rpcUrl!);
          const [wallet] = await getDeployedTestAccountsWallets(pxe);
          const owner = wallet.getCompleteAddress().address;
          const contract = await Contract.at(AztecAddress.fromString(contractAddress), TokenContractArtifact, wallet);
          const balance = await contract.methods.balance_of_private(owner).simulate({ from: owner });
          return balance;
        },
        pxeURL,
        (await getTokenAddress()).toString(),
        TokenContractArtifact,
      );
      expect(result).toEqual(initialBalance);
    });

    it('Sends a transfer TX', async () => {
      const result = await page.evaluate(
        async (rpcUrl, contractAddress, transferAmount, TokenContractArtifact) => {
          console.log(`Starting transfer tx`);
          const {
            AztecAddress,
            Contract,
            createPXEClient: createPXEClient,
            getDeployedTestAccountsWallets,
            getUnsafeSchnorrAccount,
          } = window.AztecJs;
          const pxe = createPXEClient(rpcUrl!);
          const newReceiverAccount = await getUnsafeSchnorrAccount(pxe, AztecJs.Fr.random()).waitSetup();
          const receiverAddress = newReceiverAccount.getCompleteAddress().address;
          const [wallet] = await getDeployedTestAccountsWallets(pxe);
          const contract = await Contract.at(AztecAddress.fromString(contractAddress), TokenContractArtifact, wallet);
          await contract.methods.transfer(receiverAddress, transferAmount).send().wait();
          console.log(`Transferred ${transferAmount} tokens to new Account`);
          return await contract.methods.balance_of_private(receiverAddress).simulate({ from: receiverAddress });
        },
        pxeURL,
        (await getTokenAddress()).toString(),
        transferAmount,
        TokenContractArtifact,
      );
      expect(result).toEqual(transferAmount);
    });

    const deployTokenContract = async () => {
      const [txHash, tokenAddress] = await page.evaluate(
        async (rpcUrl, initialBalance, serializedTokenContractArtifact) => {
          const {
            DeployMethod,
            createPXEClient,
            getSchnorrAccount,
            Contract,
            Fr,
            ExtendedNote,
            Note,
            computeSecretHash,
            getDeployedTestAccountsWallets,
            INITIAL_TEST_SECRET_KEYS,
            INITIAL_TEST_SIGNING_KEYS,
            INITIAL_TEST_ACCOUNT_SALTS,
            Buffer,
            contractArtifactFromBuffer,
          } = window.AztecJs;
          // We serialize the artifact since buffers (used for bytecode) do not cross well from one realm to another
          const TokenContractArtifact = contractArtifactFromBuffer(
            Buffer.from(serializedTokenContractArtifact, 'base64'),
          );
          const pxe = createPXEClient(rpcUrl!);

          // we need to ensure that a known account is present in order to create a wallet
          const knownAccounts = await getDeployedTestAccountsWallets(pxe);
          if (!knownAccounts.length) {
            const newAccount = await getSchnorrAccount(
              pxe,
              INITIAL_TEST_SECRET_KEYS[0],
              INITIAL_TEST_SIGNING_KEYS[0],
              INITIAL_TEST_ACCOUNT_SALTS[0],
            ).waitSetup();
            knownAccounts.push(newAccount);
          }
          const owner = knownAccounts[0];
          const ownerAddress = owner.getAddress();
          const tx = new DeployMethod(
            owner.getCompleteAddress().publicKeys.hash(),
            owner,
            TokenContractArtifact,
            (a: AztecJs.AztecAddress) => Contract.at(a, TokenContractArtifact, owner),
            [owner.getCompleteAddress(), 'TokenName', 'TKN', 18],
          ).send();
          const { contract: token, txHash } = await tx.wait();

          console.log(`Contract Deployed: ${token.address}`);
          const secret = Fr.random();
          const secretHash = computeSecretHash(secret);
          const mintPrivateReceipt = await token.methods.mint_private(initialBalance, secretHash).send().wait();

          const storageSlot = token.artifact.storageLayout['pending_shields'].slot;

          const noteTypeId = token.artifact.notes['TransparentNote'].id;
          const note = new Note([new Fr(initialBalance), secretHash]);
          const extendedNote = new ExtendedNote(
            note,
            ownerAddress,
            token.address,
            storageSlot,
            noteTypeId,
            mintPrivateReceipt.txHash,
          );
          await pxe.addNote(extendedNote);

          await token.methods.redeem_shield(ownerAddress, initialBalance, secret).send().wait();

          return [txHash.toString(), token.address.toString()];
        },
        pxeURL,
        initialBalance,
        contractArtifactToBuffer(TokenContractArtifact).toString('base64'),
      );

      const txResult = await testClient.getTxReceipt(AztecJs.TxHash.fromString(txHash));
      expect(txResult.status).toEqual(AztecJs.TxStatus.SUCCESS);
      contractAddress = AztecJs.AztecAddress.fromString(tokenAddress);
    };

    const getTokenAddress = async () => {
      if (!contractAddress) {
        await deployTokenContract();
      }
      return contractAddress;
    };
  });
