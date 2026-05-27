use std::sync::OnceLock;

use regex::Regex;
use runx_contracts::{JsonObject, JsonValue};

use crate::ParseError;

use super::{RawSkillIr, SkillQualityProfile};

pub fn parse_skill_markdown(markdown: &str) -> Result<RawSkillIr, ParseError> {
    static SKILL_FRONTMATTER_PATTERN: OnceLock<Result<Regex, String>> = OnceLock::new();
    let pattern = match SKILL_FRONTMATTER_PATTERN.get_or_init(|| {
        Regex::new(r"(?s)^---\r?\n(.*?)\r?\n---\r?\n?(.*)$").map_err(|error| error.to_string())
    }) {
        Ok(pattern) => pattern,
        Err(message) => {
            return Err(ParseError::InvalidDocument {
                field: "skill".to_owned(),
                message: message.clone(),
            });
        }
    };
    let Some(captures) = pattern.captures(markdown) else {
        return Err(ParseError::InvalidDocument {
            field: "skill".to_owned(),
            message: "Skill markdown must start with YAML frontmatter delimited by ---.".to_owned(),
        });
    };
    let raw_frontmatter = capture_string(&captures, 1)?;
    let body = capture_string(&captures, 2)?;
    let frontmatter = parse_yaml_object(
        &raw_frontmatter,
        "Skill frontmatter must parse to an object.",
    )?;
    Ok(RawSkillIr {
        frontmatter,
        raw_frontmatter,
        body,
    })
}

pub fn extract_skill_quality_profile(body: &str) -> Option<SkillQualityProfile> {
    extract_markdown_section(body, "Quality Profile", 2).map(|content| SkillQualityProfile {
        heading: "Quality Profile".to_owned(),
        content,
    })
}

fn parse_yaml_object(source: &str, object_error: &str) -> Result<JsonObject, ParseError> {
    crate::assert_yaml_parity_subset("skill_frontmatter", source)?;
    let parsed: JsonValue =
        serde_norway::from_str(source).map_err(|error| ParseError::InvalidYaml {
            field: "skill_frontmatter".to_owned(),
            message: error.to_string(),
        })?;
    match parsed {
        JsonValue::Object(object) => Ok(object),
        _ => Err(ParseError::InvalidDocument {
            field: "skill_frontmatter".to_owned(),
            message: object_error.to_owned(),
        }),
    }
}

fn capture_string(captures: &regex::Captures<'_>, index: usize) -> Result<String, ParseError> {
    captures
        .get(index)
        .map(|value| value.as_str().to_owned())
        .ok_or_else(|| ParseError::InvalidDocument {
            field: "skill".to_owned(),
            message: "Skill markdown must start with YAML frontmatter delimited by ---.".to_owned(),
        })
}

fn extract_markdown_section(body: &str, heading: &str, level: usize) -> Option<String> {
    let heading_prefix = "#".repeat(level);
    let boundary = "#".repeat(level + 1);
    let lines = body.lines().collect::<Vec<_>>();
    let start = lines.iter().position(|line| {
        line.trim()
            .eq_ignore_ascii_case(&format!("{heading_prefix} {heading}"))
    })?;
    let mut collected = Vec::new();
    for line in lines.iter().skip(start + 1) {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') && !trimmed.starts_with(&boundary) {
            break;
        }
        collected.push(*line);
    }
    let content = trim_blank_lines(&collected).join("\n").trim().to_owned();
    if content.is_empty() {
        None
    } else {
        Some(content)
    }
}

fn trim_blank_lines<'a>(lines: &'a [&'a str]) -> Vec<&'a str> {
    let mut start = 0;
    let mut end = lines.len();
    while start < end && lines[start].trim().is_empty() {
        start += 1;
    }
    while end > start && lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    lines[start..end].to_vec()
}
