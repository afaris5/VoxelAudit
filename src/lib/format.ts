import type { Filters, Finding } from './types';

export function bytes(value: number): string {
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KiB`;
  return `${(value / (1024 * 1024)).toFixed(1)} MiB`;
}

export function filterFindings(
  findings: Finding[],
  filters: Filters,
): Finding[] {
  const query = filters.search.toLowerCase();
  return findings.filter(
    (f) =>
      (!filters.severity || f.severity === filters.severity) &&
      (!filters.category || f.category === filters.category) &&
      (!filters.namespace || f.namespace === filters.namespace) &&
      `${f.path} ${f.message} ${f.code} ${f.context ?? ''}`
        .toLowerCase()
        .includes(query),
  );
}
