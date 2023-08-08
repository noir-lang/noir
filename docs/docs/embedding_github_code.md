---
title: Embedding Github Code
---

# Copied directly from the local codebase, when building the docs:

Here's an example of some code:

<!-- #include_code storage /yarn-project/noir-contracts/src/contracts/private_token_contract/src/storage.nr rust -->

<!-- #include_code set /yarn-project/noir-contracts/src/contracts/private_token_contract/src/storage.nr rust -->

<!-- #include_code potato /yarn-project/noir-contracts/src/contracts/private_token_contract/src/storage.nr rust -->

 
# Fetched from github, when loading the page:

Here's an example of embedding code from a file of a branch of a github repo:

import GithubCode from '../src/components/GithubCode';


<GithubCode owner="AztecProtocol" language="rust" repo="aztec-packages" branch="master" filePath="yarn-project/noir-contracts/src/contracts/private_token_contract/src/main.nr" startLine={2} endLine={30000} />

<GithubCode owner="AztecProtocol" repo="aztec-packages" branch="master" filePath="README.md" startLine={2} endLine={20} />

<GithubCode owner="AztecProtocol" repo="aztec-packages" branch="master" filePath="README.md"/>




