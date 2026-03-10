// Build script for the XNET runtime crate.
//
// Compiles the runtime Rust source into a WebAssembly blob that is embedded in
// the node binary. The WASM blob is what actually executes on-chain; the native
// version is used only for faster local development.
//
// Flags:
//   export_heap_base — tells the WASM linker where the heap starts.
//   import_memory    — the host (node) provides and manages WASM linear memory.

fn main() {
	substrate_wasm_builder::WasmBuilder::new()
		.with_current_project()
		// Required for Substrate's WASM ABI: the host supplies the memory region.
		.export_heap_base()
		.import_memory()
		.build()
}
