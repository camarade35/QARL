import http from 'http';
import fs from 'fs';
import path from 'path';
import { exec } from 'child_process';
import { promisify } from 'util';
import { fileURLToPath } from 'url';

const execAsync = promisify(exec);
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const PORT = 3456;

const htmlContent = `<!DOCTYPE html>
<html lang="fr">
<head>
  <meta charset="UTF-8">
  <title>QARL - Vérificateur de Traductions</title>
  <style>
    body {
      background-color: #121212;
      color: #e0e0e0;
      font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
      display: flex;
      justify-content: center;
      padding-top: 40px;
      margin: 0;
    }
    .container {
      width: 100%;
      max-width: 900px;
      background-color: #1e1e1e;
      padding: 30px;
      border-radius: 12px;
      box-shadow: 0 10px 30px rgba(0,0,0,0.5);
      border: 1px solid #333;
    }
    h1 { color: #ff5252; margin-top: 0; font-size: 24px; display: flex; align-items: center; gap: 10px; }
    .row { display: flex; gap: 20px; margin-bottom: 20px; }
    .col { flex: 1; display: flex; flex-direction: column; }
    label { font-size: 14px; margin-bottom: 6px; color: #aaa; }
    .input-group { display: flex; gap: 8px; }
    input[type="text"], select {
      padding: 10px;
      background: #2a2a2a;
      border: 1px solid #444;
      color: white;
      border-radius: 6px;
      outline: none;
      font-size: 14px;
      width: 100%;
      box-sizing: border-box;
    }
    input[type="text"]:focus, select:focus { border-color: #ff5252; }
    button {
      padding: 10px 15px;
      background: #ff5252;
      color: white;
      border: none;
      border-radius: 6px;
      font-weight: bold;
      font-size: 14px;
      cursor: pointer;
      transition: background 0.2s;
      white-space: nowrap;
    }
    button:hover { background: #ff1744; }
    button:disabled { background: #555; cursor: not-allowed; }
    
    .main-btn {
      width: 100%;
      padding: 12px;
      font-size: 16px;
      margin-top: 10px;
    }
    .checkbox-wrap { display: flex; align-items: center; gap: 10px; margin-bottom: 20px; font-size: 14px; color: #ccc; }
    input[type="checkbox"] { cursor: pointer; width: 16px; height: 16px; }
    pre {
      margin-top: 20px;
      background: #000;
      padding: 20px;
      border-radius: 8px;
      min-height: 200px;
      max-height: 500px;
      overflow-y: auto;
      white-space: pre-wrap;
      font-family: 'Consolas', 'Courier New', monospace;
      font-size: 14px;
      border: 1px solid #333;
      color: #b0b0b0;
    }
    .hint { font-size: 12px; color: #888; margin-top: 4px; }
  </style>
</head>
<body>
  <div class="container">
    <h1>🛠️ QARL - Vérificateur de Traductions</h1>
    
    <div class="row">
      <div class="col">
        <label>Dossier contenant les fichiers .ini :</label>
        <div class="input-group">
          <input type="text" id="folder" value="samples" />
          <button onclick="browse()" id="btn-browse">📁 Parcourir</button>
        </div>
        <div class="hint">Relatif au projet ou absolu (ex: C:\\...\\samples)</div>
      </div>
      <div class="col">
        <label>Langue à vérifier :</label>
        <select id="lang">
          <option value="all">🌍 Toutes les langues</option>
          <option value="fr">🇫🇷 Français (fr)</option>
          <option value="en">🇬🇧 Anglais (en)</option>
          <option value="es">🇪🇸 Espagnol (es)</option>
          <option value="de">🇩🇪 Allemand (de)</option>
          <option value="it">🇮🇹 Italien (it)</option>
        </select>
      </div>
    </div>

    <div class="checkbox-wrap">
      <input type="checkbox" id="orphans">
      <label for="orphans" style="margin:0; cursor:pointer;">Afficher les clés orphelines (inutiles)</label>
    </div>
    
    <button class="main-btn" id="btn-check" onclick="check()">🔍 Lancer la vérification</button>
    
    <pre id="output">Prêt à analyser...</pre>
  </div>

  <script>
    async function browse() {
      const btn = document.getElementById('btn-browse');
      const input = document.getElementById('folder');
      const originalText = btn.textContent;
      
      btn.disabled = true;
      btn.textContent = '⏳...';
      
      try {
        const res = await fetch('/browse');
        const data = await res.json();
        if (data.path) {
          input.value = data.path;
        }
      } catch (e) {
        alert('Erreur lors de l\\'ouverture de la fenêtre : ' + e.message);
      } finally {
        btn.disabled = false;
        btn.textContent = originalText;
      }
    }

    async function check() {
      const folder = document.getElementById('folder').value;
      const lang = document.getElementById('lang').value;
      const orphans = document.getElementById('orphans').checked;
      const btn = document.getElementById('btn-check');
      const out = document.getElementById('output');

      btn.disabled = true;
      btn.textContent = '⏳ Analyse en cours...';
      out.textContent = 'Patientez...';
      out.style.color = '#b0b0b0';

      try {
        const res = await fetch('/check?folder=' + encodeURIComponent(folder) + '&lang=' + lang + '&orphans=' + orphans);
        const text = await res.text();
        out.textContent = text;
        
        if (text.includes('❌')) out.style.color = '#ff5252';
        else if (text.includes('✅')) out.style.color = '#4CAF50';
      } catch (e) {
        out.textContent = '❌ Erreur de connexion au serveur local : ' + e.message;
        out.style.color = '#ff5252';
      } finally {
        btn.disabled = false;
        btn.textContent = '🔍 Lancer la vérification';
      }
    }
  </script>
</body>
</html>`;

function runCheck(folder, langs, showOrphans) {
  const ALL_LANGS = ['fr', 'en', 'es', 'de', 'it'];
  const LANGS = langs === 'all' ? ALL_LANGS : [langs];
  const LANG_DIR = path.join(ROOT, 'src/lib/i18n/locales');

  let folderPath = folder;
  if (!path.isAbsolute(folderPath)) {
    folderPath = path.join(ROOT, folderPath);
  }

  if (!fs.existsSync(folderPath) || !fs.statSync(folderPath).isDirectory()) {
    return `❌ Dossier introuvable : ${folderPath}`;
  }

  const ignoredFiles = ['module_lw.ini', 'music.ini', 'splashscreen.ini'];
  const iniPaths = fs.readdirSync(folderPath)
    .filter(f => f.endsWith('.ini') && !f.startsWith('data_') && !ignoredFiles.includes(f))
    .map(f => path.join(folderPath, f));
  
  if (iniPaths.length === 0) return `❌ Aucun fichier .ini pertinent trouvé dans : ${folderPath}`;

  const extractKeys = (iniPath) => {
    const content = fs.readFileSync(iniPath, 'utf-8');
    const keys = new Set();
    let currentSection = '';
    for (const rawLine of content.split(/\r?\n/)) {
      const line = rawLine.trim();
      if (!line || line.startsWith(';') || line.startsWith('#')) continue;
      if (line.startsWith('[')) { currentSection = line.replace(/[\[\]]/g, ''); continue; }
      if (currentSection === 'ℹ') continue;
      const eq = line.indexOf('=');
      if (eq === -1) continue;
      const key = line.slice(0, eq).trim();
      if (!key || key.startsWith('__')) continue;
      keys.add(key);
    }
    return keys;
  };

  const langsData = {};
  for (const code of LANGS) {
    const file = path.join(LANG_DIR, `${code}.json`);
    if (!fs.existsSync(file)) return `❌ Fichier JSON introuvable : ${file}`;
    langsData[code] = JSON.parse(fs.readFileSync(file, 'utf-8'));
  }

  let hasErrors = false;
  const allIniKeys = new Set();
  const missingReport = {};
  for (const code of LANGS) missingReport[code] = [];

  for (const iniPath of iniPaths) {
    const keys = extractKeys(iniPath);
    for (const k of keys) allIniKeys.add(k);
    const name = path.basename(iniPath);

    for (const code of LANGS) {
      const labels = langsData[code]?.csp?.option_labels ?? {};
      const missing = [...keys].filter((k) => !(k in labels));
      if (missing.length > 0) {
        hasErrors = true;
        missingReport[code].push({ file: name, missing });
      }
    }
  }

  let output = `🔍 Analyse de ${iniPaths.length} fichier(s) .ini pertinents terminée.\n\n`;

  if (hasErrors) {
    output += '❌ VOICI LES TRADUCTIONS MANQUANTES :\n';
    for (const code of LANGS) {
      if (missingReport[code].length > 0) {
        output += `\n   === ${code}.json ===\n`;
        for (const item of missingReport[code]) {
          output += `   Dans ${item.file} :\n`;
          for (const m of item.missing) {
            output += `      - ${m}\n`;
          }
        }
      }
    }
    output += '\n=======================================\n❌ Vérification échouée.';
  } else {
    output += '\n=======================================\n✅ Tout est parfait ! Toutes les options sont traduites.';
  }

  if (showOrphans) {
    const orphanReport = {};
    for (const code of LANGS) {
      const labels = langsData[code]?.csp?.option_labels ?? {};
      const orphans = Object.keys(labels).filter((k) => !allIniKeys.has(k));
      if (orphans.length > 0) orphanReport[code] = orphans;
    }

    if (Object.keys(orphanReport).length > 0) {
      output += '\n\n⚠️  Clés orphelines (dans le JSON mais dans aucun des .ini analysés) :\n';
      for (const [code, orphans] of Object.entries(orphanReport)) {
        output += `   ${code}.json : ${orphans.length} clé(s)\n`;
      }
    } else {
      output += '\n\n✅ Aucune clé orpheline trouvée.';
    }
  }
  return output;
}

// ⭐ FONCTION POUR OUVRIR LA FENÊTRE WINDOWS ⭐
async function pickFolder() {
  try {
    if (process.platform === 'win32') {
      const psScript = `
        Add-Type -AssemblyName System.Windows.Forms;
        $FolderBrowser = New-Object System.Windows.Forms.FolderBrowserDialog;
        $FolderBrowser.Description = 'Sélectionnez le dossier contenant les fichiers .ini';
        $FolderBrowser.ShowNewFolderButton = $false;
        $result = $FolderBrowser.ShowDialog();
        if ($result -eq [System.Windows.Forms.DialogResult]::OK) {
            Write-Output $FolderBrowser.SelectedPath
        }
      `;
      // Encodage en base64 pour éviter les problèmes de guillemets avec PowerShell
      const encoded = Buffer.from(psScript, 'utf16le').toString('base64');
      const { stdout } = await execAsync(`powershell -NoProfile -EncodedCommand ${encoded}`);
      return stdout.trim();
    } else if (process.platform === 'darwin') {
      const { stdout } = await execAsync(`osascript -e 'POSIX path of (choose folder with prompt "Sélectionnez le dossier")'`);
      return stdout.trim();
    } else {
      // Linux (Zenity)
      const { stdout } = await execAsync(`zenity --file-selection --directory --title="Sélectionnez le dossier"`);
      return stdout.trim();
    }
  } catch (e) {
    console.error('Erreur pickFolder:', e);
    return null;
  }
}

const server = http.createServer(async (req, res) => {
  const url = new URL(req.url, `http://${req.headers.host}`);
  
  if (req.method === 'GET' && url.pathname === '/') {
    res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
    res.end(htmlContent);
    return;
  }

  if (req.method === 'GET' && url.pathname === '/browse') {
    const folderPath = await pickFolder();
    res.writeHead(200, { 'Content-Type': 'application/json; charset=utf-8' });
    res.end(JSON.stringify({ path: folderPath }));
    return;
  }

  if (req.method === 'GET' && url.pathname === '/check') {
    const folder = url.searchParams.get('folder') || 'samples';
    const lang = url.searchParams.get('lang') || 'all';
    const orphans = url.searchParams.get('orphans') === 'true';
    
    try {
      const result = runCheck(folder, lang, orphans);
      res.writeHead(200, { 'Content-Type': 'text/plain; charset=utf-8' });
      res.end(result);
    } catch (e) {
      res.writeHead(500, { 'Content-Type': 'text/plain; charset=utf-8' });
      res.end('❌ Erreur interne : ' + e.message);
    }
    return;
  }
  
  res.writeHead(404);
  res.end('Not found');
});

server.listen(PORT, () => {
  console.log(`\n🖥️  Interface graphique lancée sur : http://localhost:${PORT}`);
  console.log(`⚠️  NE FERME PAS CE TERMINAL tant que tu utilises l'interface.\n`);
  
  const start = process.platform === 'win32' ? 'start ""' : process.platform === 'darwin' ? 'open' : 'xdg-open';
  exec(`${start} http://localhost:${PORT}`);
});