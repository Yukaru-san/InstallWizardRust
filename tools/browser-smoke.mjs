// Optional browser integration test. Start the static creator on localhost:8080.
// Install playwright-core locally, or set IW_PLAYWRIGHT_MODULE to its module path.
import fs from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const {chromium}=await import(process.env.IW_PLAYWRIGHT_MODULE || 'playwright-core');
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const output=path.join(root,'dist/browser-check');await fs.mkdir(output,{recursive:true});
const fixture=path.join(output,'日本語.txt');await fs.writeFile(fixture,'hello from the browser');
const browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH || '/usr/bin/chromium',headless:true,
  args:['--no-sandbox','--disable-dev-shm-usage','--use-angle=swiftshader','--enable-unsafe-swiftshader']});
try {
  const page=await browser.newPage({viewport:{width:1280,height:850},acceptDownloads:true});
  const errors=[];page.on('pageerror',error=>errors.push(error.message));
  await page.goto(process.env.IW_WEB_URL || 'http://127.0.0.1:8080');
  await page.waitForFunction(()=>!document.getElementById('loading'),null,{timeout:60000});
  const chooser=page.waitForEvent('filechooser');await page.evaluate(()=>InstallWizard.chooseSources(false));
  await (await chooser).setFiles(fixture);await page.waitForTimeout(300);
  await page.screenshot({path:path.join(output,'desktop.png'),fullPage:true});
  await page.setViewportSize({width:390,height:844});await page.waitForTimeout(300);
  await page.screenshot({path:path.join(output,'mobile.png'),fullPage:true});
  async function job(target,bad=false) {
    return page.evaluate(({target,bad})=>new Promise(resolve=>{
      const worker=new Worker('./worker.js',{type:'module'});
      worker.onmessage=({data})=>{
        if(data.kind==='download'){
          const a=document.createElement('a');a.href=URL.createObjectURL(new Blob([data.bytes],{type:'application/octet-stream'}));a.download=data.name;a.click();
        }else if(data.kind==='done'||data.kind==='error'){worker.terminate();resolve(data);}
      };
      worker.onerror=error=>{worker.terminate();resolve({kind:'error',message:error.message})};
      worker.postMessage({config:JSON.stringify({name:'Browser smoke test',icon_png:null,sources:[],targets:target==='web'?[]:[target],web:target==='web'}),
        files:[{path:'assets'},{path:'assets/empty'},{path:bad?'../escape':'assets/日本語.txt',file:new File(['hello from the browser'],'hello.txt')}]});
    }),{target,bad});
  }
  const download=page.waitForEvent('download',{timeout:60000});
  const result=await job('web');if(result.kind!=='done')throw Error(result.message);
  await (await download).saveAs(path.join(output,'web-wizard.zip'));
  const invalid=await job('web',true);if(invalid.kind!=='error')throw Error('Traversal was accepted');
  if(errors.length)throw Error(errors.join('\n'));
  console.log(`Browser startup, file picker, worker export and traversal rejection passed. Artifacts: ${output}`);
} finally {await browser.close();}
