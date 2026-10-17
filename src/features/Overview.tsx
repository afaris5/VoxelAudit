import {
  ArrowRight,
  Box,
  FileJson2,
  FolderOpen,
  Image,
  PackageOpen,
  ScanLine,
  ShieldCheck,
} from 'lucide-react';
import { bytes } from '../lib/format';
import type { Report, View } from '../lib/types';

export function Overview({
  report,
  open,
  navigate,
  busy,
}: {
  report: Report | null;
  open: (mode: string) => void;
  navigate: (view: View) => void;
  busy: boolean;
}) {
  if (!report)
    return (
      <div className="welcome">
        <div className="welcome-mark">
          <img src="/icon.svg" alt="VoxelAudit cube" />
        </div>
        <div className="eyebrow">MINECRAFT JAVA EDITION</div>
        <h1>Inspect a resource pack</h1>
        <p>
          Open a folder or ZIP to browse textures and check
          <br className="wide-break" /> models, blockstates and sound
          references.
        </p>
        <div className="welcome-actions">
          <button
            className="primary"
            disabled={busy}
            onClick={() => open('directory')}
          >
            <FolderOpen size={17} />
            Open directory
          </button>
          <button disabled={busy} onClick={() => open('archive')}>
            <PackageOpen size={17} />
            Open ZIP archive
          </button>
        </div>
        <div className="welcome-features">
          <div>
            <Image />
            <strong>Texture inspection</strong>
            <span>Dimensions, transparency and pixel zoom.</span>
          </div>
          <div>
            <ScanLine />
            <strong>Asset references</strong>
            <span>Models, textures, blockstates and sounds.</span>
          </div>
          <div>
            <ShieldCheck />
            <strong>Offline scans</strong>
            <span>Files stay on your computer.</span>
          </div>
        </div>
        <div className="welcome-note">
          MINECRAFT JAVA EDITION <span>RESOURCE PACKS / MOD ASSETS</span>
        </div>
      </div>
    );
  const kinds = [
    {
      kind: 'texture',
      name: 'Textures',
      icon: Image,
      view: 'textures' as const,
    },
    { kind: 'model', name: 'Models', icon: Box, view: 'models' as const },
    {
      kind: 'blockstate',
      name: 'Blockstates',
      icon: FileJson2,
      view: 'models' as const,
    },
    {
      kind: 'sound',
      name: 'Sound assets',
      icon: PackageOpen,
      view: 'models' as const,
    },
  ];
  return (
    <div className="page overview">
      <div className="eyebrow">PROJECT ANALYSIS</div>
      <h1>{report.projectName}</h1>
      <p className="muted">
        {report.assets.length.toLocaleString()} supported files ·{' '}
        {bytes(report.totalBytes)} · inspected in{' '}
        {(report.durationMs / 1000).toFixed(2)} s
      </p>
      <div
        className={`scan-outcome ${report.counts.error || !report.complete ? 'has-errors' : ''}`}
      >
        <ShieldCheck size={21} />
        <div>
          <strong>
            {!report.complete
              ? 'Analysis incomplete — some files were skipped'
              : report.counts.error
                ? 'Analysis complete — issues need attention'
                : report.counts.warning
                  ? 'Analysis complete — review warnings'
                  : 'Analysis complete'}
          </strong>
          <span>
            {report.complete
              ? report.findings.length
                ? 'Open Validation to review the results.'
                : 'No problems found in the inspected files.'
              : 'See import errors for inaccessible files or safety limits. Counts cover inspected files only.'}
          </span>
        </div>
        <button onClick={() => navigate('validation')}>
          Review findings
          <ArrowRight size={15} />
        </button>
      </div>
      <div className="stats">
        {(['error', 'warning', 'info'] as const).map((s) => (
          <button
            key={s}
            className={`stat ${s}`}
            onClick={() => navigate('validation')}
          >
            <span>{s === 'info' ? 'Informational' : `${s}s`}</span>
            <strong>{report.counts[s]}</strong>
            <small>
              {s === 'error'
                ? 'Invalid or unreadable assets'
                : s === 'warning'
                  ? 'References and texture dimensions'
                  : 'External references and scan details'}
            </small>
          </button>
        ))}
      </div>
      <div className="section-heading">
        <h2>Asset inventory</h2>
        <span>{report.namespaces.length} namespaces</span>
      </div>
      <div className="inventory">
        {kinds.map(({ kind, name, icon: Icon, view }) => (
          <button key={kind} onClick={() => navigate(view)}>
            <Icon size={20} />
            <span>{name}</span>
            <strong>
              {report.assets.filter((a) => a.kind === kind).length}
            </strong>
            <ArrowRight size={15} />
          </button>
        ))}
      </div>
      <div className="overview-grid">
        <section className="panel">
          <div className="section-heading">
            <h2>Namespaces</h2>
          </div>
          {report.namespaces.length ? (
            report.namespaces.map((ns) => (
              <div className="namespace-row" key={ns}>
                <code>{ns}</code>
                <span>
                  {report.assets.filter((a) => a.namespace === ns).length}{' '}
                  assets
                </span>
              </div>
            ))
          ) : (
            <p className="muted">No assets/&lt;namespace&gt; folders found.</p>
          )}
        </section>
        <section className="panel">
          <h2>Reading the results</h2>
          <p>
            Errors identify unreadable assets and invalid syntax. Warnings flag
            suspicious local references or dimensions.
          </p>
          <p>
            Vanilla and other namespaces appear as informational findings when
            their dependencies are absent. A resource pack can intentionally
            inherit those assets.
          </p>
          <button
            className="text-button"
            onClick={() => navigate('validation')}
          >
            View findings
            <ArrowRight size={15} />
          </button>
        </section>
      </div>
    </div>
  );
}
