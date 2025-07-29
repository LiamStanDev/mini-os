use std::fs::{File, read_dir};
use std::io::{Result, Write};

fn main() {
    println!("cargo:rerun-if-changed=../user/src/");
    println!("cargo:rerun-if-changed={TARGET_PATH}");
    insert_app_data().unwrap();
}

static TARGET_PATH: &str = "../user/target/riscv64gc-unknown-none-elf/release/";

fn insert_app_data() -> Result<()> {
    let mut f = File::create("src/link_app.S").unwrap();
    let mut apps: Vec<_> = read_dir("../user/src/bin")
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            // dir_entry is Result because error occur when iterating file e.g. file deleted,
            // or no permission
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    apps.sort();

    writeln!(
        f,
        r#"
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad {}"#,
        apps.len()
    )?;

    for i in 0..apps.len() {
        writeln!(f, r#"    .quad app_{i}_start"#)?;
    }
    writeln!(f, r#"    .quad app_{}_end"#, apps.len() - 1)?;

    writeln!(
        f,
        r#"
    .global _app_names
_app_names:"#
    )?;

    for app in apps.iter() {
        writeln!(f, r#"    .string "{app}""#)?;
    }

    for (idx, app) in apps.iter().enumerate() {
        println!("app_{idx}: {app}");
        writeln!(
            f,
            r#"
    .section .data
    .global app_{idx}_start
    .global app_{idx}_end
    .align 3
app_{idx}_start:
    .incbin "{TARGET_PATH}{app}"
app_{idx}_end:"#,
        )?;
    }
    Ok(())
}
