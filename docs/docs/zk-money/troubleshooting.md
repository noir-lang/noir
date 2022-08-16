# Troubleshooting Guide

## Account Not Registered Error

Most "Account Not Registered" errors can be resolved with one of two methods:

1. Double-checking your Ethereum address. Please connect to zk.money using the Ethereum address you used to sign up for your alias.

2. Mistyping or misremembering your alias. Remember that your alias is critical to access your account and forgetting or misplacing it means you may not be able to access your funds.

If either approach does not work, you may belong to one of the below cohorts whose account aliases were not migrated:

1. Users who registered **prior** to July, 2021 but **did not do an account migration in the old system**. From July, 2021 to June, 2022, accounts were required to undergo [a security update](june2021update.md). If you did not update prior to June, 2022, your alias is inaccessible in the new system. This includes users who attempted migration on old.zk.money after June, 2022.

2. Users who forgot their alias after June, 2022 and had to go through the "Forgot alias" flow on old.zk.money. If you forgot your alias for old.zk.money and recovered your account using the "Forgot alias" flow, including registering a new alias, the new alias will NOT be migrated. Your old (forgotten) alias **has** been migrated if you want to try remembering it in the new system.

###

## Hung on "Connecting to Rollup Provider" on sign-up or log-in

If you see "Connecting to Rollup Provider" for more than 60 seconds, please troubleshoot using the below steps.

You may also be presented with an "AbortError: Transaction Aborted" when attempted to sign up or log-in after a system update.

In either case, navigate to your browser console (F12 or Alt-Cmd-J) and the Application tab on the top bar. Once there, click "Storage" under the "Application" group and click "Clear site data." Refresh the page.

![image](https://user-images.githubusercontent.com/15220860/177643292-e39ce717-8a58-4916-ad51-74e10c7685d4.png)

### Firefox

If you are using Firefox, open the browser console (F12 or right click and select Inspect). Go to the Storage tab on the top bar. Select the Indexed DB tab, select https://zk.money and right click "hummus" and delete it. Refresh the page.

![firefox dev tools](https://user-images.githubusercontent.com/18372439/178279060-8c8b6d58-f0ae-4986-9649-390deaa611cb.png)

## Known issue: Deposited funds not showing in zkmoney wallet balance

Step 1: Clear Site Data - https://docs.aztec.network/zk-money/troubleshooting#:~:text=group%20and%20click%20%22-,Clear%20site%20data.,-%22%20Refresh%20the%20page

Step 2: Login to https://zk.money/balance, wait till the dashboard has loaded.

Step 3: Open Developer tool (F12). Select all levels in the console including Verbose.

![verbose](https://user-images.githubusercontent.com/4763902/184890333-a23068ae-d181-4038-8e28-f07b3e0f132d.png)


Step 4: Right click anywhere in the logs and “Save as…” to download the file.

![2022-08-16 21_00_06-zk money](https://user-images.githubusercontent.com/4763902/184889636-d2f84bfe-0dd1-4005-8573-c54f4b6a6d02.png)


Step 5: Open ticket in Discord and send over the file. 


## Frequently Asked Questions

Check the [frequently asked questions page](/how-aztec-works/faq) to see if your problem is addressed there.

If it isn't, join our [Discord server](https://discord.com/invite/UDtJr9u) and get an answer from the community.
