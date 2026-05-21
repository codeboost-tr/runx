import { describe, expect, it } from "vitest";

import { parseSkillFrontmatter } from "./skill-frontmatter.js";

describe("parseSkillFrontmatter", () => {
  it("extracts skill frontmatter without parsing the skill body", () => {
    const raw = parseSkillFrontmatter(`---
name: echo
description: Echoes input.
---
# Echo
`);

    expect(raw.frontmatter).toMatchObject({
      name: "echo",
      description: "Echoes input.",
    });
  });

  it("requires delimited frontmatter", () => {
    expect(() => parseSkillFrontmatter("# Echo")).toThrow(
      "Skill markdown must start with YAML frontmatter delimited by ---.",
    );
  });
});
