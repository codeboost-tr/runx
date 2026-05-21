import { parseDocument } from "yaml";

import { isRecord } from "@runxhq/core/util";

interface SkillFrontmatter {
  readonly frontmatter: Record<string, unknown>;
}

export function parseSkillFrontmatter(markdown: string): SkillFrontmatter {
  const match = markdown.match(/^---\r?\n([\s\S]*?)\r?\n---\r?\n?[\s\S]*$/);
  if (!match) {
    throw new Error("Skill markdown must start with YAML frontmatter delimited by ---.");
  }

  const document = parseDocument(match[1], { prettyErrors: false });
  if (document.errors.length > 0) {
    throw new Error(document.errors.map((error) => error.message).join("; "));
  }

  const frontmatter = document.toJS();
  if (!isRecord(frontmatter)) {
    throw new Error("Skill frontmatter must parse to an object.");
  }

  return { frontmatter };
}
