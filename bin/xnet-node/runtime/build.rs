fn main() {
	#[cfg(feature = "std")]
	{
		xnet_wasm_builder::WasmBuilder::new()
			.with_current_project()
			.export_heap_base()
			.import_memory()
			.build();
	}
}
