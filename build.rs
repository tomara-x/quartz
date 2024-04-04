use std::process::Command;

fn main() {
    let hash = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output().unwrap().stdout
    ).unwrap();
    println!("cargo:rustc-env=COMMIT_HASH={}", hash.trim());
}
