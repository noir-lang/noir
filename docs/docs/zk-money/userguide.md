# User Guide

Welcome to the new zk.money!

Here you can find all information about zk.money and how to use it safely. We recommend going through the docs before using the app.

# Overview

zk.money is a Layer 2 privacy app built on top of Aztec network.

Ethereum users can use it to shield tokens and protect their transaction data from the public. Shielding a token means having it under a zkSNARK (zero-knowledge proof cryptography) shield that protects the user’s privacy. Sending and receiving a token is anonymous, and does not publish any of the transaction’s data publicly.

### Existing Users
If you're an existing [zk.money](https://zk.money) user, please see [here](https://medium.com/aztec-protocol/zk-money-migration-guide-5bd45584b1b) for a guide to migrating funds to the new system.

If you're a new user, read on!

# Shielding Funds

### Step 1: Start shielding
When you land on the homepage, click on “Shield Now” to be taken to the wallet connection screen.

![image](https://user-images.githubusercontent.com/15220860/172759597-7394a992-7869-45e6-9ec7-684f883c194e.png)

### Step 2: Connect your crypto wallet
Connect your wallet with MetaMask or other wallets via WalletConnect.

![image](https://user-images.githubusercontent.com/15220860/172760020-8ce905ad-a046-4cc4-854e-751491078f23.png)

### Step 3: Pick a memorable alias
Your username is a recognizable alias that makes it easy for your friends to send you crypto.
- Write it down! Aztec does not keep a record, it is all encrypted. Only you know it.
- If you've forgotten your alias, you cannot re-register a new one.

⚠️ Write down your alias! Losing your alias means losing account access! ⚠️


![image](https://user-images.githubusercontent.com/15220860/172760386-bc32708c-bdf2-4310-bc5d-b9bc90082fdf.png)

Click "Register" to register your account. This may take several minutes. Do not close the window until you see the dashboard.

### Step 4: Shield ETH
Deposit at least 0.01 ETH. In order to prevent spam, you must shield at the same time as claiming an alias. You'd be able to withdraw your tokens later at any time.

If you registered with an empty wallet, or wallet has insufficient ETH, you may change to another wallet (while still connected to existing one) and shield ETH.

![image](https://user-images.githubusercontent.com/15220860/172760503-9cfd9928-721d-47a3-a72b-3b573b02539c.png)

### Step 5: The Wallet page
As you noticed when you scroll down, your initial shield deposit is confirmed but not settled.
![image](https://user-images.githubusercontent.com/15220860/172760715-87ed8103-3033-467d-8a76-c2fc897aa04d.png)

It should show one green check ✅ , which means the transaction has been sent to the Rollup Provider for settlement to Layer 1.

Hovering over the ✅ will also tell you approximately when the transaction will settle (updated in real time!).

Once you see ✅ ✅, the transaction is settled and your balance will show up in the “Net Worth” component up top on the Wallet page.

Note that “Net Worth” reflects the value of all your positions — liquid or illiquid — whereas “Available” funds reflects just your spendable balance.

These figures can therefore differ — don’t be alarmed.

![image](https://user-images.githubusercontent.com/15220860/172760977-0b8e35e9-4419-4097-99eb-80109bd64456.png)

This is how it looks once the initial registration and shield transactio have been confirmed and settled.

Congrats, now you have zkAssets! What can you do with shielded assets?


## Shield more
You can always add more funds to your account from the Wallet page by clicking on "Shield More."

![image](https://user-images.githubusercontent.com/15220860/172760777-67133ff2-c224-46c3-ac40-4c73bdd20c73.png)

It’s worth repeating deposit best practices:

- Do not deposit idiosyncratic amounts (e.g. 0.696969)
- Depositing many smaller quantities is better than depositing extremely large quantities
- Deposits are capped at launch to 5 ETH / 10,000 DAI

![image](https://user-images.githubusercontent.com/15220860/172760637-e1ee8849-d9de-44db-a442-ce17321bfc49.png)


## Send
Send zkETH / zkDAI to other zk.money usernames or any Ethereum address.

Select "Withdraw to L1" to withdraw funds to Ethereum, or "Send to L2" to send funds privately to another zk.money user.

![image](https://user-images.githubusercontent.com/15220860/172761821-36477974-ee21-4f5a-8b72-ceb7c10eb606.png)

After making your selection, simply paste the recipient details into the recipient section. For L2 transactions (the recipient is a zk.money username), the receipient will get zkETH/DAI directly into their account balance. In case the recipient is a regular Ethereum address, they will receive regular "unshielded" ETH/DAI tokens to their wallet.

On this screen you can also select your transaction speed and fee quantity. “Slow” transactions ride along with many others, and settlement time average approximately an hour. “Instant” transactions involve paying for the entire rollup to go, prompting the rollup to settle to Ethereum as soon as the proof is constructed.

Once you’ve entered your amount and speed, click Next!

After confirming transaction details, click "Confirm Transaction."

![image](https://user-images.githubusercontent.com/15220860/172761896-e141bcef-0451-4d5f-870f-896d9aaef891.png)


# Earn
Explore various DeFi protocols integrated into Aztec. At the **Earn** tab, you can filter by:

- **Type:** Fixed Yield, Staking
- **Project:** Element, Lido
- **Asset:** DAI, ETH, wstETH

![image](https://user-images.githubusercontent.com/15220860/172763263-954995a1-9edb-4025-9899-be8d5e39436e.png)

Each DeFi card shows you how many users are in the batch headed to Layer 1. The more there are in the batch, the closer the batch is to running and offering you cost savings!

## Lido

### Entering a position
On this screen you can select the amount of ETH you would like to swap for wrapped staked ETH (wstETH)

![image](https://user-images.githubusercontent.com/15220860/172762361-bdec3cac-a90c-4b0a-a801-b6a68d0c5ece.png)

You can also see an additional transaction speed and fee option called "Fast Track":
- **Batched** Is slowest and batches your transaction along with many others doing the same Lido swap for wstETH.
- **Fast Track** Is faster and pays for the full DeFi transaction.
- **Instant** Is fastest and pays for the full DeFi transaction _and_ sends the Aztec rollup to mainnet instantly.

Once the transaction is confirmed, you are returned to the Earn page where you can see your Lido staking action under DeFi Investments:
![image](https://user-images.githubusercontent.com/15220860/172763467-85547030-99f2-4893-84fe-e9da92dbcbef.png)


### Exiting a position
If you want to leave a position, navigate to your open positions on the Earn page and click "Claim and Exit."

![image](https://user-images.githubusercontent.com/15220860/172764596-35130ead-3396-4be5-a2f7-555beb985691.png)

From there you will be taken to the exit modal where you can unstake.

![image](https://user-images.githubusercontent.com/15220860/172764695-cc60078e-9566-480e-a2ce-590121b526d0.png)


## On Security

Disclaimer: It is responsible for projects using bleeding-edge cryptography, to highlight the risks of use.  Given the absence of an external audit, zk.money should be view as experimental software. Internally the Aztec team has conducted two internal audits of the network, described here. After patching the resultant security flaws, we have high confidence in the soundness and security of our cryptography. Users are nonetheless reminded to use any new cryptographic system with extreme caution, and to remember that you do so at your own risk.

## Community Support

For any remaining questions and live troubleshooting help, visit our [Discord](https://discord.gg/aztec) for support.
