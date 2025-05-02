use anyhow::Result;
use futures::future::BoxFuture;
use notion_client::{endpoints::Client, objects::{block::{Block, BlockType}, file::File}};
use crate::utils;

use super::types::*;


#[derive(Debug)]
pub struct BlockWithChildren {
    block: Block,
    children: Vec<BlockWithChildren>,
}
pub struct NotionToMarkdown {
    client: Client,
    config: ConfigurationOptions,
}

impl NotionToMarkdown {
    pub fn new(notion_client: Client, config: ConfigurationOptions) -> Self {
        NotionToMarkdown {
            client: notion_client,
            config,
        }
    }

    pub async fn convert_page(&self, page_id: &str) -> Result<String> {
        let blocks = self.get_block_children_recursively(page_id).await?;
        let content = NotionToMarkdown::convert_blocks_to_markdown(&blocks)?;
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

    pub fn convert_blocks_to_markdown(blocks: &[BlockWithChildren]) -> Result<String> {
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
            markdown.push_str(&NotionToMarkdown::convert_block_to_markdown(block, &mut list_context)?);
            prev_block_type = Some(block.block.block_type.clone());
        }

        Ok(markdown)
    }

    pub fn convert_block_to_markdown(
        block_with_children: &BlockWithChildren,
        list_context: &mut ListContext,
    ) -> Result<String> {
        let block = &block_with_children.block;
        let children = &block_with_children.children;

        match &block.block_type {
            BlockType::Paragraph { paragraph } => {
                let text = NotionToMarkdown::rich_text_to_markdown(&paragraph.rich_text);
                if text.trim().is_empty() {
                    Ok(String::from("\n"))
                } else {
                    Ok(format!("{}\n", text))
                }
            }
            BlockType::Heading1 { heading_1 } => {
                let text = NotionToMarkdown::rich_text_to_markdown(&heading_1.rich_text);
                Ok(format!("{}\n", utils::heading1(&text)))
            }
            BlockType::Heading2 { heading_2 } => {
                let text = NotionToMarkdown::rich_text_to_markdown(&heading_2.rich_text);
                Ok(format!("{}\n", utils::heading2(&text)))
            }
            BlockType::Heading3 { heading_3 } => {
                let text = NotionToMarkdown::rich_text_to_markdown(&heading_3.rich_text);
                Ok(format!("{}\n", utils::heading3(&text)))
            }
            BlockType::BulletedListItem { bulleted_list_item } => {
                let text = NotionToMarkdown::rich_text_to_markdown(&bulleted_list_item.rich_text);
                let mut content = format!("{}\n", utils::bullet(&text, None));

                if !children.is_empty() {
                    let child_content = NotionToMarkdown::convert_blocks_to_markdown(children)?;
                    let indented_content = child_content
                        .replace("\n\n", "\n")
                        .lines()
                        .map(|line| format!("  {}", line))
                        .collect::<Vec<_>>()
                        .join("\n");
                    if !indented_content.is_empty() {
                        content.push_str(&format!("{}\n", indented_content));
                    }
                }

                Ok(content)
            }
            BlockType::NumberedListItem { numbered_list_item } => {
                let text = NotionToMarkdown::rich_text_to_markdown(&numbered_list_item.rich_text);
                let number = list_context.next_number();
                let mut content = format!("{}\n", utils::bullet(&text, Some(number)));

                if !children.is_empty() {
                    list_context.push();
                    let child_content = NotionToMarkdown::convert_blocks_to_markdown(children)?;
                    list_context.pop();

                    let indented_content = child_content
                        .replace("\n\n", "\n")
                        .lines()
                        .map(|line| format!("  {}", line))
                        .collect::<Vec<_>>()
                        .join("\n");
                    if !indented_content.is_empty() {
                        content.push_str(&format!("{}\n", indented_content));
                    }
                }

                Ok(content)
            }
            BlockType::ToDo { to_do } => {
                let text = NotionToMarkdown::rich_text_to_markdown(&to_do.rich_text);
                Ok(format!("{}\n", utils::todo(&text, to_do.checked.unwrap_or_default())))
            }
            BlockType::Toggle { toggle } => {
                let text = NotionToMarkdown::rich_text_to_markdown(&toggle.rich_text);
                let mut content = format!("{}\n", utils::bullet(&text, None));

                if !children.is_empty() {
                    let child_content = NotionToMarkdown::convert_blocks_to_markdown(children)?;
                    let indented_content = child_content
                        .replace("\n\n", "\n")
                        .lines()
                        .map(|line| format!("  {}", line))
                        .collect::<Vec<_>>()
                        .join("\n");
                    if !indented_content.is_empty() {
                        content.push_str(&format!("{}\n", indented_content));
                    }
                }

                Ok(content)
            }
            BlockType::Quote { quote } => {
                let text = NotionToMarkdown::rich_text_to_markdown(&quote.rich_text);
                let mut content = text
                    .lines()
                    .map(|line| format!("{}\n", utils::quote(line)))
                    .collect::<String>();

                if !children.is_empty() {
                    let child_content = NotionToMarkdown::convert_blocks_to_markdown(children)?;
                    let formatted_content = child_content
                        .lines()
                        .map(|line| utils::quote(line))
                        .collect::<Vec<_>>()
                        .join("\n");
                    if !formatted_content.is_empty() {
                        content.push_str(&format!("{}\n", formatted_content));
                    }
                }

                content.push('\n');
                Ok(content)
            }
            BlockType::Code { code } => {
                let text = NotionToMarkdown::rich_text_to_markdown(&code.rich_text);
                let language = format!("{:?}", code.language).to_lowercase();
                Ok(format!("{}\n", utils::code_block(&text, Some(&language))))
            }
            BlockType::Callout { callout } => {
                let text = NotionToMarkdown::rich_text_to_markdown(&callout.rich_text);
                let mut content = format!("> [!note] {}\n", text);
                // let mut content = format!("{}\n", utils::callout(&text, None));

                if !children.is_empty() {
                    let child_content = NotionToMarkdown::convert_blocks_to_markdown(children)?;
                    let formatted_content = child_content
                        .lines()
                        .filter(|line| !line.contains(&text))
                        .map(|line| format!("> {}", line))
                        .collect::<Vec<_>>()
                        .join("\n");
                    if !formatted_content.is_empty() {
                        content.push_str(&format!("{}\n", formatted_content));
                    }
                }

                content.push('\n');
                Ok(content)
            }
            BlockType::Image { image } => {
                let url = Self::get_file_url(&image.file_type);
                Ok(format!("![]({})\n\n", url))
            }
            BlockType::Video { video } => {
                let url = Self::get_file_url(&video.file_type);
                Ok(format!("![]({})\n\n", url))
            }
            BlockType::Bookmark { bookmark } => {
                Ok(format!("[{}]({})\n\n", bookmark.url, bookmark.url))
            }
            BlockType::LinkPreview { link_preview } => {
                Ok(format!("[{}]({})\n\n", link_preview.url, link_preview.url))
            }
            BlockType::Divider { .. } => Ok("---\n\n".to_string()),
            BlockType::Table { table: _ } => {
                let mut content = String::new();

                if !children.is_empty() {
                    if let Some(first_row) = children.first() {
                        if let BlockType::TableRow { table_row } = &first_row.block.block_type {
                            content.push('|');
                            for cell in &table_row.cells {
                                let cell_text = NotionToMarkdown::rich_text_to_markdown(cell);
                                content.push_str(&format!(" {} |", cell_text));
                            }
                            content.push('\n');

                            content.push('|');
                            for _ in 0..table_row.cells.len() {
                                content.push_str(" --- |");
                            }
                            content.push('\n');

                            for row in children.iter().skip(1) {
                                if let BlockType::TableRow { table_row } = &row.block.block_type {
                                    content.push('|');
                                    for cell in &table_row.cells {
                                        let cell_text = NotionToMarkdown::rich_text_to_markdown(cell);
                                        content.push_str(&format!(" {} |", cell_text));
                                    }
                                    content.push('\n');
                                }
                            }
                        }
                    }
                }
                content.push('\n');
                Ok(content)
            }
            BlockType::Embed { embed } => Ok(format!(
                "<iframe src=\"{}\" width=\"100%\" height=\"500px\"></iframe>\n\n",
                embed.url
            )),
            _ => {
                log::warn!("Unsupported block type: {:?}", block.block_type);
                if !children.is_empty() {
                    NotionToMarkdown::convert_blocks_to_markdown(children)
                } else {
                    Ok(String::new())
                }
            }
        }
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


    fn get_file_url(file: &File) -> String {
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
    fn new() -> Self {
        Self { counters: vec![0] }
    }

    fn next_number(&mut self) -> usize {
        let current_level = self.counters.len() - 1;
        self.counters[current_level] += 1;
        self.counters[current_level]
    }

    fn push(&mut self) {
        self.counters.push(0);
    }

    fn pop(&mut self) {
        self.counters.pop();
        if self.counters.is_empty() {
            self.counters.push(0);
        }
    }
}