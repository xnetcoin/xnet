//! # XNET Node — Entry Point
//!
//! Thin binary crate that wires together the CLI, service, and chain-spec modules,
//! then hands control to the command dispatcher. All the real work lives in the
//! sub-modules listed below.

#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;
mod rpc;
mod benchmarking;

fn main() -> sc_cli::Result<()> {
    command::run()
}