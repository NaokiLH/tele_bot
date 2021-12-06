use async_trait::async_trait;
use std::error::Error;
pub mod baidu_api;
pub mod youdao_api;

#[async_trait]
pub trait Translater {
    async fn translate(
        &mut self,
        text: String,
        from: String,
        to: String,
    ) -> Result<String, Box<dyn Error + Send + Sync>>;
    async fn dictionary(
        &mut self,
        word: String,
        from: String,
        to: String,
    ) -> Result<String, Box<dyn Error + Send + Sync>>;
}
