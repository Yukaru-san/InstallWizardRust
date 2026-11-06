import init, { prepare_payload, export_artifact } from './pkg/installwizard.js';

async function template(target) {
  const response = await fetch('./templates/catalog.json');
  if (!response.ok) throw Error('No runtime catalog published. Build and publish the templates described in docs/BUILD.md.');
  const catalog = await response.json();
  if(catalog.version !== 1) throw Error("Unsupported runtime catalog version");
  const item = catalog.targets.find(item => item.id === target);
  if (!item) throw Error(`Runtime template is not published: ${target}`);
  const url = new URL(item.file, new URL('./templates/', self.location.href));
  if (url.origin !== self.location.origin) throw Error('Runtime templates must be served from the same origin');
  const download = await fetch(url);
  if (!download.ok) throw Error(`Could not load template: ${target}`);
  const bytes = new Uint8Array(await download.arrayBuffer());
  const digest = [...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(n=>n.toString(16).padStart(2,'0')).join('');
  if (digest !== item.sha256) throw Error(`Runtime template integrity failed: ${target}`);
  return bytes;
}
self.onmessage = async ({data}) => {
  try {
    await init();
    const config = JSON.parse(data.config);
    const targets = config.web ? ['web'] : [...new Set(config.targets)];
    // Fail early for missing templates, before reading or compressing user data.
    const templates = [];
    for (const target of targets) templates.push(target === 'web' ? new Uint8Array() : await template(target));
    self.postMessage({kind:'status',message:'Compressing files…'});
    const files = [];
    for (const entry of data.files) {
      files.push({path:entry.path,directory:!entry.file,data:entry.file ? new Uint8Array(await entry.file.arrayBuffer()) : new Uint8Array()});
    }
    const {payload,manifest} = prepare_payload(data.config, files);
    files.length = 0;
    for (let index = 0; index < targets.length; index++) {
      const target = targets[index];
      const bytes = export_artifact(templates[index], payload, manifest, target);
      const suffix = target === 'web' ? '.zip' : target.startsWith('macos') ? '.app.zip' : target.startsWith('windows') ? '.exe' : '';
      self.postMessage({kind:'download',name:`InstallWizard-${target}${suffix}`,bytes},[bytes.buffer]);
    }
    self.postMessage({kind:'done'});
  } catch(error) { self.postMessage({kind:'error',message:error.message || String(error)}); }
};
