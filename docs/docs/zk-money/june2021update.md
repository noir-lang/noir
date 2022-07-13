# June 2021 Security Update

TL;DR

- This update is not part of the new zk.money migration process.
- If you're being asked to update your account on [old.zk.money](https://old.zk.money), your alias is currently inaccessible in the new system. We are working to make it recoverable.

If you have not logged into the old system since June 2021 you will be prompted to update your account before being able to login.

### Explanation

In June 2021 zk.money's key signing system was changed in two ways:

1.  The signing message for generating an Aztec account public key was changed to be human readable (priorly a hex string)
2.  Accounts were required to register a spending key, also derived by signing a human readable message, removing risk of funds being stolen via snooping keys from browser storage. (Priorly viewing and spending was done with a single key, which had to be stored.)

To bring users onto this new system, it was necessary to have each user individually update their account by registering a new public key derived from the new signing message at the same time as transferring their alias and exist funds to this new public key.

Users were (and still are) guided through the necessary steps to perform this update when attempting to log into the system. If today you are being prompted to update your account, it's because you haven't logged in since June 2021.

## How this affects your alias on the new zk.money

If you didn't login to your account between June 2021 and June 2022 your alias is currently in accessible in the new zk.money. The team is currently working on tool to enable users in this situation to transfer their inaccessible alias to a new account.

### Explanation

In June **2022** (roughly one year following the security update) the registrations of all existing aliases were copied into the new Aztec Connect system. The most recent rollup then was #3120, published 01 June 2022 09:59:23 UTC. This means that every migrated alias in the new system is associated with the last Aztec account public key to have registered it for that specific point in time. Aliases that were still associated with a public key generated from the retired signing message are currently inaccessible in the new zk.money because support for signing over the retired message was dropped as an oversight.
