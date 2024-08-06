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
          '0x05eb10b0bda78b5d8dc91de3d62bed7b55d73b5509521a9f8ef7269133e58a8c2c93b9572b35f9c9e07e9003ae1ca444442a165f927bce00e347dab57cc19391148730d0deec722eb6c54747df7345bc2ab3bd8e81f438b17b81ccabd9e6a3ac0708920251ccaf6664d769cbc47c8d767f64912639e13d9f9e441b225066161900c48a65eea83f1dbf217c43daf1be6ba9cefd2754f07e3cc13e81e5432e47f30dfb47c8b1e11368bec638fd9d22c696bf9c323a0fd09050745f4b7cf150bfa529a9f3062ee5f9d0a099ac53b4e1130653fb797ed2b59914a8915951d13ad8252521211957a854707af85ad40e9ab4d474a4fcbdcbe7a47866cae0db4fd86ed2261669d85a9cfbd09365a6db5d7acfe5560104a0cb893a375d6c08ffb9cbb8270be446a16361f271ac11899ee19f990c68035da18703ba00c8e9773dfe6a784a',
        );
        // NOTE: browser does not know how to serialize CompleteAddress for return, so return a string
        // otherwise returning a CompleteAddress makes result undefined.
        return completeAddress.toString();
      });
      expect(result).toBe(
        '0x05eb10b0bda78b5d8dc91de3d62bed7b55d73b5509521a9f8ef7269133e58a8c2c93b9572b35f9c9e07e9003ae1ca444442a165f927bce00e347dab57cc19391148730d0deec722eb6c54747df7345bc2ab3bd8e81f438b17b81ccabd9e6a3ac0708920251ccaf6664d769cbc47c8d767f64912639e13d9f9e441b225066161900c48a65eea83f1dbf217c43daf1be6ba9cefd2754f07e3cc13e81e5432e47f30dfb47c8b1e11368bec638fd9d22c696bf9c323a0fd09050745f4b7cf150bfa529a9f3062ee5f9d0a099ac53b4e1130653fb797ed2b59914a8915951d13ad8252521211957a854707af85ad40e9ab4d474a4fcbdcbe7a47866cae0db4fd86ed2261669d85a9cfbd09365a6db5d7acfe5560104a0cb893a375d6c08ffb9cbb8270be446a16361f271ac11899ee19f990c68035da18703ba00c8e9773dfe6a784a',
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
          await owner.addNote(extendedNote);

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
