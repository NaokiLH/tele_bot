use dashmap::DashMap;
use std::error::Error;
use std::sync::Arc;
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

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    Help,
    List,
    Add(String),
    Remove(String),
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
        Command::Help => {
            let mes = String::from(
                "These commands are supported:\n\
                 /list - list all words\n\
                 /add <word> - add word\n\
                 /remove <word> - remove word\n\
                 /clear - clear all words
                 /exam - exam all words",
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
        Command::Add(word) => {
            if words.contains_key(&word) {
                cx.answer("Word is already in list").send().await?;
            } else {
                let mut translater = Youdao::new(APPID.to_string(), APPKEY.to_string());
                if let Ok(v) = translater.dictionary(&word, "en", "zh-CHS").await {
                    words.insert(word, v.pretty());
                    cx.answer("Word is added").send().await?;
                } else {
                    cx.answer("Word added fail").send().await?;
                }
            }
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
    if cx.update.query.is_empty() {
        cx.requester
            .answer_inline_query(
                cx.update.id,
                vec![InlineQueryResult::Article(InlineQueryResultArticle::new(
                    "1",
                    format!("list your today word"),
                    InputMessageContent::Text(InputMessageContentText::new("/list".to_string())),
                ))],
            )
            .await?;
        return Ok(());
    }
    let word = cx.update.query.to_lowercase();
    let word_clone = word.clone();
    let mut translater = Youdao::new(APPID.to_string(), APPKEY.to_string());
    match translater.dictionary(&word, "en", "zh-CHS").await {
        Ok(v) => {
            let result = vec![InlineQueryResult::Article(
                InlineQueryResultArticle::new(
                    "1",
                    format!("{} - {}", word_clone, v.translation()),
                    InputMessageContent::Text(InputMessageContentText::new(v.pretty())),
                )
                .reply_markup(InlineKeyboardMarkup {
                    inline_keyboard: vec![vec![
                        InlineKeyboardButton::new(
                            "add",
                            InlineKeyboardButtonKind::CallbackData(format!("add {}", word_clone)),
                        ),
                        InlineKeyboardButton::new(
                            "search",
                            InlineKeyboardButtonKind::Url(format!(
                                "https://www.google.com/search?q={}",
                                word_clone.clone()
                            )),
                        ),
                    ]],
                }),
            )];

            cx.requester
                .answer_inline_query(cx.update.id, result)
                .await?;
        }
        _ => {
            let result = vec![InlineQueryResult::Article(InlineQueryResultArticle::new(
                "1",
                "No result found",
                InputMessageContent::Text(InputMessageContentText::new("No word has been found")),
            ))];
            cx.requester
                .answer_inline_query(cx.update.id, result)
                .await?;
        }
    };

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
        match text.as_str() {
            "delete" => {}
            _ => {
                log::info!("{}", text);
                if words.contains_key(&text) {
                    log::info!("contained");
                    requester
                        .answer_callback_query(update.id)
                        .text("word has already in list")
                        .await?;
                } else {
                    let mut translater = Youdao::new(APPID.to_string(), APPKEY.to_string());
                    words.insert(
                        text.clone(),
                        translater
                            .dictionary(&text, "en", "zh-CHS")
                            .await?
                            .translation(),
                    );
                    requester
                        .answer_callback_query(update.id)
                        .text("word added successfully")
                        .await?;
                }
            }
        }
    }

    Ok(())
}

async fn run() {
    teloxide::enable_logging!();
    log::info!("Bot started");
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
