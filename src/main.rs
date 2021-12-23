use dashmap::DashMap;
use std::error::Error;
use std::sync::Arc;
use teloxide::payloads::AnswerCallbackQuerySetters;
use teloxide::prelude::*;
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, InlineQueryResult,
    InlineQueryResultArticle, InputMessageContent, InputMessageContentText,
};
use teloxide::utils::command::BotCommand;
use tokio_stream::wrappers::UnboundedReceiverStream;
use translater::translater::youdao_api::Youdao;
use translater::translater::Translater;
const APPID: &str = "1758905c74df1d80";
const APPKEY: &str = "zN317py0Xo9GuFAAh8t8IrkfTUGB5zml";
const TELEGRAM_TOKEN: &str = "5049537837:AAHjmZovmdP6Ni8yWdfqJ3cbcd9jTNkG4Ek";

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    Add(String),
    Remove(String),
    Trans(String),
    Start,
    Help,
    List,
    Clear,
    Exam,
}

#[tokio::main]
async fn main() {
    run().await;
}

//CommandHandler
async fn cmd_answer(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    words: Arc<DashMap<String, String>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let command = match cx.update.text() {
        Some(text) => Command::parse(text, "npx")?,
        None => Command::Help,
    };
    match command {
        Command::Start => {
            unimplemented!()
        }
        Command::Help => {
            let mes = String::from(
                "These commands are supported:\n\
                /list - list all words\n\
                /add <word> - add word\n\
                /remove <word> - remove word\n\
                /clear - clear all words\n\
                /exam - exam all words\n\
                /trans - traslate words\n",
            );
            cx.answer(mes).send().await?;
        }
        Command::List => {
            if words.is_empty() {
                cx.answer("No words").send().await?;
            } else {
                let mes = words
                    .iter()
                    .map(|ref_mut| format!("{} - {}", ref_mut.key(), ref_mut.value()))
                    .collect::<Vec<String>>()
                    .join("\n");
                cx.answer(mes).send().await?;
            }
        }
        Command::Clear => {
            words.clear();
            cx.answer("All words are removed").send().await?;
        }
        Command::Trans(word) => {
            let mut translater = Youdao::new(APPID.to_string(), APPKEY.to_string());
            let result = translater.dic(&word, "en", "zh-CHS").await?;
            cx.requester
                .send_message(cx.update.chat_id(), result.pretty())
                .reply_markup(InlineKeyboardMarkup {
                    inline_keyboard: vec![vec![
                        InlineKeyboardButton::new(
                            "Add",
                            InlineKeyboardButtonKind::CallbackData(format!("add {}", word)),
                        ),
                        InlineKeyboardButton::new(
                            "Search",
                            InlineKeyboardButtonKind::Url(
                                String::from("https://www.google.com/search?q=") + &word,
                            ),
                        ),
                    ]],
                })
                .send()
                .await?;
        }
        Command::Add(word) => {
            let mut translater = Youdao::new(APPID.to_string(), APPKEY.to_string());
            let msg = match (
                words.contains_key(&word),
                translater.dic(&word, "en", "zh-CHS").await,
            ) {
                (false, Ok(result)) => {
                    words.insert(word.clone(), result.pretty());
                    format!("Word {} is added", word)
                }
                (false, Err(_)) => format!("Word {} add failed", word),
                (true, _) => format!("Word {} is already added", word),
            };
            cx.answer(msg).send().await?;
        }
        Command::Remove(word) => {
            if words.remove(&word.to_lowercase()).is_some() {
                cx.answer("Word is removed").send().await?;
            } else {
                cx.answer("Word is not found").send().await?;
            }
        }
        Command::Exam => {
            cx.reply_to("Today's word exam")
                .reply_markup(InlineKeyboardMarkup {
                    inline_keyboard: vec![vec![
                        InlineKeyboardButton::new(
                            "Start",
                            InlineKeyboardButtonKind::CallbackData(String::from("start")),
                        ),
                        InlineKeyboardButton::new(
                            "Stop",
                            InlineKeyboardButtonKind::CallbackData(String::from("stop")),
                        ),
                    ]],
                })
                .send()
                .await?;
        }
    }
    Ok(())
}

//InlineQueryHandler
async fn inq_anwser(
    cx: UpdateWithCx<AutoSend<Bot>, InlineQuery>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // if empty, return today's list of words
    let mut translater = Youdao::new(APPID.to_string(), APPKEY.to_string());
    let query = cx.update.query.to_lowercase();
    let result = match (
        query.is_empty(),
        translater.dic(&query, "en", "zh-CHS").await,
    ) {
        (true, _) => vec![InlineQueryResult::Article(InlineQueryResultArticle::new(
            "1",
            format!("list your today word"),
            InputMessageContent::Text(InputMessageContentText::new("/list".to_string())),
        ))],
        (false, Ok(trans_word)) => vec![InlineQueryResult::Article(InlineQueryResultArticle::new(
            "2",
            format!("{} - {}", &query, trans_word.translation()),
            InputMessageContent::Text(InputMessageContentText::new(format!("/trans {}", query))),
        ))],
        (false, Err(_)) => vec![InlineQueryResult::Article(InlineQueryResultArticle::new(
            "3",
            "No result found",
            InputMessageContent::Text(InputMessageContentText::new("No word has been found")),
        ))],
    };
    cx.requester
        .answer_inline_query(cx.update.id, result)
        .await?;

    Ok(())
}

//CallbackQueryHandler
async fn cbq_answer(
    cx: UpdateWithCx<AutoSend<Bot>, CallbackQuery>,
    words: Arc<DashMap<String, String>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let UpdateWithCx {
        requester, update, ..
    } = cx;
    if let Some(text) = update.data {
        let args = text.split_whitespace().collect::<Vec<&str>>();
        match args[0] {
            "add" => {
                let word = args[1].to_lowercase();
                let mut translater = Youdao::new(APPID.to_string(), APPKEY.to_string());
                let msg = match (
                    words.contains_key(&word),
                    translater.dic(&word, "en", "zh-CHS").await,
                ) {
                    (false, Ok(result)) => {
                        words.insert(word.clone(), result.pretty());
                        format!("Word {} is added", word)
                    }
                    (false, Err(_)) => format!("Word {} add failed", word),
                    (true, _) => format!("Word {} is already added", word),
                };
                requester.answer_callback_query(update.id).text(msg).await?;
            }
            _ => {}
        };
    }

    Ok(())
}

async fn run() {
    teloxide::enable_logging!();
    log::info!("Bot started");
    std::env::set_var("TELOXIDE_TOKEN", TELEGRAM_TOKEN);
    let bot = Bot::from_env().auto_send();
    let words = Arc::new(DashMap::new());
    let words1 = words.clone();
    let words2 = words.clone();
    Dispatcher::new(bot)
        .messages_handler(|rx: DispatcherHandlerRx<AutoSend<Bot>, Message>| {
            UnboundedReceiverStream::new(rx).for_each_concurrent(None, move |cx| {
                let words = words1.clone();
                async move {
                    cmd_answer(cx, words).await.log_on_error().await;
                }
            })
        })
        .inline_queries_handler(|rx: DispatcherHandlerRx<AutoSend<Bot>, InlineQuery>| {
            UnboundedReceiverStream::new(rx).for_each_concurrent(None, move |cx| async move {
                inq_anwser(cx).await.log_on_error().await;
            })
        })
        .callback_queries_handler(|rx: DispatcherHandlerRx<AutoSend<Bot>, CallbackQuery>| {
            UnboundedReceiverStream::new(rx).for_each_concurrent(None, move |cx| {
                let words = words2.clone();
                async move {
                    cbq_answer(cx, words).await.log_on_error().await;
                }
            })
        })
        .dispatch()
        .await;
}
