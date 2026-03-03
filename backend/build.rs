use std::process::Command;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Try to read short git commit, fall back to 'unknown'
    let git_commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"]) 
        .output()
        .ok()
        .and_then(|o| if o.status.success() { Some(String::from_utf8_lossy(&o.stdout).trim().to_string()) } else { None })
        .unwrap_or_else(|| "unknown".to_string());

    // Build time as unix seconds
    let build_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string());

    println!("cargo:rustc-env=GIT_COMMIT={}", git_commit);
    println!("cargo:rustc-env=BUILD_TIME={}", build_time);

    // Re-run build script when git HEAD changes
    if let Ok(git_dir) = env::var("GIT_DIR") {
        println!("cargo:rerun-if-changed={}", git_dir);
    } else {
        println!("cargo:rerun-if-changed=.git/HEAD");
    }
}
