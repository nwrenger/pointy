use pointy_api::extension_entry;

// Define here your entry function
extension_entry!(main);

// Add now here your extension code
// Use `pointy_api` for bundled dependencies and helper functions for the clipboard
fn main() -> Result<(), String> {
    // Some sample print
    println!("Hello World!");

    Ok(())
}
