use std::process::Command;

fn main() {
    // Capture git commit hash at build time
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output();

    let commit = match output {
        Ok(o) if o.status.success() => {
            String::from_utf8(o.stdout).unwrap_or_default().trim().to_string()
        }
        _ => "unknown".to_string(),
    };

    println!("cargo:rustc-env=GIT_COMMIT={commit}");
}
