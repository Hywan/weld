use cfg_if::cfg_if;

fn main() {
    #[cfg(feature = "auto")]
    select_file_picker_feature();
}

#[allow(unused)]
fn select_file_picker_feature() {
    let file_picker_feature = {
        cfg_if! {
            if #[cfg(target_family = "unix")] {
                "mmap"
            } else {
                "fs"
            }
        }
    };

    println!(r#"cargo:rustc-cfg=feature="{file_picker_feature}""#);
}
