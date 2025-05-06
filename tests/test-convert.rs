use env_logger;
use log::info;
use notion_to_md_rs::builder::NotionToMarkdownBuilder;
use notion_to_md_rs::converters::Converters;
use notion_to_md_rs::notion_to_md::NotionToMarkdown;
use notion_client::endpoints::Client;
use notion_to_md_rs::types::ConfigurationOptions;
use std::fs;
use anyhow::Result;
use std::path::PathBuf;
use std::time::Instant;
use tokio;

const TEST_PAGE_ID: &str = "1aeb266e0c708060a6fec6eb458e1379";
const TEST_OUTPUT_PAGE_TITLE: &str = "test";
const TEST_OUTPUT_DIR: &str = "target/test_output";

#[tokio::test]
async fn test_page_conversion() -> Result<()> {
    // ロガーを初期化（テスト用に強制的にInfo以上のログを出力）
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .init();

    let start_total = Instant::now();
    info!("テスト開始");

    dotenv::dotenv().ok();

    // Notionトークンを環境変数から取得
    let notion_token = std::env::var("NOTION_TOKEN").expect("NOTION_TOKEN must be set");

    // Obsidianの出力ディレクトリを一時ディレクトリとして設定
    let start_setup = Instant::now();
    let obsidian_dir = PathBuf::from(TEST_OUTPUT_DIR);
    std::fs::create_dir_all(&obsidian_dir).expect("Failed to create test output directory");

    let notion_client = Client::new(notion_token.clone(), None)?;

    // NotionToMarkdownインスタンスを作成
    let converter = NotionToMarkdownBuilder::new(
        notion_client,
        ConfigurationOptions::default(),
    )
    .build();

    // NotionToObsidianインスタンスを作成
    info!("セットアップ完了: {:?}", start_setup.elapsed());

    // テスト対象のページを変換
    let start_conversion = Instant::now();
    let markdown_text = converter.convert_page(TEST_PAGE_ID).await?;
    info!("ページ変換完了: {:?}", start_conversion.elapsed());

    // 変換したMarkdownをファイルに保存
    let output_file_path = obsidian_dir.join(format!("{}.md", TEST_OUTPUT_PAGE_TITLE));
    fs::write(&output_file_path, markdown_text).expect("Failed to write test_output.md");
    info!("Markdownファイルの保存完了: {:?}", start_conversion.elapsed());

    // テストケースのファイルを読み込み
    let expected_content =
        fs::read_to_string("cases/test_expected.md").expect("Failed to read test.md");

    // 生成したファイルを読み込み
    let converted_content =
        fs::read_to_string(format!("{}/{}.md", TEST_OUTPUT_DIR, TEST_OUTPUT_PAGE_TITLE))
            .expect("Failed to read test_output.md");

    // 変換結果と期待される結果を比較
    // 改行コードを正規化して比較（WindowsとUnixの改行の違いを吸収）
    let normalized_converted = converted_content.replace("\r\n", "\n");
    let normalized_expected = expected_content.replace("\r\n", "\n");

    assert_eq!(
        normalized_converted.trim(),
        normalized_expected.trim(),
        "Converted content does not match expected content"
    );

    info!("テスト合計実行時間: {:?}", start_total.elapsed());
    Ok(())
}
