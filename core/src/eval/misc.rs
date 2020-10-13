use core::cmp::min;

use primitive_types::{H256, U256};

use crate::{ExitError, ExitFatal, ExitRevert, ExitSucceed, Machine};

use super::Control;

pub fn codesize(state: &mut Machine) -> Control {
	let size = U256::from(state.code.len());
	trace_op!("CodeSize: {}", size);
	push_u256!(state, size);
	Control::Continue(1)
}

pub fn codecopy(state: &mut Machine) -> Control {
	pop_u256!(state, memory_offset, code_offset, len);
	trace_op!("CodeCopy: {}", len);

	try_or_fail!(state.memory.resize_offset(memory_offset, len));
	match state.memory.copy_large(memory_offset, code_offset, len, &state.code) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn calldataload(state: &mut Machine) -> Control {
	pop_u256!(state, index);
	trace_op!("CallDataLoad: {}", index);

	let mut load = [0u8; 32];
	for i in 0..32 {
		if let Some(p) = index.checked_add(U256::from(i)) {
			if p <= U256::from(usize::max_value()) {
				let p = p.as_usize();
				if p < state.data.len() {
					load[i] = state.data[p];
				}
			}
		}
	}

	push!(state, H256::from(load));
	Control::Continue(1)
}

pub fn calldatasize(state: &mut Machine) -> Control {
	let len = U256::from(state.data.len());
	trace_op!("CallDataSize: {}", len);
	push_u256!(state, len);
	Control::Continue(1)
}

pub fn calldatacopy(state: &mut Machine) -> Control {
	pop_u256!(state, memory_offset, data_offset, len);
	trace_op!("CallDataCopy: {}", len);

	try_or_fail!(state.memory.resize_offset(memory_offset, len));
	if len == U256::zero() {
		return Control::Continue(1)
	}

	match state.memory.copy_large(memory_offset, data_offset, len, &state.data) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn pop(state: &mut Machine) -> Control {
	pop!(state, val);
	trace_op!("Pop  [@{}]: {}", state.stack.len(), val);
	Control::Continue(1)
}

pub fn mload(state: &mut Machine) -> Control {
	pop_u256!(state, index);
	trace_op!("MLoad: {}", index);
	try_or_fail!(state.memory.resize_offset(index, U256::from(32)));
	let index = as_usize_or_fail!(index);
	let value = H256::from_slice(&state.memory.get(index, 32)[..]);
	push!(state, value);
	Control::Continue(1)
}

pub fn mstore(state: &mut Machine) -> Control {
	pop_u256!(state, index);
	pop!(state, value);
	trace_op!("MStore: {}, {}", index, value);
	try_or_fail!(state.memory.resize_offset(index, U256::from(32)));
	let index = as_usize_or_fail!(index);
	match state.memory.set(index, &value[..], Some(32)) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn mstore8(state: &mut Machine) -> Control {
	pop_u256!(state, index, value);
	trace_op!("MStore8: {}, {}", index, value);
	try_or_fail!(state.memory.resize_offset(index, U256::one()));
	let index = as_usize_or_fail!(index);
	let value = (value.low_u32() & 0xff) as u8;
	match state.memory.set(index, &[value], Some(1)) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn jump(state: &mut Machine) -> Control {
	pop_u256!(state, dest);
	let dest = as_usize_or_fail!(dest, ExitError::InvalidJump);
	trace_op!("Jump: {}", dest);

	if state.valids.is_valid(dest) {
		Control::Jump(dest)
	} else {
		Control::Exit(ExitError::InvalidJump.into())
	}
}

pub fn jumpi(state: &mut Machine) -> Control {
	pop_u256!(state, dest, value);
	let dest = as_usize_or_fail!(dest, ExitError::InvalidJump);

	if value != U256::zero() {
		trace_op!("JumpI: {}", dest);
		if state.valids.is_valid(dest) {
			Control::Jump(dest)
		} else {
			Control::Exit(ExitError::InvalidJump.into())
		}
	} else {
		trace_op!("JumpI: skipped");
		Control::Continue(1)
	}
}

pub fn pc(state: &mut Machine, position: usize) -> Control {
	trace_op!("PC");
	push_u256!(state, U256::from(position));
	Control::Continue(1)
}

pub fn msize(state: &mut Machine) -> Control {
	trace_op!("MSize");
	push_u256!(state, U256::from(state.memory.effective_len()));
	Control::Continue(1)
}

pub fn push(state: &mut Machine, n: usize, position: usize) -> Control {
	let end = min(position + 1 + n, state.code.len());
	let val = U256::from(&state.code[(position + 1)..end]);

	push_u256!(state, val);
	trace_op!("Push [@{}]: {}", state.stack.len() - 1, val);
	Control::Continue(1 + n)
}

pub fn dup(state: &mut Machine, n: usize) -> Control {
	let value = match state.stack.peek(n - 1) {
		Ok(value) => value,
		Err(e) => return Control::Exit(e.into()),
	};
	trace_op!("Dup{} [@{}]: {}", n, state.stack.len(), value);
	push!(state, value);
	Control::Continue(1)
}

pub fn swap(state: &mut Machine, n: usize) -> Control {
	let val1 = match state.stack.peek(0) {
		Ok(value) => value,
		Err(e) => return Control::Exit(e.into()),
	};
	let val2 = match state.stack.peek(n) {
		Ok(value) => value,
		Err(e) => return Control::Exit(e.into()),
	};
	match state.stack.set(0, val2) {
		Ok(()) => (),
		Err(e) => return Control::Exit(e.into()),
	}
	match state.stack.set(n, val1) {
		Ok(()) => (),
		Err(e) => return Control::Exit(e.into()),
	}
	trace_op!("Swap [@0:@{}]: {}, {}", n, val1, val2);
	Control::Continue(1)
}

pub fn ret(state: &mut Machine) -> Control {
	trace_op!("Return");
	pop_u256!(state, start, len);
	try_or_fail!(state.memory.resize_offset(start, len));
	state.return_range = start..(start + len);
	Control::Exit(ExitSucceed::Returned.into())
}

pub fn revert(state: &mut Machine) -> Control {
	trace_op!("Revert");
	pop_u256!(state, start, len);
	try_or_fail!(state.memory.resize_offset(start, len));
	state.return_range = start..(start + len);
	log::trace!("Revert: {}", hex::encode(state.memory.get(start.as_usize(), len.as_usize())));
	Control::Exit(ExitRevert::Reverted.into())
}
