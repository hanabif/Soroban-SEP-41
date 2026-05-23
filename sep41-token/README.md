# Soroban-SEP-41 Token

A production-grade implementation of the [SEP-41](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md) token standard on the Stellar network using Soroban. This project focuses on high gas efficiency, automated lifecycle management, and administrative security.

## Features

- **Standard Compliance**: Full implementation of the SEP-41 interface (`transfer`, `approve`, `burn`, etc.).
- **Access Control**: Integrated administrator role for minting and contract management.
- **Emergency Pause**: Global circuit breaker to halt token movements in case of security incidents.
- **Batch Transfers**: Optimized multi-recipient transfer function to save gas on batch operations.
- **Storage Lifecycle Management**: Automated TTL (Time To Live) bumping for both instance and persistent storage to prevent data archiving.

## Design Decisions

### 1. Unified Storage Access
I implemented internal helpers (`read_balance`, `write_balance`, etc.) that consolidate storage logic. Every read/write operation automatically invokes `extend_ttl`. This ensures that active user balances and allowances are never archived by the network, providing a seamless user experience.

### 2. Gas Optimization via Storage Modes
- **Instance Storage**: Used for consensus-wide metadata (`admin`, `decimals`, `is_paused`). Since these are accessed in almost every transaction, instance storage minimizes read costs.
- **Persistent Storage**: Used for user-specific data (`balance`, `allowance`) to ensure long-term retention beyond the instance lifetime.

### 3. Safety First
Instead of simple `unwrap()` calls, the contract uses a structured `ContractError` enum. Every administrative action is gated by `require_auth()` and checked against a global `paused` state.

## Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/)
- [Soroban CLI](https://soroban.stellar.org/docs/getting-started/setup#install-the-soroban-cli)

### Installation & Testing

1. Clone the repository and navigate to the project folder:
   ```bash
   cd sep41-token
   ```

2. Run the test suite to verify implementation:
   ```bash
   cargo test
   ```

3. Build the contract:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

## Contract Interface Highlights

- `initialize(admin: Address, decimals: u32, name: String, symbol: String)`: One-time setup.
- `batch_transfer(from: Address, recipients: Vec<(Address, i128)>)`: Send to many in one TX.
- `set_pause(paused: bool)`: Emergency lock (Admin only).
- `set_admin(new_admin: Address)`: Ownership transfer.
