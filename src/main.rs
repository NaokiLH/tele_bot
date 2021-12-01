use dashmap::DashMap;
use lazy_static::lazy_static;
use std::error::Error;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommand;
const URL: &str = "https://fanyi-api.baidu.com/api/trans/vip/translate";
const APPID: &str = "20211201001015516";
const KEY: &str = "UiMAg7xBtMiUM1azC4e1";

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    Help,
    List,
    Add(String),
    Remove(String),
    Clear,
}

lazy_static! {
    static ref WORDS: DashMap<String, String> = {
        let m = DashMap::new();
        m
    };
}

#[tokio::main]
async fn main() {
    run().await;
}

async fn answer(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    command: Command,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match command {
        Command::Help => {
            let mes = String::from(
                "These commands are supported:\n\
                 /list - list all words\n\
                 /add <word> - add word\n\
                 /remove <word> - remove word\n\
                 /clear - clear all words",
            );
            cx.answer(mes).send().await?;
        }
        Command::List => {
            if WORDS.is_empty() {
                cx.answer("No words").send().await?;
            } else {
                let mes = WORDS
                    .iter()
                    .map(|ref_mut| format!("{} - {}", ref_mut.key(), ref_mut.value()))
                    .collect::<Vec<String>>()
                    .join("\n");
                cx.answer(mes).send().await?;
            }
        }
        Command::Clear => {
            WORDS.clear();
            cx.answer("All words are removed").send().await?;
        }
        Command::Add(word) => {
            if WORDS.contains_key(&word) {
                cx.answer("Word is already in list").send().await?;
            } else {
                if let Ok(Some(v)) =
                    rq_tran::rq_tran::translater::translate(&word, URL, APPID, KEY).await
                {
                    WORDS.insert(word, v);
                    cx.answer("Word is added").send().await?;
                } else {
                    cx.answer("Word added fail").send().await?;
                }
            }
        }
        Command::Remove(word) => {
            if WORDS.remove(&word.to_lowercase()).is_some() {
                cx.answer("Word is removed").send().await?;
            } else {
                cx.answer("Word is not found").send().await?;
            }
        }
    }
    Ok(())
}

async fn run() {
    teloxide::enable_logging!();
    log::info!("Bot started");
    let bot = Bot::from_env().auto_send();
    let bot_name = String::from("ndx");
    teloxide::commands_repl(bot, bot_name, answer).await;
}
