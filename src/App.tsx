import { useCallback, useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { Download, FolderOpen, PackageOpen, RefreshCw, X } from 'lucide-react';
import { Sidebar } from './components/Sidebar';
import { Overview } from './features/Overview';
import { Findings } from './features/Findings';
import { AssetBrowser } from './features/AssetBrowser';
import { Inspector } from './features/Inspector';
import { Settings } from './features/Settings';
import { cancelScan, desktop, exportReport, scan } from './lib/native';
import type { Progress, Report, View } from './lib/types';

export default function App() {
  const [view, setView] = useState<View>('project');
  const [report, setReport] = useState<Report | null>(null);
  const [busy, setBusy] = useState(false);
  const [progress, setProgress] = useState<Progress | null>(null);
  const [notice, setNotice] = useState('');
  const [selected, setSelected] = useState<string | null>(null);
  useEffect(() => {
    if (!desktop) return;
    const unlisten = listen<Progress>('scan-progress', (event) =>
      setProgress(event.payload),
    );
    unlisten.catch((error: unknown) =>
      setNotice(`Progress updates are unavailable: ${String(error)}`),
    );
    return () => {
      void unlisten.then(
        (dispose) => dispose(),
        () => undefined,
      );
    };
  }, []);
  const open = useCallback(async (mode: string) => {
    setBusy(true);
    setNotice('');
    setProgress(null);
    try {
      const result = await scan(mode);
      if (result) {
        setReport(result);
        setSelected(null);
        if (mode !== 'rescan') setView('project');
      }
    } catch (error) {
      setNotice(String(error));
    } finally {
      setBusy(false);
      setProgress(null);
    }
  }, []);
  const exporting = async () => {
    try {
      if (await exportReport()) setNotice('Report saved successfully.');
    } catch (error) {
      setNotice(String(error));
    }
  };
  const select = (path: string) => {
    if (report?.assets.some((a) => a.path === path)) setSelected(path);
    else
      setNotice(
        'This file was absent or could not be imported. See the finding for details.',
      );
  };
  const asset = report?.assets.find((a) => a.path === selected);
  return (
    <div className="app-shell">
      <Sidebar view={view} setView={setView} report={report} />
      <div className="workspace">
        <header className="toolbar">
          <div className="breadcrumb">
            Workspace<span>/</span>
            <strong>{report?.projectName ?? 'Get started'}</strong>
          </div>
          <div className="toolbar-actions">
            <button
              disabled={busy}
              onClick={() => void open('directory')}
              title="Open directory"
            >
              <FolderOpen size={16} />
              <span>Open directory</span>
            </button>
            <button
              disabled={busy}
              onClick={() => void open('archive')}
              title="Open ZIP archive"
            >
              <PackageOpen size={16} />
              <span>Open ZIP</span>
            </button>
            {report && (
              <>
                <button disabled={busy} onClick={() => void open('rescan')}>
                  <RefreshCw size={15} />
                  <span>Rescan</span>
                </button>
                <button disabled={busy} onClick={() => void exporting()}>
                  <Download size={15} />
                  <span>Export JSON</span>
                </button>
              </>
            )}
          </div>
        </header>
        {notice && (
          <div className="notice-banner" role="status">
            <span>{notice}</span>
            <button
              aria-label="Dismiss notification"
              className="icon-button"
              onClick={() => setNotice('')}
            >
              <X size={16} />
            </button>
          </div>
        )}
        {!desktop && (
          <div className="browser-note">
            Browser preview · Launch the desktop app to open local directories
            and ZIP archives.
          </div>
        )}
        {busy && (
          <div className="scan-progress" role="status">
            <div>
              <span className="activity-dot" />
              <strong>
                {progress?.phase ?? 'Waiting for project selection…'}
              </strong>
              <span>
                {progress
                  ? `${progress.processed.toLocaleString()}${progress.total !== null ? ` / ${progress.total.toLocaleString()}` : ''} files`
                  : ''}
              </span>
              <button
                onClick={() =>
                  void cancelScan().catch((e: unknown) => setNotice(String(e)))
                }
              >
                Cancel scan
              </button>
            </div>
            {progress?.total !== null && progress?.total !== undefined && (
              <progress
                value={progress.processed}
                max={Math.max(1, progress.total)}
                aria-label="Scan progress"
              />
            )}
          </div>
        )}
        <div className="content">
          <main id="main-content">
            {view === 'settings' ? (
              <Settings />
            ) : !report || view === 'project' ? (
              <Overview
                report={report}
                open={(mode) => void open(mode)}
                navigate={setView}
                busy={busy}
              />
            ) : view === 'validation' ? (
              <Findings report={report} select={select} />
            ) : (
              <AssetBrowser
                key={view}
                report={report}
                textures={view === 'textures'}
                select={select}
                selected={selected}
              />
            )}
          </main>
          {asset && report && (
            <Inspector
              key={`${report.scannedAt}-${asset.path}`}
              asset={asset}
              findings={report.findings.filter((f) => f.path === asset.path)}
              close={() => setSelected(null)}
            />
          )}
        </div>
        <footer className="statusbar">
          <span>
            <i />
            {busy
              ? 'Scanning'
              : report
                ? report.complete
                  ? 'Analysis complete'
                  : 'Incomplete analysis'
                : 'Ready to inspect'}
          </span>
          <span>
            {report
              ? `${report.assets.length.toLocaleString()} assets · ${report.sourceKind}`
              : 'Minecraft Java Edition'}
          </span>
          <span>LOCAL & OFFLINE</span>
        </footer>
      </div>
    </div>
  );
}
