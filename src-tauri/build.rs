// build.rs - Custom build script for OpenCode Tauri
// This can be used for code generation, asset embedding, etc.

fn main() {
    // In a real implementation, we might:
    // 1. Embed assets (icons, etc.)
    // 2. Generate code
    // 3. Set up build-time configuration

    println!("Building OpenCode Tauri Desktop...");

    // Re-export tauri-build's main function
    tauri_build::build()
}
