use crate::Cli;

pub enum SystemPrompt {
    Cmd,
    Commit,
}

impl SystemPrompt {
    pub fn prompt(self: Self, options: &Cli) -> String {
        match self {
            SystemPrompt::Cmd => cmd_system_prompt(),
            SystemPrompt::Commit => {
                return commit_system_prompt(options.gitmoji);
            }
        }
    }
}

pub fn cmd_system_prompt() -> String {
    let mut prompt = String::new();
    prompt.push_str(
        "You will be providing the command to run on the system based on the user inputs.\n",
    );
    return prompt;
}

pub fn commit_system_prompt(gitmoji: bool) -> String {
    let mut prompt = String::new();
    prompt.push_str("You are an assistant to a programmer that will be generating commit messages for the code changes");
    prompt.push_str(
        "\nYour task if to identify the key changes and prepare a single commit message that encapsulates the changes accordingly.",
    );
    if gitmoji {
        prompt.push_str(" (using gitmoji emojis)");
    }

    prompt.push_str("\nFollowing the format: <type> ([optional scope]): <short description>\n\n[optional body]\n[optional footer]\n");
    return prompt;
}

pub fn get_cmd_user_prompt(prompt: &str) -> String {
    let os_hint = hint_os();
    return format!("{prompt}{os_hint}:\n```bash\n#!/bin/bash\n");
}

pub fn get_commit_user_prompt(changes: Vec<String>, hint: &Option<String>) -> String {
    let mut prompt = String::new();
    if let Some(hint) = hint {
        prompt.push_str(format!("Hint: {}", hint).as_str());
    }
    prompt.push_str("Provide a commit message for the following changes:\n");

    for change in changes {
        prompt.push_str(change.as_str());
        prompt.push_str("\n");
    }
    return prompt;
}

fn hint_os() -> String {
    let os_hint = if cfg!(target_os = "macos") {
        " (on macOS)"
    } else if cfg!(target_os = "linux") {
        " (on Linux)"
    } else {
        ""
    };

    return os_hint.to_string();
}
