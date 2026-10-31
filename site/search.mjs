// Locale-aware search helpers, intentionally dependency-free.
export function fold(value) {
  return String(value).normalize("NFD").replace(/n\u0303/g, "ñ").replace(/N\u0303/g, "Ñ").replace(/[\u0300-\u036f]/g, "").toLocaleLowerCase("es");
}
export function bucketOf(query) {
  const chars = Array.from(fold(query));
  if (chars.length < 2) return null;
  return chars.slice(0, 2).map(c => /^[a-z0-9]$/.test(c) ? c : "_").join("");
}
export function resultMatches(record, query, mexicanOnly = false, includeForms = false) {
  return fold(record[0]).startsWith(fold(query))
    && (!mexicanOnly || Boolean(record[4]))
    && (includeForms || !record[5]);
}
