use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    set_git_commit();
    set_build_timestamp();
}

fn set_git_commit() {
    if let Ok(output) = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        if output.status.success() {
            if let Ok(hash) = String::from_utf8(output.stdout) {
                let trimmed = hash.trim();
                if !trimmed.is_empty() {
                    println!("cargo:rustc-env=OBSCTL_GIT_COMMIT={trimmed}");
                }
            }
        }
    }
}

fn set_build_timestamp() {
    let now = time::OffsetDateTime::now_utc();
    if let Ok(formatted) = now.format(&time::format_description::well_known::Rfc3339) {
        println!("cargo:rustc-env=OBSCTL_BUILD_TS={formatted}");
    }
}
