use std::error::Error;

use async_trait::async_trait;
use image::DynamicImage;
use teloxide::net::Download;
use teloxide::prelude::{Message, Requester};
use teloxide::types::MessageEntityKind;
use tracing::warn;
use url::Url;

use crate::utils::parse_url;
use crate::BotType;

#[async_trait]
pub trait MessageExt {
    fn links(&self) -> Vec<Url>;
    async fn image(
        &self,
        bot: &BotType,
    ) -> Result<Option<DynamicImage>, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl MessageExt for Message {
    fn links(&self) -> Vec<Url> {
        self.parse_entities().map_or(vec![], |entities| {
            entities
                .iter()
                .filter_map(|entity| match entity.kind() {
                    MessageEntityKind::Url => parse_url(entity.text()).map_or_else(
                        |_| {
                            warn!(url=%entity.text(), "invalid url");
                            None
                        },
                        Some,
                    ),
                    MessageEntityKind::TextLink { url } => Some(url.clone()),
                    _ => None,
                })
                .collect()
        })
    }
    async fn image(
        &self,
        bot: &BotType,
    ) -> Result<Option<DynamicImage>, Box<dyn Error + Send + Sync>> {
        Ok(
            match self.photo().and_then(|photos| {
                photos
                    .iter()
                    .max_by_key(|photo| photo.height * photo.width)
                    .map(move |photo| async move {
                        // download photo here
                        let file = bot.get_file(photo.file.id.clone()).await?;
                        let mut buf = vec![];
                        bot.download_file(&file.path, &mut buf).await?;
                        let img =
                            tokio::task::spawn_blocking(move || image::load_from_memory(&buf))
                                .await??;
                        Ok::<_, Box<dyn Error + Send + Sync>>(img)
                    })
            }) {
                Some(f) => Some(f.await?),
                None => None,
            },
        )
    }
}
