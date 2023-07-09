use std::process::Command;

pub fn get_commit_changes() -> Option<Vec<String>> {
    // Get the changes in the working directory
    let diff = Command::new("git")
        .arg("diff")
        .arg("--cached")
        .output()
        .unwrap_or_else(|_| {
            println!("Failed to execute git diff.");
            std::process::exit(1);
        });

    let diff = String::from_utf8_lossy(&diff.stdout);
    if diff.is_empty() {
        return None;
    }
    // Skip first line
    let diff = diff
        .lines()
        .skip(1)
        .map(|line| line.to_string())
        .collect::<Vec<String>>();
    return Some(diff);
}
