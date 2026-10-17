import { useEffect, useState } from 'react';
import { Minus, Plus, X } from 'lucide-react';
import { preview, type PreviewData } from '../lib/native';
import { bytes } from '../lib/format';
import type { Asset, Finding } from '../lib/types';

export function Inspector({
  asset,
  findings,
  close,
}: {
  asset: Asset;
  findings: Finding[];
  close: () => void;
}) {
  const [data, setData] = useState<PreviewData | null>(null);
  const [error, setError] = useState('');
  const [zoom, setZoom] = useState(4);
  useEffect(() => {
    let current = true;
    setData(null);
    setError('');
    setZoom(4);
    preview(asset.path)
      .then((value) => {
        if (current) setData(value);
      })
      .catch((e: unknown) => {
        if (current) setError(String(e));
      });
    return () => {
      current = false;
    };
  }, [asset.path]);
  return (
    <aside className="inspector" aria-label="File inspector">
      <div className="inspector-heading">
        <span>FILE INSPECTOR</span>
        <button
          className="icon-button"
          aria-label="Close inspector"
          onClick={close}
        >
          <X size={17} />
        </button>
      </div>
      <h2>{asset.path.split('/').pop()}</h2>
      <code className="file-path">{asset.path}</code>
      {error ? (
        <p role="alert" className="notice">
          {error}
        </p>
      ) : !data ? (
        <p className="muted" role="status">
          Loading original file…
        </p>
      ) : data.image ? (
        <>
          <div className="texture-canvas">
            <img
              src={data.image}
              alt={asset.path}
              style={{
                width: (asset.texture?.width ?? 16) * zoom,
                height: (asset.texture?.height ?? 16) * zoom,
              }}
            />
          </div>
          <div className="zoom-controls">
            <button
              aria-label="Zoom out"
              disabled={zoom <= 0.25}
              onClick={() => setZoom(Math.max(0.25, zoom / 2))}
            >
              <Minus size={15} />
            </button>
            <span>{zoom * 100}%</span>
            <button
              aria-label="Zoom in"
              disabled={zoom >= 32}
              onClick={() => setZoom(Math.min(32, zoom * 2))}
            >
              <Plus size={15} />
            </button>
            <button onClick={() => setZoom(1)}>1:1</button>
          </div>
        </>
      ) : (
        <pre className="source-preview" tabIndex={0}>
          {data.text}
        </pre>
      )}
      <dl className="properties">
        <div>
          <dt>File size</dt>
          <dd>{bytes(asset.bytes)}</dd>
        </div>
        <div>
          <dt>Namespace</dt>
          <dd>{asset.namespace || '—'}</dd>
        </div>
        {asset.texture && (
          <>
            <div>
              <dt>Original dimensions</dt>
              <dd>
                {asset.texture.width} × {asset.texture.height}
              </dd>
            </div>
            <div>
              <dt>Color mode</dt>
              <dd>{asset.texture.colorMode}</dd>
            </div>
            <div>
              <dt>Alpha channel / table</dt>
              <dd>{asset.texture.hasAlpha ? 'Present' : 'Absent'}</dd>
            </div>
            <div>
              <dt>Transparent pixels</dt>
              <dd>{asset.texture.transparent ? 'Yes' : 'No'}</dd>
            </div>
          </>
        )}
      </dl>
      <div className="section-heading">
        <h2>File findings</h2>
        <span>{findings.length}</span>
      </div>
      {findings.length ? (
        findings.map((f, i) => (
          <p className={`inspector-finding ${f.severity}`} key={i}>
            <b>
              {f.severity}
              {f.line ? ` · line ${f.line}` : ''}
            </b>
            {f.message}
          </p>
        ))
      ) : (
        <p className="muted">No findings for this file.</p>
      )}
    </aside>
  );
}
