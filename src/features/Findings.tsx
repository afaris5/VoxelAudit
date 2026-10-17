import { useMemo, useState } from 'react';
import { AlertCircle, AlertTriangle, Info, Search } from 'lucide-react';
import { filterFindings } from '../lib/format';
import type { Filters, Finding, Report } from '../lib/types';

const icons = { error: AlertCircle, warning: AlertTriangle, info: Info };

export function Findings({
  report,
  select,
}: {
  report: Report;
  select: (path: string) => void;
}) {
  const [filters, setFilters] = useState<Filters>({
    search: '',
    severity: '',
    category: '',
    namespace: '',
  });
  const [limit, setLimit] = useState(150);
  const update = (key: keyof Filters, value: string) => {
    setFilters((f) => ({ ...f, [key]: value }));
    setLimit(150);
  };
  const filtered = useMemo(
    () => filterFindings(report.findings, filters),
    [report.findings, filters],
  );
  const categories = [
    ...new Set(report.findings.map((f) => f.category)),
  ].sort();
  return (
    <div className="page">
      <div className="eyebrow">ASSET DIAGNOSTICS</div>
      <h1>Validation</h1>
      <p className="muted">
        Review errors, warnings and unresolved references.
      </p>
      <div className="filter-bar">
        <label className="search">
          <Search size={16} />
          <input
            aria-label="Search findings or filenames"
            placeholder="Search findings or filenames…"
            value={filters.search}
            onChange={(e) => update('search', e.target.value)}
          />
        </label>
        <select
          aria-label="Severity"
          value={filters.severity}
          onChange={(e) => update('severity', e.target.value)}
        >
          <option value="">All severities</option>
          <option value="error">Errors</option>
          <option value="warning">Warnings</option>
          <option value="info">Information</option>
        </select>
        <select
          aria-label="Category"
          value={filters.category}
          onChange={(e) => update('category', e.target.value)}
        >
          <option value="">All categories</option>
          {categories.map((c) => (
            <option key={c}>{c}</option>
          ))}
        </select>
        <select
          aria-label="Namespace"
          value={filters.namespace}
          onChange={(e) => update('namespace', e.target.value)}
        >
          <option value="">All namespaces</option>
          {report.namespaces.map((ns) => (
            <option key={ns}>{ns}</option>
          ))}
        </select>
      </div>
      <div className="section-heading">
        <span>
          {filtered.length} of {report.findings.length} findings
        </span>
        <span>PATH / DIAGNOSTIC</span>
      </div>
      <div className="findings-list">
        {filtered.slice(0, limit).map((finding, index) => (
          <FindingRow
            key={`${finding.path}-${finding.code}-${index}`}
            finding={finding}
            select={select}
          />
        ))}
      </div>
      {!filtered.length && (
        <div className="empty-inline">
          <Info size={28} />
          <h2>
            {report.findings.length
              ? 'No matching findings'
              : 'No findings in the inspected assets'}
          </h2>
          <p>Try adjusting the filters or rescan after changing your files.</p>
        </div>
      )}
      {filtered.length > limit && (
        <button className="load-more" onClick={() => setLimit(limit + 150)}>
          Show next 150 findings ({filtered.length - limit} remaining)
        </button>
      )}
    </div>
  );
}

function FindingRow({
  finding,
  select,
}: {
  finding: Finding;
  select: (path: string) => void;
}) {
  const Icon = icons[finding.severity];
  return (
    <article className={`finding ${finding.severity}`}>
      <Icon size={18} />
      <div>
        <div className="finding-title">
          <button
            onClick={() => select(finding.path)}
            title="Inspect associated file"
          >
            {finding.path || 'Project import'}
            {finding.line ? `:${finding.line}` : ''}
          </button>
          <span className="badge">{finding.category}</span>
        </div>
        <p>{finding.message}</p>
        {finding.context && (
          <code className="reference-target">↳ {finding.context}</code>
        )}
        <small>{finding.code}</small>
      </div>
      <span className="severity-label">{finding.severity}</span>
    </article>
  );
}
