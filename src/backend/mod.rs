//! # EVM backends
//!
//! Backends store state information of the VM, and exposes it to runtime.

use alloc::vec::Vec;

use primitive_types::{H160, H256, U256};

pub use self::memory::{MemoryAccount, MemoryBackend, MemoryVicinity};

mod memory;

/// Basic account information.
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct Basic {
	/// Account balance.
	pub balance: U256,
	/// Account nonce.
	pub nonce: U256,
}

/// Log information.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Log {
	/// Source address.
	pub address: H160,
	/// Topics.
	pub topics: Vec<H256>,
	/// Log data.
	pub data: Vec<u8>,
}

/// Apply state operation.
#[derive(Clone, Debug)]
pub enum Apply<I> {
	/// Modify or create at address.
	Modify {
		/// Address.
		address: H160,
		/// Basic information of the address.
		basic: Basic,
		/// Code. `None` means leaving it unchanged.
		code: Option<Vec<u8>>,
		/// Storage iterator.
		storage: I,
		/// Whether storage should be wiped empty before applying the storage
		/// iterator.
		reset_storage: bool,
	},
	/// Delete address.
	Delete {
		/// Address.
		address: H160,
	},
}

/// EVM backend.
#[async_trait::async_trait]
pub trait Backend: Send + Sync + 'static {
	/// Gas price.
	async fn gas_price(&self) -> U256;
	/// Origin.
	async fn origin(&self) -> H160;
	/// Environmental block hash.
	async fn block_hash(&self, number: U256) -> H256;
	/// Environmental block number.
	async fn block_number(&self) -> U256;
	/// Environmental coinbase.
	async fn block_coinbase(&self) -> H160;
	/// Environmental block timestamp.
	async fn block_timestamp(&self) -> U256;
	/// Environmental block difficulty.
	async fn block_difficulty(&self) -> U256;
	/// Environmental block gas limit.
	async fn block_gas_limit(&self) -> U256;
	/// Environmental chain ID.
	async fn chain_id(&self) -> U256;

	/// Whether account at address exists.
	async fn exists(&self, address: H160) -> bool;
	/// Get basic account information.
	async fn basic(&self, address: H160) -> Basic;
	/// Get account code hash.
	async fn code_hash(&self, address: H160) -> H256;
	/// Get account code size.
	async fn code_size(&self, address: H160) -> usize;
	/// Get account code.
	async fn code(&self, address: H160) -> Vec<u8>;
	/// Get storage value of address at index.
	async fn storage(&self, address: H160, index: H256) -> H256;
}

/// EVM backend that can apply changes.
#[async_trait::async_trait]
pub trait ApplyBackend {
	/// Apply given values and logs at backend.
	async fn apply<A, I, L>(
		&mut self,
		values: A,
		logs: L,
		delete_empty: bool,
	) where
		A: Sync + Send + IntoIterator<Item=Apply<I>>,
		I: Sync + Send + IntoIterator<Item=(H256, H256)>,
		L: Sync + Send + IntoIterator<Item=Log>;
}
