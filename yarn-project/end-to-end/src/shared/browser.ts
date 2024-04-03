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
        pageLogger(msg.text());
      });
      page.on('pageerror', err => {
        pageLogger.error(err.toString());
      });
      await page.goto(`${webServerURL}/index.html`);
      while (!(await page.evaluate(() => !!window.AztecJs))) {
        pageLogger('Waiting for window.AztecJs...');
        await AztecJs.sleep(1000);
      }
    }, 120_000);

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
        async (rpcUrl, privateKeyString) => {
          const { GrumpkinScalar, createPXEClient: createPXEClient, getUnsafeSchnorrAccount } = window.AztecJs;
          const pxe = createPXEClient(rpcUrl!);
          const privateKey = GrumpkinScalar.fromString(privateKeyString);
          const account = getUnsafeSchnorrAccount(pxe, privateKey);
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
    }, 15_000);

    it('Deploys Token contract', async () => {
      await deployTokenContract();
    }, 60_000);

    it('Can access CompleteAddress class in browser', async () => {
      const result: string = await page.evaluate(() => {
        const completeAddress = window.AztecJs.CompleteAddress.fromString(
          '0x115f123bbc6cc6af9890055821cfba23a7c4e8832377a32ccb719a1ba3a86483',
        );
        // NOTE: browser does not know how to serialize CompleteAddress for return, so return a string
        // otherwise returning a CompleteAddress makes result undefined.
        return completeAddress.toString();
      });
      // a lot of trailing 0s get added in the return value
      expect(result.slice(0, 66)).toBe('0x115f123bbc6cc6af9890055821cfba23a7c4e8832377a32ccb719a1ba3a86483');
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
          const newReceiverAccount = await getUnsafeSchnorrAccount(pxe, AztecJs.GrumpkinScalar.random()).waitSetup();
          const receiverAddress = newReceiverAccount.getCompleteAddress().address;
          const [wallet] = await getDeployedTestAccountsWallets(pxe);
          const contract = await Contract.at(AztecAddress.fromString(contractAddress), TokenContractArtifact, wallet);
          await contract.methods
            .transfer(wallet.getCompleteAddress().address, receiverAddress, transferAmount, 0)
            .send()
            .wait();
          console.log(`Transferred ${transferAmount} tokens to new Account`);
          return await contract.methods.balance_of_private(receiverAddress).simulate({ from: receiverAddress });
        },
        pxeURL,
        (await getTokenAddress()).toString(),
        transferAmount,
        TokenContractArtifact,
      );
      expect(result).toEqual(transferAmount);
    }, 60_000);

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
            computeMessageSecretHash,
            getDeployedTestAccountsWallets,
            INITIAL_TEST_ENCRYPTION_KEYS,
            INITIAL_TEST_SIGNING_KEYS,
            INITIAL_TEST_ACCOUNT_SALTS,
            Buffer,
          } = window.AztecJs;
          // We serialize the artifact since buffers (used for bytecode) do not cross well from one realm to another
          const TokenContractArtifact = JSON.parse(
            Buffer.from(serializedTokenContractArtifact, 'base64').toString('utf-8'),
            (key, value) => (key === 'bytecode' && typeof value === 'string' ? Buffer.from(value, 'base64') : value),
          );
          const pxe = createPXEClient(rpcUrl!);

          // we need to ensure that a known account is present in order to create a wallet
          const knownAccounts = await getDeployedTestAccountsWallets(pxe);
          if (!knownAccounts.length) {
            const newAccount = await getSchnorrAccount(
              pxe,
              INITIAL_TEST_ENCRYPTION_KEYS[0],
              INITIAL_TEST_SIGNING_KEYS[0],
              INITIAL_TEST_ACCOUNT_SALTS[0],
            ).waitSetup();
            knownAccounts.push(newAccount);
          }
          const owner = knownAccounts[0];
          const ownerAddress = owner.getAddress();
          const tx = new DeployMethod(
            owner.getCompleteAddress().publicKey,
            owner,
            TokenContractArtifact,
            (a: AztecJs.AztecAddress) => Contract.at(a, TokenContractArtifact, owner),
            [owner.getCompleteAddress(), 'TokenName', 'TKN', 18],
          ).send();
          const { contract: token, txHash } = await tx.wait();

          console.log(`Contract Deployed: ${token.address}`);
          const secret = Fr.random();
          const secretHash = computeMessageSecretHash(secret);
          const mintPrivateReceipt = await token.methods.mint_private(initialBalance, secretHash).send().wait();

          const storageSlot = new Fr(5);

          const noteTypeId = new Fr(84114971101151129711410111011678111116101n);
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
      expect(txResult.status).toEqual(AztecJs.TxStatus.MINED);
      contractAddress = AztecJs.AztecAddress.fromString(tokenAddress);
    };

    const getTokenAddress = async () => {
      if (!contractAddress) {
        await deployTokenContract();
      }
      return contractAddress;
    };
  });
