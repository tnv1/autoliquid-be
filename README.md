# Autonomous Liquidity Management Backend

> **Note:** This project is currently under active development. Features and documentation may change as the project evolves.

## Overview

AutoLiquid is an autonomous liquidity management system for the Sui blockchain. This backend service monitors liquidity positions and automatically adjusts them based on market conditions to optimize returns.

## Features

- Automated position management for Bluefin DEX
- Position monitoring and price tracking
- Automatic repositioning based on price thresholds
- User position querying and management
- Secure key storage for position owners

## Architecture

- [System Architecture](./docs/architecture_diagram.md) - Detailed overview of system components and their interactions

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
    make db-clean
    ```
