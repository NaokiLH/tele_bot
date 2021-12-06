use rand::Rng;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::str::from_utf8;

#[derive(Debug, Serialize, Deserialize)]
struct Transword {
    from: String,
    to: String,
    trans_result: Vec<TransResult>,
    error_code: Option<i32>,
}
#[derive(Debug, Deserialize, Serialize)]
struct TransResult {
    src: String,
    dst: String,
}
pub async fn translate(
    word: &String,
    url: &str,
    appid: &str,
    key: &str,
) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    //produce request
    let q = word;
    let from = "en";
    let to = "zh";
    let salt = rand::thread_rng().gen_range(0..=100);
    let sign = format!("{}{}{}{}", appid, q, salt, key);
    let sign = md5::compute(sign);
    let url = format!(
        "{}?q={}&from={}&to={}&appid={}&salt={}&sign={:?}",
        url, q, from, to, appid, salt, sign
    );
    //send request
    let resp = reqwest::get(&url).await?.text().await?;
    //parse response
    let transword: Transword = serde_json::from_str(&resp)?;
    log::debug!("{:?}", transword);
    if let Some(error_code) = transword.error_code {
        return Err(format!("{}", error_code).into());
    }

    let result = transword.trans_result;
    if result.len() == 0 {
        return Ok(None);
    }

    let transword = result[0].dst.as_bytes();
    let word = from_utf8(transword)?;

    Ok(Some(word.to_string()))
}
// To do
// dictionary api
#[test]
fn test_translate() {
    let word = "fanbook".to_string();
    let url = "https://fanyi-api.baidu.com/api/trans/vip/translate";
    let appid = "20211201001015516";
    let key = "UiMAg7xBtMiUM1azC4e1";

    let word = translate(&word, url, appid, key);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let handle = rt.handle();
    handle.block_on(async {
        let word = word.await.unwrap().unwrap();
        println!("{:?}", word);
    });
}
