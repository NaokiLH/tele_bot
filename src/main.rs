use dashmap::DashMap;
use std::error::Error;
use std::sync::Arc;
use teloxide::payloads::{AnswerCallbackQuerySetters, EditMessageTextSetters, SendMessageSetters};
use teloxide::prelude::*;
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, InlineQueryResult,
    InlineQueryResultArticle, InputMessageContent, InputMessageContentText, ParseMode,
};
use teloxide::utils::command::BotCommand;
use tokio_stream::wrappers::UnboundedReceiverStream;
use translater::translater::youdao_api::Youdao;
use translater::translater::{Tranresult, Translater};
const APPID: &str = "1758905c74df1d80";
const APPKEY: &str = "zN317py0Xo9GuFAAh8t8IrkfTUGB5zml";
const TELEGRAM_TOKEN: &str = "5049537837:AAHjmZovmdP6Ni8yWdfqJ3cbcd9jTNkG4Ek";

// TO DO
// change dashmap to DB
// add the quiz function

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    Add(String),
    Remove(String),
    Start,
    Help,
    List(String),
    ClearWords,
    Exam,
}

#[tokio::main]
async fn main() {
    run().await;
}

//CommandHandler
async fn cmd_answer(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    words: Arc<DashMap<String, Tranresult>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let command = match cx.update.text() {
        Some(text) => Command::parse(text, "npx")?,
        None => Command::Help,
    };
    match command {
        Command::Help | Command::Start => {
            let mes = String::from(
                "These commands are supported:\n\
                /list -flag - list all words,flags: -e or -c\n\
                /add <word> - add word\n\
                /remove <word> - remove word\n\
                /clearwords - clear all words\n\
                /exam - exam all words\n\
                support inline query,\n\
                you can use inline mode to translate words\n",
            );
            cx.answer(mes).send().await?;
        }
        Command::List(mode) => {
            if words.is_empty() {
                cx.answer("No words").send().await?;
            } else {
                match mode.as_str() {
                    "-e" => {
                        let num = words.len();
                        let elist = words
                            .iter()
                            .map(|ref_mut| {
                                format!("{} - {}", ref_mut.key(), ref_mut.value().translation())
                            })
                            .collect::<Vec<String>>()
                            .join("\n");
                        cx.answer(format!("Your list hava {} words\n{}", num, elist))
                            .await?;
                    }
                    "-c" => {
                        let msg = words
                            .iter()
                            .map(|ref_mut| ref_mut.value().markdown())
                            .collect::<Vec<String>>();

                        cx.reply_to(msg[0].as_str())
                            .parse_mode(ParseMode::MarkdownV2)
                            .reply_markup(InlineKeyboardMarkup {
                                inline_keyboard: vec![vec![
                                    InlineKeyboardButton::new(
                                        "Last",
                                        InlineKeyboardButtonKind::CallbackData(format!(
                                            "last {}",
                                            -1
                                        )),
                                    ),
                                    InlineKeyboardButton::new(
                                        "Next",
                                        InlineKeyboardButtonKind::CallbackData(format!(
                                            "next {}",
                                            1
                                        )),
                                    ),
                                ]],
                            })
                            .send()
                            .await?;
                    }
                    _ => {
                        cx.answer("Wrong mode").send().await?;
                    }
                }
            }
        }
        Command::ClearWords => {
            words.clear();
            cx.answer("All words are removed").send().await?;
        }
        Command::Add(word) => {
            let mut translater = Youdao::new(APPID.to_string(), APPKEY.to_string());
            let msg = match (
                words.contains_key(&word),
                translater.dic(&word, "en", "zh-CHS").await,
            ) {
                (false, Ok(result)) => {
                    words.insert(word.clone(), result);
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
            cx.requester
                .send_poll(
                    cx.update.chat_id(),
                    "test question",
                    vec!["1".to_string(), "2".to_string(), "3".to_string()],
                    teloxide::types::PollType::Regular,
                )
                .await?;
        }
    }
    Ok(())
}

//InlineQueryHandler
async fn inq_anwser(
    cx: UpdateWithCx<AutoSend<Bot>, InlineQuery>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut translater = Youdao::new(APPID.to_string(), APPKEY.to_string());
    let query = cx.update.query.to_lowercase();
    // match arms return inline query result
    let result = match (
        query.is_empty(),
        translater.dic(&query, "en", "zh-CHS").await,
    ) {
        (true, _) => vec![InlineQueryResult::Article(InlineQueryResultArticle::new(
            "1",
            format!("list your today word"),
            InputMessageContent::Text(InputMessageContentText::new("/list".to_string())),
        ))],
        (false, Ok(trans_word)) => {
            vec![InlineQueryResult::Article(
                InlineQueryResultArticle::new(
                    "2",
                    format!("{}", trans_word.explains()),
                    InputMessageContent::Text(
                        InputMessageContentText::new(trans_word.markdown())
                            .parse_mode(ParseMode::MarkdownV2),
                    ),
                )
                .reply_markup(InlineKeyboardMarkup {
                    inline_keyboard: vec![vec![
                        InlineKeyboardButton::new(
                            "Add",
                            InlineKeyboardButtonKind::CallbackData(format!("add {}", query)),
                        ),
                        InlineKeyboardButton::new(
                            "Search",
                            InlineKeyboardButtonKind::Url(format!(
                                "https://www.google.com/search?q={}",
                                query
                            )),
                        ),
                    ]],
                }),
            )]
        }
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
    words: Arc<DashMap<String, Tranresult>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let UpdateWithCx {
        requester, update, ..
    } = cx;
    let update_id = update.id.clone();
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
                        words.insert(word.clone(), result);
                        format!("Word {} is added", word)
                    }
                    (false, Err(_)) => format!("Word {} add failed", word),
                    (true, _) => format!("Word {} is already added", word),
                };

                // need to test
                requester.answer_callback_query(update.id).text(msg).await?;
                match (update.inline_message_id, update.message) {
                    (Some(inline_message_id), _) => {
                        requester
                            .edit_message_text_inline(inline_message_id, "")
                            .await?;
                    }
                    (_, Some(message)) => {
                        requester.delete_message(update_id, message.id).await?;
                    }
                    _ => (),
                }
            }
            "next" | "last" => {
                let word_list = words
                    .iter()
                    .map(|ref_mut| ref_mut.value().markdown())
                    .collect::<Vec<String>>();
                match args[1].parse::<usize>() {
                    Ok(index) if index < word_list.len() => {
                        requester
                            .edit_message_text(
                                update.from.id,
                                update.message.unwrap().id,
                                word_list[index].clone(),
                            )
                            .parse_mode(ParseMode::MarkdownV2)
                            .reply_markup(InlineKeyboardMarkup {
                                inline_keyboard: vec![vec![
                                    InlineKeyboardButton::new(
                                        "Last",
                                        InlineKeyboardButtonKind::CallbackData(format!(
                                            "last {}",
                                            index as i32 - 1
                                        )),
                                    ),
                                    InlineKeyboardButton::new(
                                        "Next",
                                        InlineKeyboardButtonKind::CallbackData(format!(
                                            "next {}",
                                            index + 1
                                        )),
                                    ),
                                ]],
                            })
                            .send()
                            .await?;
                    }
                    _ => {
                        requester
                            .answer_callback_query(update.id)
                            .text("this page does not exist")
                            .await?;
                    }
                };
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
