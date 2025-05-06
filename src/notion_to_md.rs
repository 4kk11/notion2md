use crate::converters::Converters;
use anyhow::Result;
use futures::future::BoxFuture;
use notion_client::{
    endpoints::Client,
    objects::{
        block::{Block, BlockType},
        file::File,
    },
};

#[derive(Debug)]
pub struct BlockWithChildren {
    pub block: Block,
    pub children: Vec<BlockWithChildren>,
}
pub struct NotionToMarkdown {
    client: Client,
    pub converters: Converters,
}

impl NotionToMarkdown {
    pub fn new(
        notion_client: Client,
        converters: Converters,
    ) -> Self {
        NotionToMarkdown {
            client: notion_client,
            converters,
        }
    }

    pub async fn convert_page(&self, page_id: &str) -> Result<String> {
        let blocks = self.get_block_children_recursively(page_id).await?;
        let content = self.convert_blocks_to_markdown(&blocks)?;
        Ok(content)
    }

    fn get_block_children_recursively<'a>(
        &'a self,
        block_id: &'a str,
    ) -> BoxFuture<'a, Result<Vec<BlockWithChildren>>> {
        Box::pin(async move {
            let mut blocks = Vec::new();
            let mut start_cursor = None;

            loop {
                let response = self
                    .client
                    .blocks
                    .retrieve_block_children(block_id, start_cursor.as_deref(), None)
                    .await?;
                // .map_err(|e| NotionToObsidianError::BlockRetrievalError(e.to_string()))?;

                for block in response.results {
                    let children = if block.has_children.unwrap_or(false) {
                        if let Some(id) = &block.id {
                            self.get_block_children_recursively(id).await?
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    };

                    blocks.push(BlockWithChildren { block, children });
                }

                if !response.has_more {
                    break;
                }
                start_cursor = response.next_cursor;
            }

            Ok(blocks)
        })
    }

    pub fn convert_blocks_to_markdown(&self, blocks: &[BlockWithChildren]) -> Result<String> {
        let mut markdown = String::new();
        let mut list_context = ListContext::new();
        let mut prev_block_type = None;

        for block in blocks {
            if let Some(prev_type) = &prev_block_type {
                if !matches!(prev_type, &BlockType::NumberedListItem { .. })
                    && matches!(&block.block.block_type, BlockType::NumberedListItem { .. })
                {
                    list_context = ListContext::new();
                }
            }
            markdown.push_str(&self.convert_block_to_markdown_inner(block, &mut list_context)?);
            prev_block_type = Some(block.block.block_type.clone());
        }

        Ok(markdown)
    }

    pub fn rich_text_to_markdown(
        rich_text: &[notion_client::objects::rich_text::RichText],
    ) -> String {
        if rich_text.is_empty() {
            return String::new();
        }

        let mut markdown = String::new();

        for text in rich_text {
            let mut content = match text {
                notion_client::objects::rich_text::RichText::Text {
                    text, plain_text, ..
                } => {
                    let text_content = plain_text
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or(&text.content);
                    if let Some(link) = &text.link {
                        format!("[{}]({})", text_content, link.url)
                    } else {
                        text_content.to_string()
                    }
                }
                notion_client::objects::rich_text::RichText::Mention { plain_text, .. } => {
                    plain_text.clone()
                }
                notion_client::objects::rich_text::RichText::Equation { plain_text, .. } => {
                    plain_text.clone()
                }
                notion_client::objects::rich_text::RichText::None => String::new(),
            };

            if let Some(annotations) = match text {
                notion_client::objects::rich_text::RichText::Text { annotations, .. } => {
                    annotations.clone()
                }
                notion_client::objects::rich_text::RichText::Mention { annotations, .. } => {
                    Some(annotations.clone())
                }
                notion_client::objects::rich_text::RichText::Equation { annotations, .. } => {
                    Some(annotations.clone())
                }
                notion_client::objects::rich_text::RichText::None => None,
            } {
                if annotations.bold {
                    content = format!("**{}**", content);
                }
                if annotations.italic {
                    content = format!("*{}*", content);
                }
                if annotations.strikethrough {
                    content = format!("~~{}~~", content);
                }
                if annotations.code {
                    content = format!("`{}`", content);
                }
            }

            markdown.push_str(&content);
        }

        markdown
    }

    pub fn get_file_url(file: &File) -> String {
        match file {
            File::External { external } => external.url.clone(),
            File::File { file } => file.url.clone(),
        }
    }
}

#[derive(Default)]
pub struct ListContext {
    counters: Vec<usize>,
}

impl ListContext {
    pub fn new() -> Self {
        Self { counters: vec![0] }
    }

    pub fn next_number(&mut self) -> usize {
        let current_level = self.counters.len() - 1;
        self.counters[current_level] += 1;
        self.counters[current_level]
    }

    pub fn push(&mut self) {
        self.counters.push(0);
    }

    pub fn pop(&mut self) {
        self.counters.pop();
        if self.counters.is_empty() {
            self.counters.push(0);
        }
    }
}
