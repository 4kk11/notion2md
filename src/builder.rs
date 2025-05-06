use crate::{converters::Converters, notion_to_md::NotionToMarkdown};
use notion_client::endpoints::Client;

pub struct NotionToMarkdownBuilder {
    client: Client,
    pub converters: Converters,
}

impl NotionToMarkdownBuilder {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            converters: Converters::default(),
        }
    }

    pub fn build(self) -> NotionToMarkdown {
        NotionToMarkdown::new(self.client, self.converters)
    }
}
