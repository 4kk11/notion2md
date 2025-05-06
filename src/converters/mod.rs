use crate::notion_to_md::{BlockWithChildren, ListContext};
use notion_client::objects::block::*;

// 可読性向上用。Result は anyhow::Result でも独自型でも可。
type ConvResult = anyhow::Result<String>;

// 共通クロージャ型（ジェネリック T に実際のブロック構造体を入れる）
type ConvFn<T> =
    dyn Fn(&T, /* children */ &[BlockWithChildren],
           /* list state */ &mut ListContext,
           /* cfg */ &NotionToMarkdown)
       -> ConvResult
    + Send + Sync;

mod default_conv {
    use notion_client::objects::block::{BlockType, BulletedListItemValue, HeadingsValue, NumberedListItemValue, ParagraphValue, ToDoValue, ToggleValue};

    use crate::{notion_to_md::{BlockWithChildren, ListContext, NotionToMarkdown}, utils};

    pub fn paragraph(
        _paragraph: &ParagraphValue,
        _children: &[BlockWithChildren],
        _list_context: &mut ListContext,
        _owner: &NotionToMarkdown,
    ) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&_paragraph.rich_text);
        if text.trim().is_empty() {
            Ok(String::from("\n"))
        } else {
            Ok(format!("{}\n", text))
        }
    }

    pub fn heading_1(
        _heading_1: &HeadingsValue,
        _children: &[BlockWithChildren],
        _list_context: &mut ListContext,
        _owner: &NotionToMarkdown,
    ) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&_heading_1.rich_text);
        Ok(format!("{}\n", utils::heading1(&text)))
    }

    pub fn heading_2(
        _heading_2: &HeadingsValue,
        _children: &[BlockWithChildren],
        _list_context: &mut ListContext,
        _owner: &NotionToMarkdown,
    ) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&_heading_2.rich_text);
        Ok(format!("{}\n", utils::heading2(&text)))
    }

    pub fn heading_3(
        _heading_3: &HeadingsValue,
        _children: &[BlockWithChildren],
        _list_context: &mut ListContext,
        _owner: &NotionToMarkdown,
    ) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&_heading_3.rich_text);
        Ok(format!("{}\n", utils::heading3(&text)))
    }

    pub fn bulleted_list_item(
        _bulleted_list: &BulletedListItemValue,
        _children: &[BlockWithChildren],
        _list_context: &mut ListContext,
        _owner: &NotionToMarkdown,
    ) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&_bulleted_list.rich_text);
        let mut content = format!("{}\n", utils::bullet(&text, None));

        if !_children.is_empty() {
            let child_content = _owner.convert_blocks_to_markdown(_children)?;
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
        _numbered_list: &NumberedListItemValue,
        _children: &[BlockWithChildren],
        _list_context: &mut ListContext,
        _owner: &NotionToMarkdown,
    ) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&_numbered_list.rich_text);
        let number = _list_context.next_number();
        let mut content = format!("{}\n", utils::bullet(&text, Some(number)));

        if !_children.is_empty() {
            _list_context.push();
            let child_content = _owner.convert_blocks_to_markdown(_children)?;
            _list_context.pop();

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


    pub fn to_do(
        _todo: &ToDoValue,
        _children: &[BlockWithChildren],
        _list_context: &mut ListContext,
        _owner: &NotionToMarkdown,
    ) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&_todo.rich_text);
        Ok(format!("{}\n", utils::todo(&text, _todo.checked.unwrap_or_default())))
    }


    pub fn toggle(
        _toggle: &ToggleValue,
        _children: &[BlockWithChildren],
        _list_context: &mut ListContext,
        _owner: &NotionToMarkdown,
    ) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&_toggle.rich_text);
        let mut content = format!("{}\n", utils::bullet(&text, None));

        if !_children.is_empty() {
            let child_content = _owner.convert_blocks_to_markdown(_children)?;
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

    pub fn quote(
        _quote: &notion_client::objects::block::QuoteValue,
        _children: &[BlockWithChildren],
        _list_context: &mut ListContext,
        _owner: &NotionToMarkdown,
    ) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&_quote.rich_text);
        let mut content = text
            .lines()
            .map(|line| format!("{}\n", utils::quote(line)))
            .collect::<String>();

        if !_children.is_empty() {
            let child_content = _owner.convert_blocks_to_markdown(_children)?;
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

    pub fn code(
        _code: &notion_client::objects::block::CodeValue,
        _children: &[BlockWithChildren],
        _list_context: &mut ListContext,
        _owner: &NotionToMarkdown,
    ) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&_code.rich_text);
        let language = format!("{:?}", _code.language).to_lowercase();
        Ok(format!("{}\n", utils::code_block(&text, Some(&language))))
    }

    pub fn callout(
        _callout: &notion_client::objects::block::CalloutValue,
        _children: &[BlockWithChildren],
        _list_context: &mut ListContext,
        _owner: &NotionToMarkdown,
    ) -> anyhow::Result<String> {
        let text = NotionToMarkdown::rich_text_to_markdown(&_callout.rich_text);
        let mut content = format!("> [!note] {}\n", text);
        // let mut content = format!("{}\n", utils::callout(&text, None));

        if !_children.is_empty() {
            let child_content = _owner.convert_blocks_to_markdown(_children)?;
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

    pub fn image(
        _image: &notion_client::objects::block::ImageValue,
        _children: &[BlockWithChildren],
        _list_context: &mut ListContext,
        _owner: &NotionToMarkdown,
    ) -> anyhow::Result<String> {
        let url = NotionToMarkdown::get_file_url(&_image.file_type);
        Ok(format!("![]({})\n\n", url))
    }

    pub fn video(
        _video: &notion_client::objects::block::VideoValue,
        _children: &[BlockWithChildren],
        _list_context: &mut ListContext,
        _owner: &NotionToMarkdown,
    ) -> anyhow::Result<String> {
        let url = NotionToMarkdown::get_file_url(&_video.file_type);
        Ok(format!("![]({})\n\n", url))
    }

    pub fn bookmark(
        _bookmark: &notion_client::objects::block::BookmarkValue,
        _children: &[BlockWithChildren],
        _list_context: &mut ListContext,
        _owner: &NotionToMarkdown,
    ) -> anyhow::Result<String> {
        Ok(format!("[{}]({})\n\n", _bookmark.url, _bookmark.url))
    }

    pub fn link_preview(
        _link_preview: &notion_client::objects::block::LinkPreviewValue,
        _children: &[BlockWithChildren],
        _list_context: &mut ListContext,
        _owner: &NotionToMarkdown,
    ) -> anyhow::Result<String> {
        Ok(format!("[{}]({})\n\n", _link_preview.url, _link_preview.url))
    }

    pub fn divider(
        _divider: &notion_client::objects::block::DividerValue,
        _children: &[BlockWithChildren],
        _list_context: &mut ListContext,
        _owner: &NotionToMarkdown,
    ) -> anyhow::Result<String> {
        Ok("---\n\n".to_string())
    }

    pub fn table(
        _table: &notion_client::objects::block::TableValue,
        _children: &[BlockWithChildren],
        _list_context: &mut ListContext,
        _owner: &NotionToMarkdown,
    ) -> anyhow::Result<String> {
        let mut content = String::new();

        if !_children.is_empty() {
            if let Some(first_row) = _children.first() {
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

                    for row in _children.iter().skip(1) {
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


    pub fn embed(
        _embed: &notion_client::objects::block::EmbedValue,
        _children: &[BlockWithChildren],
        _list_context: &mut ListContext,
        _owner: &NotionToMarkdown,
    ) -> anyhow::Result<String> {
        Ok(format!(
            "<iframe src=\"{}\" width=\"100%\" height=\"500px\"></iframe>\n\n",
            _embed.url
        ))
    }
}

//==================== 1. マクロ本体 ====================
macro_rules! define_converters {
    (
        $( ($Variant:ident, $field:ident, $Payload:ty) ),+ $(,)?
    ) => {

        use crate::builder::NotionToMarkdownBuilder;
        use crate::notion_to_md::NotionToMarkdown;
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
                F: Fn(&$Payload,
                      &[BlockWithChildren],
                      &mut ListContext,
                      &NotionToMarkdown) -> ConvResult
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
                        (self.converters.$field)(inner, &bwc.children, ctx, self)
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