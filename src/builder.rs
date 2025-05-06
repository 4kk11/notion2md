use crate::{converters::Converters, notion_to_md::NotionToMarkdown, types::ConfigurationOptions};
use notion_client::endpoints::Client;

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
        NotionToMarkdown::new(self.client, self.config, self.converters)
    }
}
