use crate::notion_to_md::{BlockWithChildren, ListContext, NotionToMarkdown};
use notion_client::objects::block::*;

// 可読性向上用。Result は anyhow::Result でも独自型でも可。
type ConvResult = anyhow::Result<String>;

// 各コンバータに渡す共通のペイロード
pub struct ConvFuncPayload<'a, T> {
    pub value: &'a T,
    pub children: &'a [BlockWithChildren],
    pub list_ctx: &'a mut ListContext,
    pub owner: &'a NotionToMarkdown,
}

// 共通クロージャ型（ジェネリック T に実際のブロック構造体を入れる）
type ConvFn<T> = dyn for<'a> Fn(ConvFuncPayload<'a, T>) -> ConvResult + Send + Sync;

mod default_conv {
    use notion_client::objects::block::*;

    use super::ConvFuncPayload;
    use crate::{notion_to_md::NotionToMarkdown, utils};

    pub fn paragraph(payload: ConvFuncPayload<'_, ParagraphValue>) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&payload.value.rich_text);
        if text.trim().is_empty() {
            Ok(String::from("\n"))
        } else {
            Ok(format!("{}\n", text))
        }
    }

    pub fn heading_1(payload: ConvFuncPayload<'_, HeadingsValue>) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&payload.value.rich_text);
        Ok(format!("{}\n", utils::heading1(&text)))
    }

    pub fn heading_2(payload: ConvFuncPayload<'_, HeadingsValue>) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&payload.value.rich_text);
        Ok(format!("{}\n", utils::heading2(&text)))
    }

    pub fn heading_3(payload: ConvFuncPayload<'_, HeadingsValue>) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&payload.value.rich_text);
        Ok(format!("{}\n", utils::heading3(&text)))
    }

    pub fn bulleted_list_item(
        payload: ConvFuncPayload<'_, BulletedListItemValue>,
    ) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&payload.value.rich_text);
        let mut content = format!("{}\n", utils::bullet(&text, None));

        if !payload.children.is_empty() {
            let child_content = payload.owner.convert_blocks_to_markdown(payload.children)?;
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

    pub fn numbered_list_item(
        payload: ConvFuncPayload<'_, NumberedListItemValue>,
    ) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&payload.value.rich_text);
        let number = payload.list_ctx.next_number();
        let mut content = format!("{}\n", utils::bullet(&text, Some(number)));

        if !payload.children.is_empty() {
            payload.list_ctx.push();
            let child_content = payload.owner.convert_blocks_to_markdown(payload.children)?;
            payload.list_ctx.pop();

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

    pub fn to_do(payload: ConvFuncPayload<'_, ToDoValue>) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&payload.value.rich_text);
        Ok(format!(
            "{}\n",
            utils::todo(&text, payload.value.checked.unwrap_or_default())
        ))
    }

    pub fn toggle(payload: ConvFuncPayload<'_, ToggleValue>) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&payload.value.rich_text);
        let mut content = format!("{}\n", utils::bullet(&text, None));

        if !payload.children.is_empty() {
            let child_content = payload.owner.convert_blocks_to_markdown(payload.children)?;
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

    pub fn quote(payload: ConvFuncPayload<'_, QuoteValue>) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&payload.value.rich_text);
        let mut content = text
            .lines()
            .map(|line| format!("{}\n", utils::quote(line)))
            .collect::<String>();

        if !payload.children.is_empty() {
            let child_content = payload.owner.convert_blocks_to_markdown(payload.children)?;
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

    pub fn code(payload: ConvFuncPayload<'_, CodeValue>) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&payload.value.rich_text);
        let language = format!("{:?}", payload.value.language).to_lowercase();
        Ok(format!("{}\n", utils::code_block(&text, Some(&language))))
    }

    pub fn callout(payload: ConvFuncPayload<'_, CalloutValue>) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&payload.value.rich_text);
        let mut content = format!("> [!note] {}\n", text);

        if !payload.children.is_empty() {
            let child_content = payload.owner.convert_blocks_to_markdown(payload.children)?;
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

    pub fn image(payload: ConvFuncPayload<'_, ImageValue>) -> anyhow::Result<String> {
        let url = NotionToMarkdown::get_file_url(&payload.value.file_type);
        Ok(format!("![]({})\n\n", url))
    }

    pub fn video(payload: ConvFuncPayload<'_, VideoValue>) -> anyhow::Result<String> {
        let url = NotionToMarkdown::get_file_url(&payload.value.file_type);
        Ok(format!("![]({})\n\n", url))
    }

    pub fn bookmark(payload: ConvFuncPayload<'_, BookmarkValue>) -> anyhow::Result<String> {
        Ok(format!(
            "[{}]({})\n\n",
            payload.value.url, payload.value.url
        ))
    }

    pub fn link_preview(payload: ConvFuncPayload<'_, LinkPreviewValue>) -> anyhow::Result<String> {
        Ok(format!(
            "[{}]({})\n\n",
            payload.value.url, payload.value.url
        ))
    }

    pub fn divider(_payload: ConvFuncPayload<'_, DividerValue>) -> anyhow::Result<String> {
        Ok("---\n\n".to_string())
    }

    pub fn table(payload: ConvFuncPayload<'_, TableValue>) -> anyhow::Result<String> {
        let mut content = String::new();

        if !payload.children.is_empty() {
            if let Some(first_row) = payload.children.first() {
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

                    for row in payload.children.iter().skip(1) {
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

    pub fn embed(payload: ConvFuncPayload<'_, EmbedValue>) -> anyhow::Result<String> {
        Ok(format!(
            "<iframe src=\"{}\" width=\"100%\" height=\"500px\"></iframe>\n\n",
            payload.value.url
        ))
    }
}

//==================== 1. マクロ本体 ====================
macro_rules! define_converters {
    (
        $( ($Variant:ident, $field:ident, $Payload:ty) ),+ $(,)?
    ) => {
        use crate::builder::NotionToMarkdownBuilder;
        use notion_client::objects::block::BlockType;

        // ① Converters 構造体
        pub struct Converters {
            $( pub $field: std::sync::Arc<ConvFn<$Payload>>, )+
        }

        impl Default for Converters {
            fn default() -> Self {
                Self {
                    $( $field: std::sync::Arc::new(default_conv::$field), )+
                }
            }
        }

        // ② Builder 差し替え
        impl NotionToMarkdownBuilder {
            $(
            pub fn $field<F>(mut self, f: F) -> Self
            where
                F: for<'a> Fn(ConvFuncPayload<'a, $Payload>) -> ConvResult
                   + Send + Sync + 'static,
            {
                self.converters.$field = std::sync::Arc::new(f);
                self
            })+
        }

        // ③ dispatch
        impl NotionToMarkdown {
            pub fn convert_block_to_markdown_inner(
                &self,
                bwc: &BlockWithChildren,
                ctx: &mut ListContext,
            ) -> ConvResult {
                match &bwc.block.block_type {
                    $(
                    BlockType::$Variant { $field: inner } => {
                        (self.converters.$field)(
                            ConvFuncPayload {
                                value: inner,
                                children: &bwc.children,
                                list_ctx: ctx,
                                owner: self,
                            }
                        )
                    }
                    )+
                    _ => {
                        log::warn!("Unsupported block type: {:?}", bwc.block.block_type);
                        Ok(String::new())
                    }
                }
            }
        }
    };
}

//==================== 2. たった 1 行で宣言 ====================
define_converters! {
    // Variant       field_ident        payload_type
    (Paragraph,      paragraph,         ParagraphValue),
    (Heading1,       heading_1,         HeadingsValue),
    (Heading2,       heading_2,         HeadingsValue),
    (Heading3,       heading_3,         HeadingsValue),
    (BulletedListItem, bulleted_list_item,   BulletedListItemValue),
    (NumberedListItem, numbered_list_item,   NumberedListItemValue),
    (ToDo,          to_do,             ToDoValue),
    (Toggle,        toggle,           ToggleValue),
    (Quote,         quote,            QuoteValue),
    (Code,          code,             CodeValue),
    (Callout,       callout,          CalloutValue),
    (Image,         image,            ImageValue),
    (Video,         video,            VideoValue),
    (Bookmark,      bookmark,         BookmarkValue),
    (LinkPreview,   link_preview,      LinkPreviewValue),
    (Divider,       divider,          DividerValue),
    (Table,         table,            TableValue),
    (Embed,         embed,            EmbedValue),
}
