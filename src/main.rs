#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use bat::PrettyPrinter;
use clap::Parser;
use colored::Colorize;
use config::Config;
use question::{Answer, Question};
use regex::Regex;
use reqwest::blocking::{Client, Response};
use serde_json::json;
use spinners::{Spinner, Spinners};
use std::process::Command;

mod config;

const MAX_TOKENS: usize = 100;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Description of the command to execute
    prompt: Vec<String>,

    #[clap(short, long, default_value = "default")]
    mode: String,

    // Gitmoji support
    #[clap(short, long)]
    gitmoji: bool,

    /// Run the generated program without asking for confirmation
    #[clap(short = 'y', long)]
    force: bool,

    #[clap(short, long)]
    token_limit: Option<usize>,
}

enum Mode {
    Command,
    Commit,
}

fn get_commit_changes() -> Vec<String> {
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
        println!("No changes to commit.");
        std::process::exit(0);
    }
    // Skip first line
    let diff = diff
        .lines()
        .skip(1)
        .map(|line| line.to_string())
        .collect::<Vec<String>>();
    return diff;
}

fn get_ai_response(
    system_prompt: String,
    user_prompt: String,
    cli: &Cli,
    config: &Config,
) -> Response {
    let client = Client::new();
    let api_addr = format!("{}/chat/completions", config.api_base);
    let max_tokens = cli.token_limit.unwrap_or(MAX_TOKENS);

    let response = client
        .post(api_addr)
        .json(&json!({
            "top_p": 1,
            "temperature": 0,
            "max_tokens": max_tokens,
            "presence_penalty": 0,
            "frequency_penalty": 0,
            "model": "gpt-3.5-turbo",
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
        }))
        .header("Authorization", format!("Bearer {}", &config.api_key))
        .send()
        .unwrap();

    return response;
}

fn main() {
    println!(
        "{} {}",
        "🤖".bright_green(),
        "Welcome to GitAI!".bright_green()
    );
    let cli = Cli::parse();
    let mut mode = Mode::Command;
    println!("AI prompt: {}", cli.prompt.join(" "));
    match &*cli.mode {
        "commit" => mode = Mode::Commit,
        "command" => mode = Mode::Command,
        _ => (),
    }
    println!("AI mode: {}", cli.mode);
    let config = Config::new();

    match mode {
        Mode::Command => {
            command_run_workflow(cli, &config);
        }
        Mode::Commit => {
            commit_workflow(cli, &config);
        }
    }
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
fn build_cmd_prompt(prompt: &str) -> (String, String) {
    let os_hint = hint_os();
    return (
        "system".to_string(),
        format!("{prompt}{os_hint}:\n```bash\n#!/bin/bash\n"),
    );
}

fn build_commit_prompt(changes: Vec<String>, gitmoji: bool) -> (String, String) {
    let mut system_prompt = "You are an assistant to a programmer that will be generating commit messages for the code changes, your commit messages".to_string();
    system_prompt.push_str(
        "\nYour task if to identify the key changes and prepare the commit message accordingly.",
    );
    if gitmoji {
        system_prompt.push_str(" (using gitmoji)");
    }
    let commit_format_hint =
        "\nFollowing the format: <type> ([optional scope]): <short description>\n\n[optional body]\n[optional footer]\n";
    system_prompt.push_str(commit_format_hint);
    let mut user_prompt = "Provide a commit message for the following changes:\n".to_string();

    for change in changes {
        user_prompt.push_str(change.as_str());
        user_prompt.push('\n');
    }
    return (system_prompt, user_prompt);
}

fn validate_response(response: Response, mut spinner: Spinner) -> (Response, Spinner) {
    let status_code = response.status();
    if status_code.is_client_error() {
        let response_body = response.json::<serde_json::Value>().unwrap();
        let error_message = response_body["error"]["message"].as_str().unwrap();
        spinner.stop_and_persist(
            "✖".red().to_string().as_str(),
            format!("API error: \"{error_message}\"").red().to_string(),
        );
        std::process::exit(1);
    } else if status_code.is_server_error() {
        spinner.stop_and_persist(
            "✖".red().to_string().as_str(),
            format!("OpenAI is currently experiencing problems. Status code: {status_code}")
                .red()
                .to_string(),
        );
        std::process::exit(1);
    }
    return (response, spinner);
}

fn command_run_workflow(cli: Cli, config: &Config) {
    let spinner = Spinner::new(Spinners::BouncingBar, "Generating your command...".into());
    let (system_prompt, user_prompt) = build_cmd_prompt(&cli.prompt.join(" "));
    let response = get_ai_response(system_prompt, user_prompt, &cli, &config);
    let (response, mut spinner) = validate_response(response, spinner);

    let code = response.json::<serde_json::Value>().unwrap()["choices"][0]["text"]
        .as_str()
        .unwrap()
        .trim()
        .to_string();

    spinner.stop_and_persist(
        "✔".green().to_string().as_str(),
        "Got some code!".green().to_string(),
    );

    PrettyPrinter::new()
        .input_from_bytes(code.as_bytes())
        .language("bash")
        .grid(true)
        .print()
        .unwrap();

    let should_run = if cli.force {
        true
    } else {
        Question::new(
            ">> Run the generated program? [Y/n]"
                .bright_black()
                .to_string()
                .as_str(),
        )
        .yes_no()
        .until_acceptable()
        .default(Answer::YES)
        .ask()
        .expect("Couldn't ask question.")
            == Answer::YES
    };

    if should_run {
        config.write_to_history(code.as_str());
        spinner = Spinner::new(Spinners::BouncingBar, "Executing...".into());

        let output = Command::new("bash")
            .arg("-c")
            .arg(code.as_str())
            .output()
            .unwrap_or_else(|_| {
                spinner.stop_and_persist(
                    "✖".red().to_string().as_str(),
                    "Failed to execute the generated program.".red().to_string(),
                );
                std::process::exit(1);
            });

        if !output.status.success() {
            spinner.stop_and_persist(
                "✖".red().to_string().as_str(),
                "The program threw an error.".red().to_string(),
            );
            println!("{}", String::from_utf8_lossy(&output.stderr));
            std::process::exit(1);
        }

        spinner.stop_and_persist(
            "✔".green().to_string().as_str(),
            "Command ran successfully".green().to_string(),
        );

        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
}

fn commit_workflow(cli: Cli, config: &Config) {
    let spinner = Spinner::new(
        Spinners::BouncingBar,
        "Generating your commit message...".into(),
    );

    let commit_changes = get_commit_changes();
    let (system_prompt, user_prompt) = build_commit_prompt(commit_changes, cli.gitmoji);
    let response = get_ai_response(system_prompt, user_prompt, &cli, &config);
    let (response, mut spinner) = validate_response(response, spinner);

    let mut commit_message = response.json::<serde_json::Value>().unwrap()["choices"][0]["message"]
        ["content"]
        .as_str()
        .unwrap()
        .trim()
        .to_string();

    if cli.gitmoji {
        commit_message = replace_gitmoji(commit_message);
    }

    spinner.stop_and_persist(
        "✔".green().to_string().as_str(),
        "Got your commit message!".green().to_string(),
    );

    PrettyPrinter::new()
        .input_from_bytes(commit_message.as_bytes())
        .language("bash")
        .grid(true)
        .print()
        .unwrap();

    let accept_commit = Question::new(
        ">> Accept the generated commit? [Y/n]"
            .bright_black()
            .to_string()
            .as_str(),
    )
    .yes_no()
    .until_acceptable()
    .default(Answer::YES)
    .ask()
    .expect("Couldn't ask question.")
        == Answer::YES;

    if accept_commit {
        let generate_commit_cmd = Question::new(
            ">> Generate a commit with the generated message? [Y/n]"
                .bright_black()
                .to_string()
                .as_str(),
        )
        .yes_no()
        .until_acceptable()
        .default(Answer::YES)
        .ask()
        .expect("Couldn't ask question.")
            == Answer::YES;

        if generate_commit_cmd {
            let mut commit_cmd = "git commit -m '".to_string();
            commit_cmd.push_str(commit_message.as_str());
            commit_cmd.push_str("'");

            PrettyPrinter::new()
                .input_from_bytes(commit_cmd.as_bytes())
                .language("bash")
                .grid(true)
                .print()
                .unwrap();

            let should_run = Question::new(
                ">> Run the generated commit? [Y/n]"
                    .bright_black()
                    .to_string()
                    .as_str(),
            )
            .yes_no()
            .until_acceptable()
            .default(Answer::YES)
            .ask()
            .expect("Couldn't ask question.")
                == Answer::YES;

            if should_run {
                spinner = Spinner::new(Spinners::BouncingBar, "Executing...".into());
                let output = Command::new("bash")
                    .arg("-c")
                    .arg(commit_cmd.as_str())
                    .output()
                    .unwrap_or_else(|_| {
                        spinner.stop_and_persist(
                            "✖".red().to_string().as_str(),
                            "Failed to execute the generated program.".red().to_string(),
                        );
                        std::process::exit(1);
                    });

                if !output.status.success() {
                    spinner.stop_and_persist(
                        "✖".red().to_string().as_str(),
                        "The program threw an error.".red().to_string(),
                    );
                    println!("{}", String::from_utf8_lossy(&output.stderr));
                    std::process::exit(1);
                }

                spinner.stop_and_persist(
                    "✔".green().to_string().as_str(),
                    "Commit generated successfully".green().to_string(),
                );
            }
        }
    }
}

fn get_gitmojis(tag: String) -> String {
    let awk_cmd = "awk '{print $1}'";
    let gitmoji = Command::new("bash")
        .arg("-c")
        .arg(format!("gitmoji -s {} | {}", tag, awk_cmd))
        .output()
        .unwrap_or_else(|_| {
            println!("Failed to execute gitmoji.");
            std::process::exit(1);
        });
    // Check if gitmoji is empty
    if gitmoji.stdout.is_empty() {
        println!("No gitmojis found.");
        let out = String::from_utf8_lossy(&gitmoji.stdout).to_string();
        println!("{}", out);
        std::process::exit(1);
    }
    return String::from_utf8_lossy(&gitmoji.stdout)
        .to_string()
        .trim()
        .to_string();
}

fn replace_gitmoji(commit_message: String) -> String {
    let mut new_message = commit_message.clone();
    // Parse the string looking for unique gitmojis tags, e.g. :bug:
    let re = Regex::new(r":\w+:").unwrap();
    let matches: Vec<_> = re.find_iter(&commit_message).collect();
    // If there are no matches, return the original message
    if matches.is_empty() {
        return commit_message;
    }
    // Get the gitmojis
    for tag in matches {
        let gitmoji = get_gitmojis(tag.as_str().to_string());
        // If there are no gitmojis for the tag, skip it
        if gitmoji.is_empty() {
            continue;
        }
        // Replace the tag with the gitmoji
        new_message = new_message.replace(&tag.as_str(), &gitmoji);
    }
    return new_message;
}
