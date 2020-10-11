use crate::{CallScheme, ExitReason, ExternalOpcode, Handler, Runtime};

#[macro_use]
mod macros;
mod system;

pub enum Control<H: Handler> {
	Continue,
	CallInterrupt(H::CallInterrupt),
	CreateInterrupt(H::CreateInterrupt),
	Exit(ExitReason)
}

pub async fn eval<H: Handler>(state: &mut Runtime, opcode: ExternalOpcode, handler: &mut H) -> Control<H> {
	match opcode {
		ExternalOpcode::Sha3 => system::sha3(state),
		ExternalOpcode::Address => system::address(state),
		ExternalOpcode::Balance => system::balance(state, handler).await,
		ExternalOpcode::SelfBalance => system::selfbalance(state, handler).await,
		ExternalOpcode::Origin => system::origin(state, handler).await,
		ExternalOpcode::Caller => system::caller(state),
		ExternalOpcode::CallValue => system::callvalue(state),
		ExternalOpcode::GasPrice => system::gasprice(state, handler).await,
		ExternalOpcode::ExtCodeSize => system::extcodesize(state, handler).await,
		ExternalOpcode::ExtCodeHash => system::extcodehash(state, handler).await,
		ExternalOpcode::ExtCodeCopy => system::extcodecopy(state, handler).await,
		ExternalOpcode::ReturnDataSize => system::returndatasize(state),
		ExternalOpcode::ReturnDataCopy => system::returndatacopy(state),
		ExternalOpcode::BlockHash => system::blockhash(state, handler).await,
		ExternalOpcode::Coinbase => system::coinbase(state, handler).await,
		ExternalOpcode::Timestamp => system::timestamp(state, handler).await,
		ExternalOpcode::Number => system::number(state, handler).await,
		ExternalOpcode::Difficulty => system::difficulty(state, handler).await,
		ExternalOpcode::GasLimit => system::gaslimit(state, handler).await,
		ExternalOpcode::SLoad => system::sload(state, handler).await,
		ExternalOpcode::SStore => system::sstore(state, handler).await,
		ExternalOpcode::Gas => system::gas(state, handler),
		ExternalOpcode::Log(n) => system::log(state, n, handler),
		ExternalOpcode::Suicide => system::suicide(state, handler).await,
		ExternalOpcode::Create => system::create(state, false, handler).await,
		ExternalOpcode::Create2 => system::create(state, true, handler).await,
		ExternalOpcode::Call => system::call(state, CallScheme::Call, handler).await,
		ExternalOpcode::CallCode => system::call(state, CallScheme::CallCode, handler).await,
		ExternalOpcode::DelegateCall => system::call(state, CallScheme::DelegateCall, handler).await,
		ExternalOpcode::StaticCall => system::call(state, CallScheme::StaticCall, handler).await,
		ExternalOpcode::ChainId => system::chainid(state, handler).await,
		ExternalOpcode::Other(opcode) => {
			match handler.other(
				opcode,
				&mut state.machine
			) {
				Ok(()) => Control::Continue,
				Err(e) => Control::Exit(e.into()),
			}
		},
	}
}
