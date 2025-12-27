fn run_cmd(cmd: &str, args: &[&str]) -> String {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map_or_else(|| "unknown".to_string(), |s| s.trim().to_string())
}

fn main() {
    let jj_change = run_cmd(
        "jj",
        &["log", "-r", "@", "-T", "change_id.short(8)", "--no-graph"],
    );
    println!("cargo:rustc-env=JJ_CHANGE_ID={jj_change}");

    let git_commit = run_cmd(
        "jj",
        &["log", "-r", "@", "-T", "commit_id.short(8)", "--no-graph"],
    );
    println!("cargo:rustc-env=GIT_COMMIT={git_commit}");

    let date = run_cmd("date", &["-u", "+%Y-%m-%d"]);
    println!("cargo:rustc-env=BUILD_DATE={date}");

    println!("cargo:rerun-if-changed=.jj/repo/op_heads");
}
