use std::env;
use std::process::Command;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

const COMPILE: &str = "|compile";

#[derive(Debug)]
enum Language {
    Rust,
    Java,
    Snow,
    None,
}

impl Language {
    fn len(&self) -> usize {
        format!("{self:?}").len()
    }
}

impl From<&str> for Language {
    fn from(lang: &str) -> Self {
        match lang {
            "rust" => Self::Rust,
            "java"  => Self::Java,
            "snow"  => Self::Snow,
            _ => Self::None
        }
    }
}

#[derive(Debug)]
struct CodeBlock {
    lang: Language,
    code: String,
}

impl CodeBlock {
    fn compile(&self) -> String {
        let output = match self.lang {
            Language::Rust => self.rust(),
            Language::Java => self.java(),
            Language::Snow => self.snow(),
            Language::None => "not a supported language".into(),
        };
        let lang_type = format!("{:?}", self.lang).to_lowercase();
        format!("```{lang_type}\n{output}```")
    }

    fn rust(&self) -> String {
        let Ok(_) = std::fs::write("rust_file.rs", &self.code) else {
            return "".into();
        };
        let Ok(output) = Command::new("rustc").arg("rust_file.rs").output() else {
            return "".into();
        };
        let Ok(out) = String::from_utf8(output.stdout) else {
            return "".into();
        };
        let Ok(err) = String::from_utf8(output.stderr) else {
            return "".into();
        };
        format!("{out}\n{err}")
    }

    fn java(&self) -> String {
        let Ok(_) = std::fs::write("Main.java", &self.code) else {
            return "".into();
        };
        match Command::new("javac").arg("Main.java").output() {
            Ok(output) => {

                let Ok(out) = String::from_utf8(output.stdout) else {
                    return "".into();
                };
                let Ok(err) = String::from_utf8(output.stderr) else {
                    return "".into();
                };
                format!("{out}\n{err}")
            }
            Err(e) => e.to_string()
        }
    }

    fn snow(&self) -> String {
        let Ok(_) = std::fs::write("snow_file.rs", &self.code) else {
            return "".into();
        };
        let Ok(output) = Command::new("snowc").arg("snowc_file.rs").output() else {
            return "".into();
        };
        let Ok(out) = String::from_utf8(output.stdout) else {
            return "".into();
        };
        let Ok(err) = String::from_utf8(output.stderr) else {
            return "".into();
        };
        format!("{out}\n{err}")
    }
}

impl From<&str> for CodeBlock {
    fn from(code_block: &str) -> Self {
        let name = code_block.split('\n').collect::<Vec<_>>()[0];
        let lang = Language::from(name);
        let code = code_block[lang.len()..].trim().to_string();
        Self { lang, code }
    }
}

fn strip_code_of_backticks<'a>(code: &'a str) -> &'a str {
    code
        .trim()
        .strip_prefix("```")
        .map(|i|i.strip_suffix("```"))
        .flatten()
        .unwrap_or(code.trim()
            .strip_prefix("`")
            .map(|i|i.strip_suffix("`"))
            .flatten().unwrap_or(code))
}

fn get_code_block<'a>(msg: &'a str, command: &str) -> Result<&'a str, &'a str> {
    println!("{}", msg);
    let code_block = msg.get(command.len()..);
    let is_empty = code_block.map(|i| i.is_empty()).unwrap_or(true);
    let (false, Some(code)) = (is_empty, code_block) else {
        return Err("No code block was given to");
    };
    Ok(strip_code_of_backticks(code))
}

// fn format_block(msg: &str) -> String {
//     let line_len = msg.lines().count();
//     let prefix = if line_len > 1 { "```" } else {"`"};
//     format!("{prefix}{msg}{prefix}")
// }

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with(COMPILE) {
            match get_code_block(msg.content.as_str(), COMPILE) {
                Ok(code) => {
                    let code_block = CodeBlock::from(code);
                    let output = code_block.compile();
                    if let Err(why) = msg.channel_id.say(&ctx.http, output).await {
                        println!("Error sending message: {:?}", why);
                    }
                }
                Err(err) => {
                    let output = format!("{err} {COMPILE}");
                    if let Err(why) = msg.channel_id.say(&ctx.http, output).await {
                        println!("Error sending message: {:?}", why);
                    }
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

