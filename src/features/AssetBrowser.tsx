import { useMemo, useState } from 'react';
import { FileJson2, Image, Search } from 'lucide-react';
import { bytes } from '../lib/format';
import type { Report } from '../lib/types';

export function AssetBrowser({
  report,
  textures,
  select,
  selected,
}: {
  report: Report;
  textures: boolean;
  select: (path: string) => void;
  selected: string | null;
}) {
  const [search, setSearch] = useState('');
  const [namespace, setNamespace] = useState('');
  const [kind, setKind] = useState('');
  const [limit, setLimit] = useState(200);
  const assets = useMemo(
    () =>
      report.assets.filter(
        (a) =>
          (!textures || a.kind === 'texture') &&
          (!kind || a.kind === kind) &&
          (!namespace || a.namespace === namespace) &&
          a.path.toLowerCase().includes(search.toLowerCase()),
      ),
    [report.assets, textures, search, namespace, kind],
  );
  return (
    <div className="page">
      <div className="eyebrow">
        {textures ? 'PNG TEXTURES' : 'PROJECT FILES'}
      </div>
      <h1>{textures ? 'Textures' : 'Models & assets'}</h1>
      <p className="muted">
        {textures
          ? 'Select a PNG to inspect its pixels, dimensions and transparency.'
          : 'Browse models, blockstates, metadata, language files and sound dependencies.'}
      </p>
      <div className="filter-bar">
        <label className="search">
          <Search size={16} />
          <input
            aria-label="Filter asset filenames"
            placeholder="Filter by filename…"
            value={search}
            onChange={(e) => {
              setSearch(e.target.value);
              setLimit(200);
            }}
          />
        </label>
        <select
          aria-label="Asset namespace"
          value={namespace}
          onChange={(e) => {
            setNamespace(e.target.value);
            setLimit(200);
          }}
        >
          <option value="">All namespaces</option>
          {report.namespaces.map((ns) => (
            <option key={ns}>{ns}</option>
          ))}
        </select>
        {!textures && (
          <select
            aria-label="Asset kind"
            value={kind}
            onChange={(e) => {
              setKind(e.target.value);
              setLimit(200);
            }}
          >
            <option value="">All kinds</option>
            {[...new Set(report.assets.map((a) => a.kind))].sort().map((k) => (
              <option key={k}>{k}</option>
            ))}
          </select>
        )}
      </div>
      <div className="section-heading">
        <span>{assets.length} assets</span>
        <span>SELECT A FILE TO INSPECT</span>
      </div>
      <div className="asset-table" role="list">
        {assets.slice(0, limit).map((asset) => (
          <button
            role="listitem"
            className={selected === asset.path ? 'selected' : ''}
            key={asset.path}
            onClick={() => select(asset.path)}
          >
            {asset.kind === 'texture' ? (
              <Image size={17} />
            ) : (
              <FileJson2 size={17} />
            )}
            <span>
              <strong>{asset.path.split('/').pop()}</strong>
              <small>{asset.path}</small>
            </span>
            <code>
              {asset.texture
                ? `${asset.texture.width} × ${asset.texture.height}`
                : asset.kind}
            </code>
            <small>{bytes(asset.bytes)}</small>
          </button>
        ))}
      </div>
      {!assets.length && (
        <div className="empty-inline">
          <Image size={28} />
          <h2>No matching assets</h2>
          <p>
            Choose another filter or open a project containing supported files.
          </p>
        </div>
      )}
      {assets.length > limit && (
        <button className="load-more" onClick={() => setLimit(limit + 200)}>
          Show next 200 assets
        </button>
      )}
    </div>
  );
}
