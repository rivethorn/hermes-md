#!/usr/bin/env bun

import { Command } from "commander";
import dotenv from "dotenv";
import { publish } from "./publish";
import { deletePost } from "./delete";
import { genConfig, loadConfig } from "./config";
import { listItems } from "./list";

dotenv.config();

const program = new Command();

program
  .name("hermes")
  .description("Publish markdown posts to Supabase (storage + posts table)")
  .option("--config <path>", "Path to config file");

program
  .command("publish")
  .argument("<path>", "Markdown file")
  .action(async (filePath, opts) => {
    const cfg = await loadConfig(program.opts().config);
    await publish(cfg, filePath);
  });

program
  .command("delete")
  .argument("<slug>", "Post slug")
  .option("--soft", "Keep storage file")
  .action(async (slug, options) => {
    const cfg = await loadConfig(program.opts().config);
    await deletePost(cfg, slug, options.soft);
  });

program.command("list").action(async () => {
  const cfg = await loadConfig(program.opts().config || "./config.toml");
  await listItems(cfg);
});

program.command("gen-config").action(async () => {
  const p = await genConfig();
  console.log(`Sample config written to ${p}`);
});

program.parse();
