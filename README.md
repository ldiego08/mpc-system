# Transactions MPC System

## Overview

This system is a Multi-Party Computation (MPC) system POC in Rust, involving multiple nodes processing computations locally, verifying results through a collective threshold signature scheme, and ensuring data consistency across the network using HTTP for communication.

> HTTP has been chosen for this MVP as it simplifies the implementation while still being secure enough.

## Components

- **Node:** An individual participant in the network. Each node processes transactions, manages wallets, and registers with peers.

- **Wallet:** A user account with a unique ID and balance.

- **Transaction:** A transfer of funds from one wallet to another.

- **Transaction verification:** A request sent to peers for transaction verification, including the computation result and a signature share.

- **Peer registration:** Register a node with its peers.

- **Wallet creation:** A request to create a new wallet with an initial balance.

## Interaction and Data Flow

### Node Initialization

Each node is initialized with a unique ID, address, secret key, public key, and shared stores for computation results, signature shares, and wallets.

### Peer Registration

Nodes register with each other by calling the `/register` endpoint. Each node stores the public key and address of its peers in a shared peers store.

### Wallet Creation

A node creates a wallet by calling the `/wallet_creation` endpoint.

The node generates a unique wallet ID, stores the wallet, and broadcasts the creation to all peers.

Peers receive the wallet creation message, store the new wallet, and ensure consistency across the network.

### Transactions

A node initiates a transaction by calling the /transaction endpoint.

The node processes the transaction by verifying the signature and ensuring sufficient balance in the sender's wallet. If successful, the transaction is broadcasted to all peers for consistency.

### Verification

After processing a transaction, the node sends a verification request to the /verification endpoint. Peers receive the request, verify the signature share, and store the result if valid.

## Endpoints

- **POST /register:** Handles peer registration.
- **POST /transaction:** Processes incoming transactions.
- **POST /wallet_creation:** Handles wallet creation requests.
- **POST /verification:** Handles incoming verification requests.
- **GET /peers:** Returns the list of registered peers.

## Security Features

### Public Key Infrastructure

Each node is identified by a unique public key, and transactions and verification requests are signed using the node's private key.

Nodes verify signatures using the sender's public key, ensuring the authenticity of messages.

### Threshold Signature Scheme

The system uses a threshold signature scheme to ensure that transactions are validated collectively in a consensus of nodes. A transaction or computation result is considered valid only if a sufficient number of nodes (meeting the threshold) provide valid signature shares.

### Encryption

Communication between nodes uses HTTPS to ensure data is encrypted in transit, preventing eavesdropping and man-in-the-middle attacks.

### Data Consistency

Nodes broadcast wallet creation and transaction messages to all peers, ensuring that all nodes maintain a consistent state.

Each node processes and verifies incoming messages independently, mitigating the risk of a single point of failure or malicious node disrupting the network.

### Secure Storage

Each node stores its private key securely, ensuring that it cannot be accessed or tampered with by unauthorized parties.

## Component Interaction

### Node Initialization and Peer Registration

- When a node starts, it reads its configuration (node ID, address, initial peers) and initializes its key pair and data stores.

- It then registers with peers. Each listening peer stores the sender's public key and address.

### Wallet Creation and Broadcasting

- When a wallet creation request is received, the node creates a new wallet with a unique ID and initial balance, then stores it in the wallets store.

- The node broadcasts the wallet creation to all peers to ensure data onsistency across the network.

### Transaction Processing and Verification

- When a transaction is received, the node verifies the signature and checks the sender's balance.

- If the transaction is valid, the node updates the wallets store and broadcasts the transaction to peers.

- The node then sends a verification request to peers for collective verification.

By following this design, the system achieves decentralized transaction processing, robust verification through a threshold signature scheme, and secure, consistent state management across the network. This ensures both privacy and integrity in a decentralized environment.
