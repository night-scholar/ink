#![cfg_attr(not(all(test, feature = "test-env")), no_std)]

use parity_codec::{Decode};
use pdsl_core::{
	env::{Env, ContractEnv},
	storage::{
		alloc,
		Key,
		Value,
		Flush,
		alloc::{
			AllocateUsing,
			Initialize,
		},
	},
};

// #[cfg(not(test))]
// use pdsl_core::println;

/// An incrementer smart contract.
///
/// Can only increment and return its current value.
struct Incrementer {
	/// The current value stored in the storage.
	current: Value<u32>,
}

impl Incrementer {
	/// Increments the current value.
	pub fn inc(&mut self, by: u32) {
		self.current += by;
	}

	/// Returns the current value.
	pub fn get(&self) -> u32 {
		*self.current
	}
}

impl Initialize for Incrementer {
	type Args = ();

	fn initialize(&mut self, _args: Self::Args) {
		self.current.set(0)
	}
}

// Everything below this point can be generated by the upcoming eDSL.

impl AllocateUsing for Incrementer {
	unsafe fn allocate_using<A>(alloc: &mut A) -> Self
	where
		A: pdsl_core::storage::Allocator,
	{
		Self {
			current: AllocateUsing::allocate_using(alloc),
		}
	}
}

impl Flush for Incrementer {
	fn flush(&mut self) {
		self.current.flush()
	}
}

#[derive(parity_codec::Encode, parity_codec::Decode)]
enum Action {
	Get,
	Inc(u32),
}

/// The allocation key for the bump allocator.
const ALLOC_KEY: Key = Key([0x0; 32]);

fn ret<T>(val: T) -> !
where
	T: parity_codec::Encode,
{
	ContractEnv::return_(&val.encode())
}

fn instantiate() -> Incrementer {
	unsafe {
		let mut alloc = alloc::BumpAlloc::from_raw_parts(ALLOC_KEY);
		Incrementer::allocate_using(&mut alloc)
	}
}

#[no_mangle]
pub extern "C" fn deploy() {
	instantiate().initialize_into(()).flush()
}

#[no_mangle]
pub extern "C" fn call() {
	let input = ContractEnv::input();
	let action = Action::decode(&mut &input[..]).unwrap();
	let mut incrementer = instantiate();

	match action {
		Action::Get => {
			let returned_val = &incrementer.get();
			// println!("CALL: identified get() and returned {:?}", returned_val);
			ret(&returned_val)
		}
		Action::Inc(by) => {
			// println!("CALL: identified inc({:?})", by);
			incrementer.inc(by);
			incrementer.flush();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_get() {
		let incrementer = unsafe {
			let mut alloc = alloc::BumpAlloc::from_raw_parts(ALLOC_KEY);
			Incrementer::allocate_using(&mut alloc).initialize(())
		};
		assert_eq!(incrementer.get(), 0)
	}

	#[test]
	fn test_set() {
		let mut incrementer = unsafe {
			let mut alloc = alloc::BumpAlloc::from_raw_parts(ALLOC_KEY);
			Incrementer::allocate_using(&mut alloc).initialize(())
		};
		incrementer.inc(1);
		assert_eq!(incrementer.get(), 1);
		incrementer.inc(42);
		assert_eq!(incrementer.get(), 43);
	}
}
