use colored::Colorize;
use reqwest::blocking::{Client, Response};
use serde_json::json;
use spinners::Spinner;

use crate::prompts;
use crate::{Cli, Config, Mode};

pub struct ChatCompletions {
    system_prompt: String,
    pub cli: Cli,
    pub config: Config,
    pub mode: Mode,
}

impl ChatCompletions {
    pub fn new(cli: Cli, config: Config) -> Self {
        let mode = match cli.mode.as_str() {
            "commit" => Mode::Commit,
            "command" => Mode::Command,
            _ => Mode::Command,
        };
        Self {
            system_prompt: "".to_string(),
            cli,
            config,
            mode,
        }
    }

    pub fn set_system_prompt(&mut self, system_prompt: prompts::SystemPrompt) {
        self.system_prompt = system_prompt.prompt(&self.cli);
    }

    pub fn run(self: &Self, prompt: String, spinner: &mut Spinner) -> String {
        let client = Client::new();
        let api_addr = format!("{}/chat/completions", self.config.api_base);
        let max_tokens = self.cli.token_limit.unwrap_or(self.config.max_tokens);

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
                    {"role": "system", "content": self.system_prompt},
                    {"role": "user", "content": prompt}
                ],
            }))
            .header("Authorization", format!("Bearer {}", &self.config.api_key))
            .send()
            .unwrap();

        let validated_response = self.validate_response(response, spinner);
        let response_string = validated_response.json::<serde_json::Value>().unwrap()["choices"][0]
            ["message"]["content"]
            .as_str()
            .unwrap()
            .trim()
            .to_string();

        return response_string;
    }

    fn validate_response(self: &Self, response: Response, spinner: &mut Spinner) -> Response {
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
        return response;
    }
}
