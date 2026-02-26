// Build script for switchboard
// This script helps distinguish between cargo test and cargo nextest at compile time

use std::env;

fn main() {
    // Check if we're compiling for tests
    if env::var("CARGO_CFG_TEST").is_ok() {
        // Set a cfg flag that indicates we're in test mode
        println!("cargo:rustc-cfg=switchboard_test");
    }
}
