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
        "You are an assistant to a programmer that will be running commands on the system",
    );
    prompt.push_str(
        "\nYour task if to identify the key inputs and prepare a single command that encapsulates the inputs accordingly.",
    );
    prompt.push_str("\nFollowing the format: <command> <input1> <input2> ... <inputN>\n");
    prompt.push_str("Example: ls -l -a -h\n");
    prompt.push_str("Example: git commit -m \"<message>\"\n");
    prompt.push_str("Example: cat /etc/passwd | awk -F: '{ print $1 }'\n");
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
    return format!("{prompt}{os_hint}:\n");
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
