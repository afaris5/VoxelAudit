import { describe, expect, it } from 'vitest';
import { bytes, filterFindings } from '../src/lib/format';
import type { Finding } from '../src/lib/types';

const findings: Finding[] = [
  {
    severity: 'error',
    category: 'JSON',
    path: 'assets/atelier/models/block/a.json',
    code: 'json.syntax',
    namespace: 'atelier',
    message: 'Invalid JSON syntax',
    line: 2,
    context: null,
  },
  {
    severity: 'warning',
    category: 'References',
    path: 'assets/other/models/block/b.json',
    code: 'reference.missing',
    namespace: 'other',
    message: 'Missing texture',
    line: null,
    context: 'assets/other/textures/block/moss.png',
  },
];

describe('finding filters', () => {
  it('combines severity, category, namespace and case-insensitive filename search', () => {
    expect(
      filterFindings(findings, {
        severity: 'error',
        category: 'JSON',
        namespace: 'atelier',
        search: 'A.JSON',
      }),
    ).toEqual([findings[0]]);
    expect(
      filterFindings(findings, {
        severity: 'warning',
        category: 'JSON',
        namespace: '',
        search: '',
      }),
    ).toEqual([]);
  });
  it('searches resolved target paths and messages', () => {
    expect(
      filterFindings(findings, {
        severity: '',
        category: '',
        namespace: '',
        search: 'MOSS',
      }),
    ).toEqual([findings[1]]);
    expect(
      filterFindings(findings, {
        severity: '',
        category: '',
        namespace: '',
        search: 'syntax',
      }),
    ).toEqual([findings[0]]);
  });
});

it('formats measured file sizes', () => {
  expect(bytes(800)).toBe('800 B');
  expect(bytes(1536)).toBe('1.5 KiB');
  expect(bytes(2 * 1024 * 1024)).toBe('2.0 MiB');
});
