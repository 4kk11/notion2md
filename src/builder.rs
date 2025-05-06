use std::sync::Arc;

use anyhow::Result;
use notion_client::{endpoints::Client, objects::block::ParagraphValue};

use crate::{converters::Converters, notion_to_md::{BlockWithChildren, ListContext, NotionToMarkdown}, types::ConfigurationOptions};




pub struct NotionToMarkdownBuilder {
    client: Client,
    config: ConfigurationOptions,
    pub converters: Converters,
}

impl NotionToMarkdownBuilder {
    pub fn new(client: Client, config: ConfigurationOptions) -> Self {
        Self {
            client,
            config,
            converters: Converters::default(),
        }
    }

    pub fn build(self) -> NotionToMarkdown {
        NotionToMarkdown::new(
            self.client,
            self.config,
            self.converters,
        )
    }

}
