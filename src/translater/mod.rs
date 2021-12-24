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
        String::new()
    }
    pub fn markdown(&self) -> String {
        let mut translation = String::new();
        for v in self.translation.iter() {
            translation.push_str(&format!("_{}_", v));
        }
        let mut explains = String::new();
        for (i, v) in self.basic.explains.iter().enumerate() {
            explains.push_str(&format!("\n*{}.* {}", i + 1, v));
        }

        let result = format!(
            "🌟 __*{}*__ 🌟\n\n\
            🥇 *直译：* {}\n\n\
            🥈 *发音：* \\[{}\\]\n\n\
            🥉 *词典：*——————————————————————————{}\n———————————————————————————————\n",
            self.query, translation, self.basic.phonetic, explains
        );

        let result = result.replace('.', "\\.");
        let result = result.replace('<', "\\<");
        let result = result.replace('>', "\\>");
        let result = result.replace('(', "\\(");
        let result = result.replace(')', "\\)");

        result
    }
    pub fn html(&self) -> String {
        String::new()
    }
    pub fn translation(&self) -> String {
        format!("{:?}", self.translation)
    }
    pub fn explains(&self) -> String {
        format!("{}", self.basic.explains[0])
    }
}
