# rust-eth-playground

A from-scratch Rust implementation of Ethereum protocol primitives. The goal is to internalise every significant pattern used in [Reth](https://github.com/paradigmxyz/reth), [Revm](https://github.com/bluealloy/revm), [Lighthouse](https://github.com/sigp/lighthouse), and [Alloy](https://github.com/alloy-rs/core) by building miniature versions of them.

**Rule:** every function returns `Result`. No `.unwrap()` outside tests.

## Workspace layout

```
rust-eth-playground/
├── types/        # Newtypes, Transaction enum, errors
├── rlp-codec/    # RLP encode/decode, signing, Merkle Patricia Trie
├── networking/   # Async TCP p2p with tokio + mpsc/broadcast
└── execution/    # Provider traits, executor, validator, pipeline
```

The crates form a layered dependency graph: `types` → `rlp-codec` → `execution`, with `networking` depending on `types` and `rlp-codec`.

## What's inside

### `types` — core domain types
- `Address` (20 bytes), `B256` (32 bytes), `Bloom` (256 bytes) as newtypes with manually implemented `Display`, `LowerHex`, `UpperHex`, `From`, `AsRef`, `TryFrom<&[u8]>`, `FromStr`, `Hash`, `Default`.
- `Transaction` enum with `Legacy`, `Eip1559`, and `Eip4844` variants, plus `AccessListItem`.
- Helper methods: `effective_gas_price`, `max_cost`, `tx_type`, `is_create`, all using checked arithmetic.
- `DecodeError` and `TransactionError` built with `thiserror`, including `#[from]` conversions.

### `rlp-codec` — RLP, signing, and tries
- `RlpItem` enum with hand-written encoder and decoder covering single bytes, short/long strings, and short/long lists.
- `RlpEncodable` and `RlpDecodable` traits implemented for `u64`, `u128`, `bool`, `Vec<u8>`, `Address`, `B256`, and `Transaction`.
- Roundtrip tests over deeply-nested and edge-case inputs.
- **EIP-155 transaction signing** (`signing.rs`): keccak256 helper, EIP-155 legacy payload, EIP-2718 typed envelopes for EIP-1559 / EIP-4844, ECDSA signing via `k256`, sender recovery, `SignedTransaction::hash()` over the wire format. `SigningError` propagates RLP and ECDSA failures via `#[from]`.
- **Merkle Patricia Trie** (`trie.rs`): leaf / extension / branch nodes, nibble-walking insert with path-splitting, `root_hash` that inlines nodes < 32 bytes and hashes larger ones.

### `networking` — async p2p
- `tokio_util::codec::{Encoder, Decoder}` implementation using a 4-byte big-endian length prefix, 1-byte type tag, and RLP-encoded payload, with partial-read handling.
- Message enum: `Ping`, `Pong`, `Status`, `Transactions`, `GetBlockHeaders`.
- Three-layer architecture: TCP listener → per-connection tasks → central manager task communicating over `mpsc` channels.
- `tokio::select!` in the manager driving peer messages, a 10-second ping interval, and a `broadcast`-channel shutdown wired to ctrl-c.
- Shared chain state behind `Arc<RwLock<...>>`.
- `JoinSet`-based graceful shutdown.

### `execution` — provider traits and pipeline
- Five provider traits (`BlockProvider`, `HeaderProvider`, `StateProvider`, `TransactionProvider`, `ReceiptProvider`) and a `FullProvider` supertrait with a blanket impl, mirroring Reth's split.
- `InMemoryProvider` backed by `HashMap`s implementing all five traits.
- `CachedProvider<T>` — generic cache wrapper exercising trait bounds, forwarding, and interior-mutability decisions.
- `BlockExecutor` trait with associated output type, `ConsensusValidator` trait, and a `Pipeline` generic over four trait-bounded type parameters.
- **Production-faithful sender handling**: blocks store `Vec<SignedTransaction>`; senders are recovered into a `BlockWithSenders { block, senders }` wrapper before execution, mirroring Reth's pattern (rather than caching `from` on the transaction the way geth does).
- Receipts use the real `SignedTransaction::hash()` rather than a placeholder.

## Building and testing

```sh
cargo build --workspace
cargo test --workspace

# Run the networking demo binary
cargo run -p networking
```
