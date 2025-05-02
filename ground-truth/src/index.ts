import { Client } from "@notionhq/client";
import { NotionToMarkdown } from "notion-to-md";
import fs from "fs";
import * as dotenv from "dotenv";
import { MdStringObject } from "notion-to-md/build/types";
dotenv.config();

const PAGE_ID = "1aeb266e0c708060a6fec6eb458e1379";

const notion = new Client({
  auth: process.env.NOTION_TOKEN,
});

const n2m = new NotionToMarkdown({ notionClient: notion });

(async () => {
    const mdBlocks = await n2m.pageToMarkdown(PAGE_ID);
    const mdString: MdStringObject = n2m.toMarkdownString(mdBlocks);
    const markdownText = Object.values(mdString).join("\n\n");
    fs.writeFileSync("output.md", markdownText, "utf-8");
    console.log("Markdown file created successfully!");
})();