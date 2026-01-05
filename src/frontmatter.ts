import yaml from "js-yaml";
import type { FrontMatter } from "./types";

export function parseFrontmatter(input: string): {
  frontmatter?: FrontMatter;
  body: string;
} {
  const trimmed = input.trimStart();
  if (!trimmed.startsWith("---")) {
    return { body: input };
  }

  const [, yamlBlock, rest] = trimmed.split(/---\s*/s);
  const fm = yaml.load(yamlBlock || "this is not good") as FrontMatter;

  return { frontmatter: fm, body: rest ?? "" };
}
