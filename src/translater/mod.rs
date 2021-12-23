use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
pub mod baidu_api;
pub mod youdao_api;

#[async_trait]
pub trait Translater {
    async fn trans(
        &mut self,
        text: &String,
        from: &'static str,
        to: &'static str,
    ) -> Result<Tranresult, Box<dyn Error + Send + Sync>>;
    async fn dic(
        &mut self,
        text: &String,
        from: &'static str,
        to: &'static str,
    ) -> Result<Tranresult, Box<dyn Error + Send + Sync>>;
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Tranresult {
    error_code: Option<i32>,
    query: String,
    translation: Vec<String>,
    basic: Basic,
    web: Vec<Web>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Basic {
    phonetic: String,
    explains: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Web {
    key: String,
    value: Vec<String>,
}
impl Tranresult {
    pub fn pretty(&self) -> String {
        let result = format!(
            "{:?}\n\n\
            直译:{:?}\n\
            发音:{:?}\n\
            词典:{:?}\n",
            self.query, self.translation, self.basic.phonetic, self.basic.explains
        );
        result
    }
    pub fn markdown(&self) -> String {
        String::new()
    }
    pub fn html(&self) -> String {
        String::new()
    }
    pub fn translation(&self) -> String {
        format!("{:?}", self.translation)
    }
}
