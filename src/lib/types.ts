export type Severity = 'error' | 'warning' | 'info';

export interface Finding {
  severity: Severity;
  category: string;
  code: string;
  path: string;
  message: string;
  context: string | null;
  line: number | null;
  namespace: string;
}

export interface Texture {
  width: number;
  height: number;
  colorMode: string;
  hasAlpha: boolean;
  transparent: boolean;
}

export interface Asset {
  path: string;
  kind: string;
  namespace: string;
  bytes: number;
  texture: Texture | null;
}

export interface Report {
  schemaVersion: number;
  projectName: string;
  sourceKind: string;
  scannedAt: number;
  durationMs: number;
  totalBytes: number;
  assets: Asset[];
  findings: Finding[];
  namespaces: string[];
  counts: Record<Severity, number>;
  complete: boolean;
}

export interface Progress {
  phase: string;
  processed: number;
  total: number | null;
}

export type View =
  'project' | 'textures' | 'models' | 'validation' | 'settings';

export interface Filters {
  search: string;
  severity: string;
  category: string;
  namespace: string;
}
