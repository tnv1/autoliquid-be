# Autonomous Liquidity Management Backend

## Overview

AutoLiquid is an autonomous liquidity management system for the Sui blockchain. This backend service monitors liquidity positions and automatically adjusts them based on market conditions to optimize returns.

## Features

- Automated position management for Bluefin DEX
- Position monitoring and price tracking
- Automatic repositioning based on price thresholds
- User position querying and management
- Secure key storage for position owners

## Architecture

The system consists of the following components:

1. **REST API Service**
   - Provides HTTP endpoints for frontend applications
   - Handles user requests for position data, pool information, etc.
   - Communicates with the database and Sui blockchain

2. **Indexer**
   - Syncs data from the Sui blockchain
   - Processes transactions and updates position information
   - Tracks progress using the progress_store table

3. **PostgreSQL Database**
   - Stores all application data
   - Main tables:
     - progress_store: Tracks sync progress
     - sui_error_transactions: Logs failed transactions
     - position_updates: Stores liquidity position information

4. **Cache Service**
   - Provides fast access to frequently requested data
   - Reduces load on the database

5. **Sui Blockchain**
   - External blockchain system
   - Source of liquidity position data

6. **Reposition Manager**
   - Implements auto-repositioning liquidity logic
   - Monitors positions and market conditions
   - Executes rebalancing transactions when needed
   - Maintains optimal liquidity ranges

7. **Price Oracle**
   - Monitors current market prices
   - Provides price feeds for token pairs
   - Supports decision-making for repositioning logic

8. **Signer Storage**
    - Secure signers storage for user keys

## Development

1. Clone the repository
2. Copy `.env.example` to `.env` and configure the parameters
3. Set up the database with docker

    ```bash
    make db-up
    ```

4. Run indexer:

    ```bash
    cargo run --bin indexer
    ```

5. Run app service

    ```bash
    cargo run --bin app
    ```

6. Cleanup db

    ```bash
    make db-down
    ```

## Documents

- [System Architecture](./docs/architecture_diagram.md) - Detailed overview of system components and their interactions
