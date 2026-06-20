const divergentBoolish = ["yes", "no", "on", "off"] as const;

export class YamlSubsetError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "YamlSubsetError";
  }
}

export function assertYamlParitySubset(field: string, source: string): void {
  let blockScalarIndent: number | undefined;
  for (const [lineIndex, line] of source.split(/\r?\n/).entries()) {
    const lineNumber = lineIndex + 1;
    const content = stripYamlComment(line);
    if (content === undefined) {
      continue;
    }
    const trimmed = content.trim();
    if (blockScalarIndent !== undefined) {
      if (trimmed === "" || leadingSpaces(content) > blockScalarIndent) {
        continue;
      }
      blockScalarIndent = undefined;
    }
    if (trimmed === "" || trimmed.startsWith("---") || trimmed.startsWith("...")) {
      continue;
    }
    rejectExplicitMappingKey(field, lineNumber, trimmed);
    rejectEmbeddedColonKey(field, lineNumber, trimmed);
    rejectColonSpacePlainScalar(field, lineNumber, content);
    blockScalarIndent = blockScalarIndentAfter(content) ?? blockScalarIndent;
  }
}

export function assertExecutionProfileYamlSubset(field: string, source: string): void {
  assertYamlParitySubset(field, source);
  const mappingStack: MappingFrame[] = [];
  let blockScalarIndent: number | undefined;
  for (const [lineIndex, line] of source.split(/\r?\n/).entries()) {
    const lineNumber = lineIndex + 1;
    const content = stripYamlComment(line);
    if (content === undefined) {
      continue;
    }
    const trimmed = content.trim();
    if (blockScalarIndent !== undefined) {
      if (trimmed === "" || leadingSpaces(content) > blockScalarIndent) {
        continue;
      }
      blockScalarIndent = undefined;
    }
    if (trimmed === "") {
      continue;
    }
    rejectDocumentMarker(field, lineNumber, trimmed);
    rejectYamlReferenceSyntax(field, lineNumber, content);
    rejectDuplicateMappingKey(field, lineNumber, content, mappingStack);
    blockScalarIndent = blockScalarIndentAfter(content) ?? blockScalarIndent;
  }
}

export function yamlScalarSubsetAllows(literal: string): boolean {
  const trimmed = literal.trim();
  return !isBoolish(trimmed)
    && !isBasePrefixedNumber(trimmed)
    && !isSexagesimalLike(trimmed)
    && !isDateLike(trimmed)
    && !isSpecialFloat(trimmed);
}

export function assertYamlScalarSubset(field: string, literal: string): void {
  if (yamlScalarSubsetAllows(literal)) {
    return;
  }
  throw new YamlSubsetError(`${field} uses unsupported YAML scalar ${JSON.stringify(literal)}.`);
}

function stripYamlComment(line: string): string | undefined {
  const scanner = new QuoteScanner();
  for (let index = 0; index < line.length; index += 1) {
    const char = line[index]!;
    if (scanner.isPlainAt(char) && char === "#" && isCommentStart(line, index)) {
      return line.slice(0, index);
    }
    scanner.consume(char);
  }
  return line;
}

function isCommentStart(line: string, index: number): boolean {
  return index === 0 || /\s/.test(line[index - 1]!);
}

function rejectExplicitMappingKey(field: string, lineNumber: number, trimmed: string): void {
  if (trimmed === "?" || trimmed.startsWith("? ")) {
    throw ambiguousYaml(field, lineNumber, trimmed);
  }
}

function rejectEmbeddedColonKey(field: string, lineNumber: number, trimmed: string): void {
  const key = topLevelPlainKey(trimmed)?.[0];
  if (key?.includes(":")) {
    throw ambiguousYaml(field, lineNumber, trimmed);
  }
}

function topLevelPlainKey(trimmed: string): [string, number] | undefined {
  const first = trimmed[0];
  if (first === undefined || ["-", "?", "{", "[", "\"", "'"].includes(first)) {
    return undefined;
  }
  const scanner = new QuoteScanner();
  for (let index = 0; index < trimmed.length; index += 1) {
    const char = trimmed[index]!;
    if (scanner.isPlainAt(char) && char === ":" && isMappingDelimiter(trimmed, index)) {
      return [trimmed.slice(0, index).trim(), index];
    }
    scanner.consume(char);
  }
  return undefined;
}

type QuoteState =
  | "plain"
  | "in-double"
  | "in-single-pending-apostrophe"
  | "in-single"
  | "in-double-escape";

class QuoteScanner {
  private state: QuoteState = "plain";

  isPlainAt(char: string): boolean {
    if (this.state === "plain") {
      return true;
    }
    if (this.state === "in-single-pending-apostrophe") {
      return char !== "'";
    }
    return false;
  }

  consume(char: string): void {
    if (this.state === "plain") {
      this.state = this.plainStateAfter(char);
      return;
    }
    if (this.state === "in-double") {
      this.state = char === "\\" ? "in-double-escape" : char === "\"" ? "plain" : "in-double";
      return;
    }
    if (this.state === "in-double-escape") {
      this.state = "in-double";
      return;
    }
    if (this.state === "in-single") {
      this.state = char === "'" ? "in-single-pending-apostrophe" : "in-single";
      return;
    }
    this.state = char === "'" ? "in-single" : this.plainStateAfter(char);
  }

  private plainStateAfter(char: string): QuoteState {
    if (char === "'") {
      return "in-single";
    }
    if (char === "\"") {
      return "in-double";
    }
    return "plain";
  }
}

function isMappingDelimiter(value: string, index: number): boolean {
  const next = value[index + 1];
  return next === undefined || /\s/.test(next);
}

function rejectColonSpacePlainScalar(field: string, lineNumber: number, content: string): void {
  const split = splitPlainMappingValue(content);
  if (!split) {
    return;
  }
  const [, value] = split;
  if (plainScalarContainsColonSpace(value)) {
    throw ambiguousYaml(field, lineNumber, value.trim());
  }
}

function rejectDocumentMarker(field: string, lineNumber: number, trimmed: string): void {
  if (
    trimmed === "---"
    || trimmed === "..."
    || trimmed.startsWith("--- ")
    || trimmed.startsWith("... ")
  ) {
    throw new YamlSubsetError(
      `${field}: YAML document markers are not supported in X.yaml at line ${lineNumber}; use one plain profile document.`,
    );
  }
}

function rejectYamlReferenceSyntax(field: string, lineNumber: number, content: string): void {
  for (const token of [": &", ": *", ": !", "- &", "- *", "- !"] as const) {
    if (containsPlainToken(content, token)) {
      throw new YamlSubsetError(
        `${field}: YAML anchors, aliases, and tags are not supported in X.yaml at line ${lineNumber}; write the profile explicitly.`,
      );
    }
  }
  const trimmed = content.trimStart();
  if (trimmed.startsWith("&") || trimmed.startsWith("*") || trimmed.startsWith("!")) {
    throw new YamlSubsetError(
      `${field}: YAML anchors, aliases, and tags are not supported in X.yaml at line ${lineNumber}; write the profile explicitly.`,
    );
  }
}

function containsPlainToken(content: string, token: string): boolean {
  const scanner = new QuoteScanner();
  for (let index = 0; index < content.length; index += 1) {
    const char = content[index]!;
    if (scanner.isPlainAt(char) && content.startsWith(token, index)) {
      return true;
    }
    scanner.consume(char);
  }
  return false;
}

interface MappingFrame {
  readonly indent: number;
  readonly keys: Set<string>;
}

function rejectDuplicateMappingKey(
  field: string,
  lineNumber: number,
  content: string,
  stack: MappingFrame[],
): void {
  const indent = leadingSpaces(content);
  const trimmed = content.trimStart();
  const sequenceKey = sequenceItemKey(trimmed, indent);
  const keyMatch = sequenceKey
    ? { keyIndent: sequenceKey[0], key: sequenceKey[1], sequenceItem: true }
    : topLevelPlainKey(trimmed)
      ? { keyIndent: indent, key: topLevelPlainKey(trimmed)![0], sequenceItem: false }
      : undefined;
  if (!keyMatch) {
    return;
  }
  const { key, keyIndent, sequenceItem } = keyMatch;
  if (key === "<<") {
    throw new YamlSubsetError(
      `${field}: YAML merge keys are not supported in X.yaml at line ${lineNumber}; write the profile explicitly.`,
    );
  }
  if (sequenceItem) {
    while (stack.at(-1) && stack.at(-1)!.indent >= keyIndent) {
      stack.pop();
    }
  } else {
    while (stack.at(-1) && stack.at(-1)!.indent > keyIndent) {
      stack.pop();
    }
  }
  if (!stack.at(-1) || stack.at(-1)!.indent !== keyIndent) {
    stack.push({ indent: keyIndent, keys: new Set() });
  }
  const frame = stack.at(-1)!;
  if (frame.keys.has(key)) {
    throw new YamlSubsetError(
      `${field}: duplicate mapping key ${JSON.stringify(key)} in X.yaml at line ${lineNumber}; keep profile keys unique.`,
    );
  }
  frame.keys.add(key);
}

function blockScalarIndentAfter(content: string): number | undefined {
  return blockScalarValueCandidates(content).some(isBlockScalarHeader) ? leadingSpaces(content) : undefined;
}

function blockScalarValueCandidates(content: string): string[] {
  const candidates: string[] = [];
  const mapping = splitPlainMappingValue(content);
  if (mapping) {
    candidates.push(mapping[1]);
  }
  const trimmed = content.trimStart();
  if (trimmed.startsWith("- ")) {
    const item = trimmed.slice(2).trimStart();
    candidates.push(item);
    const itemMapping = splitPlainMappingValue(item);
    if (itemMapping) {
      candidates.push(itemMapping[1]);
    }
  }
  return candidates;
}

function isBlockScalarHeader(value: string): boolean {
  return /^[|>](?:[+-]?\d?|\d?[+-]?)$/.test(value.trim());
}

function sequenceItemKey(trimmed: string, indent: number): [number, string] | undefined {
  const rest = trimmed.startsWith("- ") ? trimmed.slice(2) : undefined;
  if (rest === undefined) {
    return undefined;
  }
  const item = rest.trimStart();
  const leading = rest.length - item.length;
  const key = topLevelPlainKey(item)?.[0];
  return key === undefined ? undefined : [indent + 2 + leading, key];
}

function leadingSpaces(content: string): number {
  return content.length - content.trimStart().length;
}

function splitPlainMappingValue(content: string): [string, string] | undefined {
  const trimmed = content.trimStart();
  const split = topLevelPlainKey(trimmed);
  if (!split) {
    return undefined;
  }
  const [key, delimiterIndex] = split;
  return [key, trimmed.slice(delimiterIndex + 1)];
}

function plainScalarContainsColonSpace(value: string): boolean {
  const trimmed = value.trimStart();
  if (
    trimmed === ""
    || trimmed.startsWith("\"")
    || trimmed.startsWith("'")
    || trimmed.startsWith("|")
    || trimmed.startsWith(">")
    || trimmed.startsWith("{")
    || trimmed.startsWith("[")
    || trimmed === "null"
    || trimmed === "true"
    || trimmed === "false"
  ) {
    return false;
  }
  return containsUnquotedColonSpace(trimmed);
}

function containsUnquotedColonSpace(value: string): boolean {
  const scanner = new QuoteScanner();
  for (let index = 0; index < value.length; index += 1) {
    const char = value[index]!;
    if (scanner.isPlainAt(char) && char === ":" && isMappingDelimiter(value, index)) {
      return true;
    }
    scanner.consume(char);
  }
  return false;
}

function ambiguousYaml(field: string, lineNumber: number, literal: string): YamlSubsetError {
  return new YamlSubsetError(
    `${field}: ambiguous YAML construct at line ${lineNumber}; quote the value or key: ${literal}`,
  );
}

function isBoolish(value: string): boolean {
  return divergentBoolish.some((candidate) => candidate.toLowerCase() === value.toLowerCase());
}

function isBasePrefixedNumber(value: string): boolean {
  const unsigned = value.replace(/^[+-]/, "");
  return unsigned.startsWith("0x") || unsigned.startsWith("0X") || unsigned.startsWith("0o");
}

function isSexagesimalLike(value: string): boolean {
  const unsigned = value.replace(/^[+-]/, "");
  const parts = unsigned.split(":");
  const [first, ...rest] = parts;
  return Boolean(first)
    && /^\d+$/.test(first)
    && rest.length > 0
    && rest.every((part) => part !== "" && /^\d+$/.test(part));
}

function isDateLike(value: string): boolean {
  return /^\d{4}-\d{2}-\d{2}/.test(value);
}

function isSpecialFloat(value: string): boolean {
  return [".nan", ".inf", "+.inf", "-.inf"].includes(value.toLowerCase());
}
