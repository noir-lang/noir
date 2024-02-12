Santiago mentioned the ability to automatically deref an AztecAddress into a Field.

I believe I mentioned something like #[transparent]. The issue with this is 
that you lose type safety, though perhaps its fine if you opt-in to it, and its 
only usecase would be to make the part where you input values to the abi smoother.

Not a hard push on this, I just saw a forced usecase for this a minute ago.