# Infinite Privacy (Sets)

In the last article in our series on Aztec’s privacy architecture, we explored how a private network is even possible on a public blockchain.

Today we’re exploring why a private network is endlessly more private than simple solutions like privacy mixers.

Just like mixers, Aztec offers basic deposits and withdrawals — meant to break the link between a sending address and receiving address: our current private payments front-end zk.money has been live on mainnet since March and recently crossed 4,800 ETH in deposits bridged.

But zk.money also offers private internal transfers and will soon offer a full suite of Ethereum Layer 1 Defi functionality enabled by our bridge, Aztec Connect.

The addition of these anonymizing activities means Aztec will offer a large and dynamic privacy set that will become increasingly more difficult to de-anonymize —a concept we like to call Infinite Privacy.

## The Infinite City
Imagine Aztec as a walled city. All an outside observer can see is users entering and leaving Aztec via our bridge.

Within the walls of the city, users can exchange assets with fully private transactions. Neither the network nor its participants can see the senders and recipients of transactions, nor their amounts.

In addition, once inside the system, users can batch transactions and teleport back to L1 — to swap, stake for yield, lend funds, vote in DAOs, or buy NFTs. In the near future the contributors to ConstitutionDAO will be able to do so privately, and with nearly 0 gas.

Because Aztec allows for these two new anonymizing activities — internal transactions and batched interactions with Layer 1 Ethereum— the privacy set is significantly harder for an observer to calculate than, for instance, on a privacy mixer without those features.

That’s a very good thing.

## Sleuthing and Deducing
Let’s put ourselves in the shoes of an adversary attempting to run de-anonymizing transaction graph analysis.

As an observer watching Ethereum activity, we might watch deposits to and from Aztec, and attempt to deduce what set of deposits a certain withdrawal might belong to.

This is what we mean by privacy or anonymity set — the group or set of users a forensic target could be. If the privacy set the target belongs to is large, then we can only guess with a small probability which addresses and transactions the target is associated with.

Once the privacy set you belong to approaches 1, the probability an observer knows who you are also approaches 1, and your privacy is no longer protected.

Let’s talk through an example.

## Anonymity Sets 101: Mixer Math
Pretend Aztec were a simple privacy mixer without internal transactions, and we were internet sleuths trying to de-anonymize the network 🕵.️

If we saw someone withdraw 1 ETH, we’d know for certain that they’d deposited at least 1 ETH into the mixer. Because there are no internal transfers, aggregation of multiple deposits into a larger withdrawal simply isn’t possible.

“So,” we’d puzzle, “all we need to figure out is how many people have ever deposited at least 1 ETH, and then the withdrawer must be one of those people!”

Good thing we have Dune Analytics to help us visualize how some of these privacy sets might work! You can see our privacy set dashboard here: https://dune.xyz/jaosef/Aztec-2.

The answer to the question of who the 1 ETH withdrawal could be in this case would be “everyone to the right of 1 ETH in this diagram,” which turns out to be 1,174 deposits.

![image](https://user-images.githubusercontent.com/15220860/174936505-ea2afa43-ba15-4921-b8ea-6c8c3d69c0f7.png)


## The privacy set in a world without internal transfers.
Of course, the probability that a 1-ETH withdrawer came from the 1 ETH deposit set is much higher than the probability that she came from the >1 ETH deposit set, for a purely behavioral reason:

It’s annoying to break 5-, 10-, or 30-ETH deposits into smaller 1 ETH withdrawals. It’s much simpler to do one big monolithic withdrawal.

So as sophisticated sleuths, our investigative instincts would say that there is some non-zero but small probability that the withdrawer deposited an amount >1 ETH, with that probability diminishing for larger deposits:
![image](https://user-images.githubusercontent.com/15220860/174936543-5f6c3606-2271-41de-a5db-c4c73724cea6.png)


This is an example of a simple probability distribution — and the “spikier” it is, the more certainty an adversary has about user behaviors.

In this case, based on observations of other privacy mixers and other comparable behaviors on Defi, a forensic analyst might think the probability of a 1 ETH withdrawal coming from a 5 ETH deposit is 5x lower than the probability of a 1 ETH withdrawal coming from a 1 ETH deposit.

## Standing In or Standing Out
Let’s establish a rough heuristic guide to thinking about privacy sets:

In order to figure out how to blend in, figure out how to stand out, and then do the opposite.

The most obvious way for me to stand out in the case without internal transactions would be if I bridged a massive monolithic deposit and withdrew the same amount shortly afterward.

To make myself even more highly identifiable, I would use a unique quantity of a certain asset (e.g. depositing 69.696969 ETH to Aztec, then subsequently withdrawing 69.696969 ETH). To prevent de-anonymizing behavior, the zk.money front-end suggests round-number deposits and withdrawals — so you don’t stand out — and will soon also nudge users toward the largest anonymity sets.

As users what we want to do is introduce uncertainty into any forensic analysis. Keen observers should feel frustrated by our actions. They should say something like, “Dang, calculating the probability that these two addresses are affiliated is so complex and low-probability that it’s not meaningful for me to try to figure out which deposit is related to which withdrawal.”

I sum our conclusions in this extremely sophisticated 2x2 matrix of behaviors:
![image](https://user-images.githubusercontent.com/15220860/174936558-a0f87f4f-dadb-4724-97b2-812a30bd99cc.png)


## Very. Sophisticated.
So given what we know, how as a collective can we introduce more uncertainty into the adversary’s analysis?

Increase the size of each deposit set, especially large deposits
“Spread out” the probability that the withdrawal could have come from any deposit set!
Simple mixers focus on #1. Let’s talk about what #2 adds.

Internal Transfers: The Inner Sanctum
There is a big caveat here that differentiates our current zk.money front-end from mixers — there is a possibility that your anonymity set includes deposits that are smaller than your withdrawal amount.

How? Because internal transfers.

Say you withdraw 10 ETH. You could have deposited 10 ETH.

Or you could have deposited 5 ETH, used or coordinated offchain with 5 unassociated addresses to deposit and send you 1 ETH each internally, and then in the end withdrew 10 ETH. Suddenly you could be, well, pretty much anyone, so long as two things hold:

The amount of assets in the system is sufficient to support the hypothetical withdrawal scenario
The number of transactions in the intervening time exceeds the number needed to compose the hypothetical withdrawal scenario
So in this case, you could be some combination of internal transactions summing to 10E:

* 10 internal 1E transfers
* 100 internal 0.1E transfers
* 1,000 internal 0.01E transfers
* Some combination of the above

Of course, as the number of internal transfers needed to sum to a withdrawal amount increases, the less likely it actually happened — realistically, who’s going to coordinate 1,000 unaffiliated addresses to privately send them funds!

That’s why the internal economy of Aztec matters, and why Aztec Connect — a bridge allowing anyone to interact with defi contracts on Layer 1 — will help explode the anonymity set, making it highly improbable to associate any deposits with withdrawals.

Internal transfers muddy up anonymity set calculations, but only if there are a sufficient number of internal transfers and a large enough quantity of deposits in the system.

Aztec Connect: The Bridge to Infinity
Now let’s introduce the idea of Aztec Connect, an expansion of zk.money’s functionality to include batch interactions with any Layer 1 smart contract.

That means to begin with, any Layer 1 Defi functionality will be available to Aztec users.

Deposit $ETH on zk.money? On a mixer, you’d have to wait for the funds to be “seasoned” before withdrawal— essentially waiting for the privacy set to grow before withdrawing funds. That’s why for instance there are billions sitting in privacy mixers.

Using Aztec Connect, you can bridge funds back to Layer 1 and make shielded assets productive while you wait — meaning you might not want to ever withdraw!

With a simple conventional privacy mixer, you deposit funds and simply wait — for what, you ask? For the anonymity set to grow! Meanwhile, deposited funds are completely unproductive. Capital efficiency, schmapital efficiency.


Here are some arbitrary possible examples of using private assets while they are inside the system:

Staking ETH in Lido for stETH, depositing it in the stETH-ETH pool and getting double yield
Entering into an Element.fi fixed rate yield vault
Swapping ETH for OHM on OlympusDAO, staking for sOHM, and (3, 3)ing privately
This is just a teaser! We’ll cover Aztec Connect in depth later in this series, but for now I want to focus on the idea that investing capital for yield on Aztec will grow the value locked in the system.

Bridging back to Ethereum functionally has the same privacy-set-expanding benefits as having new users deposit fresh funds or having you deposit more funds, while being privacy-protected.

This spreads out the probability distribution:
![image](https://user-images.githubusercontent.com/15220860/174936603-8563d1dc-3404-49bc-8f6d-1ff216bf0062.png)

And makes it less and less likely that you belong to any one given depositor set. Imagine depositing 0.1 ETH and (3, 3)ing on OlympusDAO until your OHM is worth 1 ETH! Now you’ve really thrown off the scent.

## Purify Before Entering
Now, what Aztec doesn’t do is protect users on mainnet, and poor security hygiene on Ethereum can hurt user privacy.

But there’s some good news here — simply follow privacy best-practices.

Let’s start with one of the biggest no-no’s for any privacy preservation system: withdrawing to the same address.


**Don’t do this.**
Why is withdrawing to the same address “bad?” In addition to reducing your own anonymity, you’re basically screwing everyone else over. You’re reducing the anonymity set by removing yourself from it, saying, “I’m taking my ball back.”

Now there’s no way your deposit could actually be the source of anyone else’s withdrawal but your own!

While Aztec isn’t a mixer, this analogy might help clear things up:

Everyone chucks their balls into a giant ball pit. Now we play in the ball pit, trading balls, mixing them up, splashing around. And at the end of the day, everyone takes a ball and goes home.

It’s unclear who brought and took home which balls, right?

Withdrawing to the same address you deposited is akin to saying, “This is the ball I brought! It’s special because my mommy gave it to me.” Okay Jimmy, that’s fine, but imagine if everyone did that. If everyone identified themselves and their funds, it would:

* defeat the purpose of using private transfers in the first place
* harm everyone else’s ability to blend in!

Now consider the inverse: a large number of addresses deposit, and a large number (but not the same!) addresses withdraw. Now we’d have a very hard time associating one wallet with another.

Critically, any transaction graph analysis on Layer 1 may be able to associate those accounts and therefore collapse many addresses. Depositing to Aztec and withdrawing to an address already associated with the depository address is akin to withdrawing to the same address. That’s why withdrawals should only happen to untouched or otherwise unaffiliated wallets.

## Hygiene takeaways:

* Use common deposit and withdrawal amounts
* Avoid making large deposits or large withdrawals (though large withdrawals are worse)
* Don’t withdraw to the same address you deposited to

Remember that internal L2 transfers are always fully privacy protected and do NOT require privacy set considerations!
