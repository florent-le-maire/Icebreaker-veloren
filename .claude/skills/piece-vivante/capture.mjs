// Icebreaker, skill piece-vivante : capture une page locale ouverte dans Edge sans fenêtre (port de débogage).
// node capture.mjs <port> <sortie.png> [js à évaluer avant]
const [port, sortie, avant] = process.argv.slice(2);
const cibles = await (await fetch(`http://127.0.0.1:${port}/json`)).json();
const page = cibles.find(c => c.type === 'page');
const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise(r => ws.addEventListener('open', r));
let id = 0;
const attente = new Map();
ws.addEventListener('message', e => {
  const m = JSON.parse(e.data);
  if (m.id && attente.has(m.id)) { attente.get(m.id)(m); attente.delete(m.id); }
});
const envoyer = (method, params = {}) => new Promise(r => { const i = ++id; attente.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const evaluer = async expr => (await envoyer('Runtime.evaluate', { expression: expr, awaitPromise: true, returnByValue: true })).result;
for (let k = 0; k < 120; k++) {
  const r = await evaluer('document.body.dataset.pret || ""');
  if (r.result?.value) break;
  await new Promise(r => setTimeout(r, 500));
}
if (avant) console.log(JSON.stringify(await evaluer(avant)).slice(0, 2000));
await new Promise(r => setTimeout(r, 3000));
const img = await envoyer('Page.captureScreenshot', { format: 'png' });
(await import('fs')).writeFileSync(sortie, Buffer.from(img.result.data, 'base64'));
console.log('ok', sortie);
process.exit(0);
