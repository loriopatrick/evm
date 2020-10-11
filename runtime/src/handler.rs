use alloc::vec::Vec;

use primitive_types::{H160, H256, U256};

use crate::{Capture, Context, CreateScheme, ExitError, ExitReason,
			ExternalOpcode, Machine, Opcode, Stack};

/// Transfer from source to target, with given value.
#[derive(Clone, Debug)]
pub struct Transfer {
	/// Source address.
	pub source: H160,
	/// Target address.
	pub target: H160,
	/// Transfer value.
	pub value: U256,
}

/// EVM context handler.
#[async_trait::async_trait]
pub trait Handler {
	/// Type of `CREATE` interrupt.
	type CreateInterrupt;
	/// Feedback value for `CREATE` interrupt.
	type CreateFeedback;
	/// Type of `CALL` interrupt.
	type CallInterrupt;
	/// Feedback value of `CALL` interrupt.
	type CallFeedback;

	/// Get balance of address.
	async fn balance(&self, address: H160) -> U256;
	/// Get code size of address.
	async fn code_size(&self, address: H160) -> U256;
	/// Get code hash of address.
	async fn code_hash(&self, address: H160) -> H256;
	/// Get code of address.
	async fn code(&self, address: H160) -> Vec<u8>;
	/// Get storage value of address at index.
	async fn storage(&self, address: H160, index: H256) -> H256;
	/// Get original storage value of address at index.
	async fn original_storage(&self, address: H160, index: H256) -> H256;

	/// Get the gas left value.
	fn gas_left(&self) -> U256;
	/// Get the gas price value.
	async fn gas_price(&self) -> U256;
	/// Get execution origin.
	async fn origin(&self) -> H160;
	/// Get environmental block hash.
	async fn block_hash(&self, number: U256) -> H256;
	/// Get environmental block number.
	async fn block_number(&self) -> U256;
	/// Get environmental coinbase.
	async fn block_coinbase(&self) -> H160;
	/// Get environmental block timestamp.
	async fn block_timestamp(&self) -> U256;
	/// Get environmental block difficulty.
	async fn block_difficulty(&self) -> U256;
	/// Get environmental gas limit.
	async fn block_gas_limit(&self) -> U256;
	/// Get environmental chain ID.
	async fn chain_id(&self) -> U256;

	/// Check whether an address exists.
	async fn exists(&self, address: H160) -> bool;
	/// Check whether an address has already been deleted.
	fn deleted(&self, address: H160) -> bool;

	/// Set storage value of address at index.
	async fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<(), ExitError>;
	/// Create a log owned by address with given topics and data.
	fn log(&mut self, address: H160, topcis: Vec<H256>, data: Vec<u8>) -> Result<(), ExitError>;
	/// Mark an address to be deleted, with funds transferred to target.
	async fn mark_delete(&mut self, address: H160, target: H160) -> Result<(), ExitError>;
	/// Invoke a create operation.
	async fn create(
		&mut self,
		caller: H160,
		scheme: CreateScheme,
		value: U256,
		init_code: Vec<u8>,
		target_gas: Option<usize>,
	) -> Capture<(ExitReason, Option<H160>, Vec<u8>), Self::CreateInterrupt>;
	/// Feed in create feedback.
	fn create_feedback(
		&mut self,
		_feedback: Self::CreateFeedback
	) -> Result<(), ExitError> {
		Ok(())
	}
	/// Invoke a call operation.
	async fn call(
		&mut self,
		code_address: H160,
		transfer: Option<Transfer>,
		input: Vec<u8>,
		target_gas: Option<usize>,
		is_static: bool,
		context: Context,
	) -> Capture<(ExitReason, Vec<u8>), Self::CallInterrupt>;
	/// Feed in call feedback.
	fn call_feedback(
		&mut self,
		_feedback: Self::CallFeedback
	) -> Result<(), ExitError> {
		Ok(())
	}

	/// Pre-validation step for the runtime.
	async fn pre_validate(
		&mut self,
		context: &Context,
		opcode: Result<Opcode, ExternalOpcode>,
		stack: &Stack
	) -> Result<(), ExitError>;
	/// Handle other unknown xternal opcodes.
	fn other(
		&mut self,
		_opcode: u8,
		_stack: &mut Machine
	) -> Result<(), ExitError> {
		Err(ExitError::OutOfGas)
	}
}
