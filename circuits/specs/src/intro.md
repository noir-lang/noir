# About

- This spec is very much a work in progress. It will change many times between now and deployment.
- Much of this document is to aid development and hence is _very_ detailed, so that things which have been thought about aren't forgotten. The detail might make some sections unreadable (e.g. execution logic bullet points).
- It's such a big document, that there will be mistakes - please ask & we can correct any you see :)
- All constants in this spec are somewhat arbitrarily chosen, as the architecture must be implemented and performance measured to determine optimal values.
- Stuff relating to fees and attributing 'work done' by provers in the form of 'proverRecords' is only half thought-out. It needs more work, so skim read it for now.
# Goals of Aztec 3

- Private L2
  - General-purpose, private smart contracts.
- Public L2
  - Cheaper public state transactions than interacting with L1.
- Interacting with Ethereum's L1 still supported (and required for many use cases).

> Whenever we refer to 'contract' in this document, we'll be referring to an Aztec 3 L2 contract (which may comprise both public and private functions). If we're talking about an Ethereum contract, we'll explicitly say something like 'Ethereum contract' or 'L1 contracct' or 'Portal contract'.


Here are the beginnings of some [sexy diagrams](https://drive.google.com/file/d/1gCFhE78QhfEboF0hq3scb4vAU1pE0emH/view?usp=sharing) (open with -> diagrams.net)
