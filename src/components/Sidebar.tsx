import { Box, FileCheck2, FolderOpen, Image, Info, Shapes } from 'lucide-react';
import type { Report, View } from '../lib/types';

const navigation = [
  { id: 'project', label: 'Project overview', icon: FolderOpen },
  { id: 'textures', label: 'Textures', icon: Image },
  { id: 'models', label: 'Models & assets', icon: Shapes },
  { id: 'validation', label: 'Validation', icon: FileCheck2 },
] as const;

export function Sidebar({
  view,
  setView,
  report,
}: {
  view: View;
  setView: (view: View) => void;
  report: Report | null;
}) {
  return (
    <aside className="sidebar">
      <div className="brand">
        <img src="/icon.svg" alt="" />
        <div>
          VoxelAudit<small>RESOURCE QUALITY TOOLKIT</small>
        </div>
      </div>
      <div className="section-label">WORKSPACE</div>
      <nav aria-label="Main navigation">
        {navigation.map(({ id, label, icon: Icon }) => (
          <button
            key={id}
            className={`nav-item ${view === id ? 'active' : ''}`}
            onClick={() => setView(id)}
            aria-current={view === id ? 'page' : undefined}
          >
            <Icon size={17} />
            <span>{label}</span>
            {id === 'validation' && report && report.counts.error > 0 && (
              <b>{report.counts.error}</b>
            )}
          </button>
        ))}
      </nav>
      <div className="sidebar-project">
        <Box size={18} />
        <div>
          <strong>{report?.projectName ?? 'No project open'}</strong>
          <small>
            {report
              ? `${report.namespaces.length} namespaces · ${report.sourceKind}`
              : 'Directory or resource pack ZIP'}
          </small>
        </div>
      </div>
      <div className="sidebar-bottom">
        <button
          className={`nav-item ${view === 'settings' ? 'active' : ''}`}
          onClick={() => setView('settings')}
        >
          <Info size={17} />
          About VoxelAudit
        </button>
        <small>
          VoxelAudit <span>0.1.0</span>
        </small>
      </div>
    </aside>
  );
}
