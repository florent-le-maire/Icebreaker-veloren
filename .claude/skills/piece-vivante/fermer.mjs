// Icebreaker, skill piece-vivante : ferme proprement un Edge sans fenêtre par son port de débogage.
const port = process.argv[2];
const v = await (await fetch(`http://127.0.0.1:${port}/json/version`)).json();
const ws = new WebSocket(v.webSocketDebuggerUrl);
await new Promise(r => ws.addEventListener('open', r));
ws.send(JSON.stringify({ id: 1, method: 'Browser.close' }));
setTimeout(() => process.exit(0), 1000);
