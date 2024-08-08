---
id: index
sidebar_position: 0
title: Tutorials and Examples
---

# Code-Along Tutorials and Examples

In this section, you will find two things: code-along tutorials and code examples of Aztec applications. 

Tutorials will teach you how to build a full application or smart contract locally. Examples are not intended for you to replicate locally as they have more complex setups, but can be useful for exploring what you can do with Aztec.

This page includes the most popular tutorials in order of increasing complexity. Explore the sidebar for more!

## Code-Along Tutorials

### Beginner: Write your first smart contract

<div className="card-container full-width">
  <Card shadow='tl' link='/guides/developer_guides/getting_started/quickstart'>
    <CardHeader>
      <h3>Simple counter contract</h3>
    </CardHeader>
    <CardBody>
     Follow this tutorial to build, compile and deploy your first Aztec smart contract - a simple private counter 
    </CardBody>
  </Card>
</div>


### Intermediate: Write increasingly more complex contracts

It is recommended to follow these in order.

<div className="card-container">
  <Card shadow='tl' link='/tutorials/codealong/contract_tutorials/counter_contract'>
    <CardHeader>
      <h3>Simple private voting contract</h3>
    </CardHeader>
    <CardBody>
      Build a contract with hybrid state and calling public functions from private
    </CardBody>
  </Card>

  <Card shadow='tl' link='/tutorials/codealong/contract_tutorials/crowdfunding_contract'>
    <CardHeader>
      <h3>Crowdfunding contract</h3>
    </CardHeader>
    <CardBody>
      A more complex contract that interacts with other contracts
    </CardBody>
  </Card>

  <Card shadow='tl' link='/tutorials/codealong/contract_tutorials/token_contract'>
    <CardHeader>
      <h3>Token contract with hybrid state</h3>
    </CardHeader>
    <CardBody>
      A very complex contract for a token that can move across public & private state and be transferred to others
    </CardBody>
  </Card>

   <Card shadow='tl' link='/tutorials/codealong/contract_tutorials/write_accounts_contract'>
    <CardHeader>
      <h3>Accounts contract</h3>
    </CardHeader>
    <CardBody>
      A simple accounts contract that will teach you about account abstraction in Aztec
    </CardBody>
  </Card>
</div>

<div className="view-all-link">
  <a href="/tutorials/codealong/contract_tutorials/counter_contract">View all smart contract tutorials</a>
</div>

## Examples

<div className="card-container">
  <Card shadow='tl' link='/tutorials/examples/uniswap/l2_contract'>
    <CardHeader>
      <h3>Interacting with L1 Uniswap from L2 Aztec</h3>
    </CardHeader>
    <CardBody>
      An example app inspired by Aztec Connect that allows users to swap publicly & privately on L1 Uniswap from Aztec
    </CardBody>
  </Card>

<Card shadow='tl' link='https://github.com/AztecProtocol/aztec-packages/tree/master/noir-projects/noir-contracts/contracts/card_game_contract' style={{ position: 'relative', overflow: 'hidden' }}>
  <svg 
    viewBox="0 0 24 24" 
    xmlns="http://www.w3.org/2000/svg" 
    style={{
      position: 'absolute',
      top: '-10px',
      right: '-10px',
      width: '80px',
      height: '80px',
      opacity: 0.1,
      zIndex: 0
    }}
  >
    <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" fill="currentColor" />
  </svg>
  <CardHeader style={{ position: 'relative', zIndex: 1 }}>
    <h3>Card game contract <span style={{ marginLeft: '5px', fontSize: '0.8em' }}>↗️</span></h3>
  </CardHeader>
  <CardBody style={{ position: 'relative', zIndex: 1 }}>
    A set of contracts that allow players to take turns playing cards 
  </CardBody>
</Card>
</div>
