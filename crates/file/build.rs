use std::env;

fn main() {
    #[cfg(feature = "auto")]
    select_file_picker_feature();
}

#[allow(unused)]
fn select_file_picker_feature() {
    // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts
    let file_picker_feature = match env::var("CARGO_CFG_TARGET_FAMILY") {
        Ok(family) if family == "unix" => "mmap",
        // family can be `windows` or `wasm`
        _ => "fs",
    };

    println!(r#"cargo:rustc-cfg=feature="{file_picker_feature}""#);
}
