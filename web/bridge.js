// Browser capabilities only. Rust owns package validation and construction.
const pending = [];
const sources = new Map();
let nextId = 1;
let worker;
let downloads = [];
function event(kind, fields = {}) { pending.push({ kind, ...fields }); }
function register(entries, root = null) {
  const records = root ? [{path: root, entries}] : entries;
  const added = records.map(entry => {
    const key = `${nextId++} · ${entry.path}`;
    sources.set(key, entry);
    return { source: key, destination: entry.path, size: entry.file?.size ?? 0, directory: !entry.file };
  });
  event('sources', { sources: added });
}
function expand(config) {
  return config.sources.flatMap(source => {
    const record = sources.get(source.source);
    if (!record) throw Error('Selected file is no longer available. Add it again.');
    if (!record.entries) return [{...record, path: source.destination}];
    return record.entries.map(entry => {
      const relative = entry.path.slice(record.path.length).replace(/^\//, '');
      const path = [source.destination, relative].filter(Boolean).join('/');
      return {...entry, path};
    }).filter(entry => entry.path);
  });
}
function picker(folder, icon = false) {
  const input = document.createElement('input');
  input.type = 'file'; input.multiple = !icon;
  if (folder) input.setAttribute('webkitdirectory', '');
  if (icon) input.accept = '.png,.ico';
  input.addEventListener('change', async () => {
    try {
      const files = [...input.files];
      if (icon && files[0]) {
        if (files[0].size > 8 * 1024 * 1024) throw Error('Icon exceeds 8 MiB');
        event('icon', { bytes: [...new Uint8Array(await files[0].arrayBuffer())] });
      } else {
        register(files.map(file => ({ file, path: file.webkitRelativePath || file.name })), folder ? files[0]?.webkitRelativePath.split('/')[0] : null);
        if (folder) event('status', { message: 'Folder selected. This browser cannot report empty folders through its file picker.' });
      }
    } catch (error) { event('error', { message: error.message }); }
    input.remove();
  });
  input.click();
}
async function chooseFolder() {
  if (!window.showDirectoryPicker) return picker(true);
  try {
    const root = await window.showDirectoryPicker({ mode: 'read' });
    const entries = [{ path: root.name }];
    async function walk(directory, base) {
      for await (const [name, handle] of directory.entries()) {
        const path = `${base}/${name}`;
        if (handle.kind === 'directory') { entries.push({ path }); await walk(handle, path); }
        else entries.push({ path, file: await handle.getFile() });
      }
    }
    await walk(root, root.name); register(entries, root.name);
  } catch (error) { if (error.name !== 'AbortError') event('error', { message: error.message }); }
}
globalThis.InstallWizard = {
  chooseSources(folder) { if (folder) void chooseFolder(); else picker(false); },
  chooseIcon() { picker(false, true); },
  sourceMetadata(json) {
    return JSON.stringify(expand(JSON.parse(json)).map(entry => ({source:entry.path,destination:entry.path,size:entry.file?.size || 0,directory:!entry.file})));
  },
  takeEvents() { return JSON.stringify(pending.splice(0)); },
  begin(json) {
    try {
      if (worker) worker.terminate();
      for (const download of downloads) URL.revokeObjectURL(download.url);
      downloads = [];
      const config = JSON.parse(json);
      const files = expand(config);
      const total = files.reduce((sum, entry) => sum + (entry.file?.size || 0), 0);
      if (total > 500_000_000) throw Error('Browser packages are limited to 500 MB. Use the desktop creator for larger packages.');
      worker = new Worker(new URL('./worker.js', import.meta.url), { type: 'module' });
      worker.onmessage = ({data}) => {
        if (data.kind === 'download') {
          const blob = new Blob([data.bytes], {type: 'application/octet-stream'});
          downloads.push({name: data.name, url: URL.createObjectURL(blob)});
          event('download', {message: data.name});
        } else event(data.kind, {message: data.message || ''});
      };
      worker.onerror = error => { event('error', {message: error.message || 'Browser memory limit reached. Try a smaller package.'}); worker.terminate(); };
      worker.postMessage({config: json, files});
    } catch (error) { event('error', {message: error.message}); }
  },
  cancel() { if(worker) worker.terminate(); worker = undefined; },
  download(index) {
    const item = downloads[index]; if(!item)return;
    const a = document.createElement('a');a.href = item.url;a.download = item.name;
    document.body.append(a);a.click();a.remove();
  }
};
