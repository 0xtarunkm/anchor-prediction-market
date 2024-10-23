# Cricket Prediction Market

This repository contains a Cricket Prediction Market contract written in [Anchor](https://project-serum.github.io/anchor/) for the Solana blockchain. The contract allows users to participate in a decentralized prediction market by placing bets on the outcomes of cricket matches.

## Features

- **Prediction Markets**: Users can place predictions (bets) on cricket match outcomes.
- **Cricket Match Integration**: Results are based on real cricket matches, fetched from external APIs or oracles.
- **Decentralized**: The smart contract runs on the Solana blockchain, ensuring transparency and fairness.
- **Anchor Framework**: Built using the Anchor framework for Rust-based Solana programs.
- **Tokenized Bets**: Bets are represented as tokens that can be traded or redeemed based on the outcome.
- **Automated Settlement**: Upon match completion, winnings are distributed automatically based on the match result.

## Table of Contents

- [Cricket Prediction Market](#cricket-prediction-market)
  - [Features](#features)
  - [Table of Contents](#table-of-contents)
  - [Installation](#installation)
  - [Usage](#usage)
    - [Prediction Market Curve](#prediction-market-curve)
    - [Web Interface](#web-interface)

## Installation

To install and build the project locally:

1. Clone the repository:

   ```bash
   git clone https://github.com/0xtarunkm/anchor-prediction-market.git
   cd anchor-prediction-market
   ```

2. Install dependencies:

   ```bash
   yarn install
   ```

3. Build the Anchor program:

   ```bash
   anchor build
   ```

4. Deploy the program to Solana Localnet:

   ```bash
   anchor deploy
   ```

## Usage

Once deployed, users can interact with the contract via CLI or through a web interface (if implemented). The contract supports the following operations:

1. **Create a Market**: Create a new prediction market for a specific cricket match.
2. **Place a Bet**: Users place their bets on possible outcomes (win/loss/draw).
3. **Close the Market**: After the match is concluded, the market is settled based on the actual result.
4. **Claim Winnings**: Users with successful predictions can claim their rewards.
