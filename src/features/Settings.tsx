export function Settings() {
  return (
    <div className="page settings">
      <div className="eyebrow">VOXELAUDIT</div>
      <h1>About VoxelAudit</h1>
      <p className="muted">Version 0.1.0 · Minecraft Java Edition</p>
      <section className="panel">
        <h2>Your files</h2>
        <p>
          Scans run on your computer. Open a folder or ZIP to inspect its
          contents; VoxelAudit leaves the original files unchanged.
        </p>
        <p>
          Save exported reports outside the selected project, using a new
          filename.
        </p>
      </section>
      <section className="panel">
        <h2>Reference checks</h2>
        <p>
          Missing files in the current namespace appear as warnings. References
          to Minecraft or another namespace appear as information when their
          files are absent, since another pack or mod may supply them.
        </p>
        <dl className="properties">
          <div>
            <dt>Default namespace</dt>
            <dd>minecraft</dd>
          </div>
          <div>
            <dt>Vanilla / external dependencies</dt>
            <dd>Informational</dd>
          </div>
          <div>
            <dt>Absent local dependencies</dt>
            <dd>Warning</dd>
          </div>
        </dl>
      </section>
      <section className="panel">
        <h2>Import limits</h2>
        <p>
          Larger files are skipped and listed in the scan results. Text preview
          has a separate limit; it does not affect validation.
        </p>
        <dl className="properties">
          <div>
            <dt>Maximum imported entries</dt>
            <dd>50,000</dd>
          </div>
          <div>
            <dt>Maximum supported file size</dt>
            <dd>16 MiB</dd>
          </div>
          <div>
            <dt>Maximum project data</dt>
            <dd>512 MiB</dd>
          </div>
          <div>
            <dt>Decoded PNG limit</dt>
            <dd>64 MiB</dd>
          </div>
          <div>
            <dt>Text preview limit</dt>
            <dd>1 MiB</dd>
          </div>
        </dl>
      </section>
      <section className="panel">
        <h2>Supported files</h2>
        <p>
          PNG textures, JSON models, blockstates, sounds.json, language files
          and resource metadata. OGG files are included in reference checks;
          audio playback is not available.
        </p>
        <p>
          Select the folder containing assets, or a ZIP with assets at its root.
          Custom model loaders, animation timing, newer item definitions and
          pack format compatibility need to be checked in Minecraft.
        </p>
      </section>
    </div>
  );
}
