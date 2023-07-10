use crate::Cli;
use serde::{Deserialize, Serialize};

pub enum SystemPrompt {
    Cmd,
    Commit,
}

impl SystemPrompt {
    pub fn prompt(self: Self, options: &Cli) -> Prompt {
        return match self {
            SystemPrompt::Cmd => cmd_system_prompt(),
            SystemPrompt::Commit => commit_system_prompt(options.gitmoji),
        };
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Prompt {
    pub role: String,
    pub content: String,
}

impl Prompt {
    pub fn new() -> Self {
        return Prompt {
            role: String::new(),
            content: String::new(),
        };
    }

    pub fn build(role: String, content: String) -> Self {
        return Prompt { role, content };
    }
}

pub fn cmd_system_prompt() -> Prompt {
    let mut content = String::new();
    content.push_str(
        "You are an assistant to a programmer that will be running commands on the system",
    );
    content.push_str(
        "\nYour task if to identify the key inputs and prepare a single command that encapsulates the inputs accordingly.",
    );
    content.push_str("\nFollowing the format: <command> <input1> <input2> ... <inputN>\n");
    content.push_str("Example: ls -l -a -h\n");
    content.push_str("Example: git commit -m \"<message>\"\n");
    content.push_str("Example: cat /etc/passwd | awk -F: '{ print $1 }'\n");
    return Prompt::build("system".to_string(), content);
}

pub fn commit_system_prompt(gitmoji: bool) -> Prompt {
    let mut content = String::new();
    content.push_str("You are an assistant to a programmer that will be generating commit messages for the code changes");
    content.push_str(
        "\nYour task if to identify the key changes and prepare a single commit message that encapsulates the changes accordingly.",
    );
    if gitmoji {
        content.push_str(" (using gitmoji emojis)");
    }

    content.push_str("\nFollowing the format: <type> ([optional scope]): <short description>\n\n[optional body]\n[optional footer]\n");
    return Prompt::build("system".to_string(), content);
}

pub fn get_cmd_user_prompt(prompt: &str) -> Prompt {
    let os_hint = hint_os();
    return Prompt::build("user".to_string(), format!("{}{}:\n", prompt, os_hint));
}

pub fn get_commit_user_prompt(changes: Vec<String>, hint: &Option<String>) -> Prompt {
    let mut content = String::new();
    if let Some(hint) = hint {
        content.push_str(format!("Hint: {}", hint).as_str());
    }
    content.push_str("Provide a commit message for the following changes:\n");

    for change in changes {
        content.push_str(change.as_str());
        content.push_str("\n");
    }
    return Prompt::build("user".to_string(), content);
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
