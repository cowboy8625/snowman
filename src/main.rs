use std::env;
use snowc::*;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

const PARSE: &str = "@parse";
const EVAL: &str = "@eval";

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

fn format_block(msg: &str) -> String {
    let line_len = msg.lines().count();
    let prefix = if line_len > 1 { "```" } else {"`"};
    format!("{prefix}{msg}{prefix}")
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with(PARSE) {
            match get_code_block(msg.content.as_str(), PARSE) {
                Ok(code) => {
                    let output = parse(code, true)
                        .map(|ast| ast.iter()
                            .map(|func| {
                                format!("{func}\n")
                            }).collect::<String>()
                        ).unwrap_or_else(|why|format!("Error parsing '{code}' {why}"));
                    let output = format_block(&output);
                    println!("{output}");
                    if let Err(why) = msg.channel_id.say(&ctx.http, output).await {
                        println!("Error sending message: {:?}", why);
                    }
                }
                Err(err) => {
                    let output = format!("{err} {PARSE}");
                    if let Err(why) = msg.channel_id.say(&ctx.http, output).await {
                        println!("Error sending message: {:?}", why);
                    }
                }
            }

        } else if msg.content.starts_with(EVAL) {
            match get_code_block(msg.content.as_str(), EVAL) {
                Ok(code) => {
                    let output = format!("Not working at the moment as you can see\n[EVAL]: {code}");
                    let output = format_block(&output);
                    if let Err(why) = msg.channel_id.say(&ctx.http, output).await {
                        println!("Error sending message: {:?}", why);
                    }
                }
                Err(err) => {
                    let output = format!("{err} {EVAL}");
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

