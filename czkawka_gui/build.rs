// Force cargo to recompile when UI files change.
// The .ui files are embedded via include_str!() but cargo doesn't track them automatically.
fn main() {
    println!("cargo:rerun-if-changed=ui/");
}
