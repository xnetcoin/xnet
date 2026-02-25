// Build script for the XNET node binary.
//
// Embeds the Git commit hash and branch name into the binary so that
// `xnet-node --version` reports an accurate source revision.
// The `rerun_if_git_head_changed` call ensures Cargo re-runs this script
// whenever the Git HEAD pointer moves (e.g. after a commit or checkout).

use substrate_build_script_utils::{generate_cargo_keys, rerun_if_git_head_changed};

fn main() {
	// Inject CARGO_PKG_* env-vars and the Git commit hash as compile-time constants.
	generate_cargo_keys();

	// Re-run this build script when HEAD changes so the version string stays accurate.
	rerun_if_git_head_changed();
}
