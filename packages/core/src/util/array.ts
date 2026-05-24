export function unique<T>(values: readonly T[]): readonly T[] {
  return Array.from(new Set(values));
}

/** First trimmed non-empty value, or `undefined` when none qualify. */
export function firstNonEmptyOrUndefined(...values: readonly (string | undefined)[]): string | undefined {
  for (const value of values) {
    const trimmed = value?.trim();
    if (trimmed) {
      return trimmed;
    }
  }
  return undefined;
}

/** First trimmed non-empty value, or `""` when none qualify. */
export function firstNonEmpty(...values: readonly (string | undefined)[]): string {
  return firstNonEmptyOrUndefined(...values) ?? "";
}
