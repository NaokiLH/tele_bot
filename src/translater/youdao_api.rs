use super::{Tranresult, Translater};
use async_trait::async_trait;
use reqwest::{Client, RequestBuilder};
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
    fn compute(&mut self, text: &String, from: &'static str, to: &'static str) {
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
        self.body.insert("q".to_string(), text.clone());
        self.body.insert("from".to_string(), from.to_string());
        self.body.insert("to".to_string(), to.to_string());
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
        text: &String,
        from: &'static str,
        to: &'static str,
    ) -> Result<Tranresult, Box<dyn Error + Send + Sync>> {
        self.compute(text, from, to);
        let req = self
            .post(YOUDAO_API.to_string())
            .form(&self.body)
            .send()
            .await?;

        let text = req.text().await?;
        let tranresult: Tranresult = serde_json::from_str(&text)?;

        Ok(tranresult)
    }
    async fn dictionary(
        &mut self,
        word: &String,
        from: &'static str,
        to: &'static str,
    ) -> Result<Tranresult, Box<dyn Error + Send + Sync>> {
        self.compute(word, from, to);
        let req = self
            .post(YOUDAO_API.to_string())
            .form(&self.body)
            .send()
            .await?;

        let text = req.text().await?;
        let tranresult: Tranresult = serde_json::from_str(&text)?;

        Ok(tranresult)
    }
}

#[test]
fn test_translate() {
    let mut youdao = Youdao::new(
        "1758905c74df1d80".to_string(),
        "zN317py0Xo9GuFAAh8t8IrkfTUGB5zml".to_string(),
    );

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(youdao.dictionary(&"fanbook".to_string(), "en", "zh-CHS"));

    if let Ok(text) = result {
        println!("{:?}", text.pretty());
    } else {
        println!("dsada");
    }
}
