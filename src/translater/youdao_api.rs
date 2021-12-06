use super::Translater;
use async_trait::async_trait;
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use sha256::digest;
use std::collections::HashMap;
use std::error::Error;

const YOUDAO_API: &str = "https://openapi.youdao.com/api";

#[derive(Debug, Clone)]
pub struct Youdao {
    app_id: String,
    app_key: String,
    body: HashMap<String, String>,
    client: Client,
}

#[derive(Debug, Serialize, Deserialize)]
struct Tranresult {
    error_code: Option<i32>,
    query: String,
    translation: Vec<String>,
    basic: Basic,
    web: Vec<Web>,
}
#[derive(Debug, Serialize, Deserialize)]
struct Basic {
    phonetic: String,
    explains: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
struct Web {
    key: String,
    value: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
struct Testtrans {
    translation: Vec<String>,
}

impl Youdao {
    pub fn new(app_id: String, app_key: String) -> Self {
        let map = HashMap::new();
        Youdao {
            app_id,
            app_key,
            body: map,
            client: Client::new(),
        }
    }
    fn post(&self, url: String) -> RequestBuilder {
        self.client.post(url)
    }
    fn compute(&mut self, text: String, from: String, to: String) {
        let input = match text.len() {
            x if x >= 10 => format!("{}{}{}", &text[0..10], text.len(), &text[text.len() - 10..]),
            _ => text.clone(),
        };
        let salt = time::get_time().sec.to_string();
        let curtime = time::now_utc().to_timespec().sec.to_string();
        let secret_key = format!(
            "{}{}{}{}{}",
            self.app_id, input, salt, curtime, self.app_key
        );
        self.body.insert("q".to_string(), text);
        self.body.insert("from".to_string(), from);
        self.body.insert("to".to_string(), to);
        self.body.insert("appKey".to_string(), self.app_id.clone());
        self.body.insert("salt".to_string(), salt);
        self.body.insert("sign".to_string(), digest(secret_key));
        self.body.insert("signType".to_string(), "v3".to_string());
        self.body.insert("curtime".to_string(), curtime);
    }
}
#[async_trait]
impl Translater for Youdao {
    async fn translate(
        &mut self,
        text: String,
        from: String,
        to: String,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        self.compute(text, from, to);
        let req = self
            .post(YOUDAO_API.to_string())
            .form(&self.body)
            .send()
            .await?;

        let text = req.text().await?;
        let tranresult: Tranresult = serde_json::from_str(&text)?;
        let s = format!("翻译：{:?}", tranresult.translation);

        Ok(s)
    }
    async fn dictionary(
        &mut self,
        word: String,
        from: String,
        to: String,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        self.compute(word, from, to);
        let req = self
            .post(YOUDAO_API.to_string())
            .form(&self.body)
            .send()
            .await?;

        let text = req.text().await?;
        let tranresult: Tranresult = serde_json::from_str(&text)?;
        let s = format!(
            "翻译：{:?}\n音标：{}\n释义：{:?}\n",
            tranresult.translation, tranresult.basic.phonetic, tranresult.basic.explains,
        );

        Ok(s)
    }
}

#[test]
fn test_translate() {
    let mut youdao = Youdao::new(
        "1758905c74df1d80".to_string(),
        "zN317py0Xo9GuFAAh8t8IrkfTUGB5zml".to_string(),
    );

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(youdao.dictionary(
        "fanbook".to_string(),
        "en".to_string(),
        "zh-CHS".to_string(),
    ));

    if let Ok(text) = result {
        println!("{}", text);
    }
}
