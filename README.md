# notion2md

NotionのページをMarkdownに変換するためのRustライブラリです。Notion APIを使用してページの内容を取得し、様々なブロックタイプに対応したMarkdown形式に変換します。

## 特徴

- 豊富なブロックタイプのサポート
  - 見出し（H1, H2, H3）
  - リスト（箇条書き、番号付き）
  - TODOリスト
  - トグル（折りたたみ可能なセクション）
  - 引用
  - コードブロック（言語シンタックスハイライト対応）
  - コールアウト（ノート形式）
  - メディア（画像、動画）
  - ブックマーク、リンクプレビュー
  - 区切り線
  - テーブル
  - 埋め込みコンテンツ
- ブロックの入れ子構造のサポート
- テキスト装飾（太字、斜体、取り消し線、コード）対応
- 非同期処理による効率的なページ取得
- カスタマイズ可能な変換処理

## インストール

```toml
[dependencies]
notion2md = "0.1.0"
```

## 使用方法

### 基本的な使用方法

```rust
use notion2md::builder::NotionToMarkdownBuilder;
use notion_client::endpoints::Client;
use anyhow::Result;

async fn convert_page() -> Result<String> {
    // Notion APIトークンを環境変数から取得
    let notion_token = std::env::var("NOTION_TOKEN")?;
    
    // Notion APIクライアントを初期化
    let notion_client = Client::new(notion_token, None)?;
    
    // コンバーターをビルド
    let converter = NotionToMarkdownBuilder::new(notion_client)
        .build();
    
    // ページを変換
    let page_id = "your-page-id";
    let markdown = converter.convert_page(page_id).await?;
    
    Ok(markdown)
}
```

### カスタムコンバーターの使用

各ブロックタイプの変換方法をカスタマイズできます：

```rust
use notion2md::builder::NotionToMarkdownBuilder;
use notion_client::endpoints::Client;

async fn custom_converter() -> Result<()> {
    let notion_client = Client::new(notion_token, None)?;
    
    // カスタムコンバーターを設定
    let converter = NotionToMarkdownBuilder::new(notion_client)
        .heading_1(|payload| {
            let text = NotionToMarkdown::rich_text_to_markdown(&payload.value.rich_text);
            Ok(format!("# {}\n", text))
        })
        .code(|payload| {
            let text = NotionToMarkdown::rich_text_to_markdown(&payload.value.rich_text);
            let lang = format!("{:?}", payload.value.language).to_lowercase();
            Ok(format!("```{}\n{}\n```\n", lang, text))
        })
        .build();
    
    // 変換を実行
    let markdown = converter.convert_page(page_id).await?;
    Ok(())
}
```

### ファイルへの保存

```rust
use std::fs;
use std::path::PathBuf;

async fn save_to_file() -> Result<()> {
    let markdown = converter.convert_page(page_id).await?;
    
    // 出力ディレクトリを作成
    let output_dir = PathBuf::from("output");
    fs::create_dir_all(&output_dir)?;
    
    // ファイルに保存
    let output_path = output_dir.join("page.md");
    fs::write(output_path, markdown)?;
    
    Ok(())
}
```

## ライセンス

MIT License
