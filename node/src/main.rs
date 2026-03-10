//! # XNET Node — Entry Point
//!
//! Thin binary crate that wires together the CLI, service, and chain-spec modules,
//! then hands control to the command dispatcher. All the real work lives in the
//! sub-modules listed below.

#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod benchmarking;
mod cli;
mod command;
mod rpc;

#[allow(clippy::result_large_err)]
fn main() -> sc_cli::Result<()> {
	command::run()
}
