use std::process::Command;

fn main() {
    let hash_utf8 = Command::new("git").args(["rev-parse", "--short", "HEAD"]).output();
    if let Ok(hash_utf8) = hash_utf8 {
        let hash = String::from_utf8(hash_utf8.stdout).unwrap();
        println!("cargo:rustc-env=COMMIT_HASH={}", hash.trim());
    } else {
        println!("cargo:rustc-env=COMMIT_HASH=NO_GIT");
    }
}
