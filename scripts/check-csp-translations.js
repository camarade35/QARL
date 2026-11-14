#!/usr/bin/env node
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { createInterface } from 'readline/promises';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const LANG_DIR = path.join(ROOT, 'src/lib/i18n/locales');
const SAMPLES_DIR = path.join(ROOT, 'samples');

async function main() {
  const rl = createInterface({ input: process.stdin, output: process.stdout });
  let iniPaths = [];
  let selectedLang = 'all';
  let showOrphans = false;

  const args = process.argv.slice(2);
  
  if (args.length > 0) {
    // Mode classique si tu passes des arguments
    iniPaths = args.filter(a => !a.startsWith('--'));
    const langArg = args.find(a => a.startsWith('--lang='));
    if (langArg) selectedLang = langArg.split('=')[1];
    showOrphans = args.includes('--orphans');
  } else {
    // Mode INTERACTIF magique ✨
    console.log("\n🛠️  VÉRIFICATEUR DE TRADUCTIONS CSP\n");
    
    const folderInput = await rl.question(`📁 Dossier contenant les .ini (Entrée pour "./samples") : `);
    const folderToScan = folderInput.trim() || SAMPLES_DIR;

    if (!fs.existsSync(folderToScan)) {
      console.error(`❌ Dossier introuvable : ${folderToScan}`);
      rl.close();
      process.exit(1);
    }

    if (fs.statSync(folderToScan).isDirectory()) {
      iniPaths = fs.readdirSync(folderToScan).filter(f => f.endsWith('.ini')).map(f => path.join(folderToScan, f));
      console.log(`   -> ${iniPaths.length} fichiers .ini trouvés.\n`);
    } else {
      iniPaths = [folderToScan];
    }

    if (iniPaths.length === 0) {
      console.error('❌ Aucun fichier .ini trouvé dans ce dossier.');
      rl.close();
      process.exit(1);
    }

    const langInput = await rl.question(`🌍 Langue à vérifier (fr, en, es, de, it, ou Entrée pour "toutes") : `);
    selectedLang = langInput.trim() || 'all';

    const orphansInput = await rl.question(`⚠️  Voir les clés orphelines ? (o/N) : `);
    showOrphans = orphansInput.trim().toLowerCase() === 'o';
    
    console.log("");
  }

  rl.close();

  const ALL_LANGS = ['fr', 'en', 'es', 'de', 'it'];
  const LANGS = selectedLang === 'all' ? ALL_LANGS : [selectedLang];

  if (!LANGS.every(l => ALL_LANGS.includes(l))) {
    console.error(`❌ Langue non supportée. Choix possibles : ${ALL_LANGS.join(', ')}`);
    process.exit(1);
  }

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
    if (!fs.existsSync(file)) {
      console.error(`❌ Fichier de langue introuvable : ${file}`);
      process.exit(1);
    }
    langsData[code] = JSON.parse(fs.readFileSync(file, 'utf-8'));
  }

  let hasErrors = false;
  const allIniKeys = new Set();
  const missingReport = {};
  for (const code of LANGS) missingReport[code] = [];

  for (const iniPath of iniPaths) {
    if (!fs.existsSync(iniPath)) continue;
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

  console.log(`🔍 Analyse de ${iniPaths.length} fichier(s) .ini terminée.\n`);

  if (hasErrors) {
    console.log('❌ VOICI LES TRADUCTIONS MANQUANTES :');
    for (const code of LANGS) {
      if (missingReport[code].length > 0) {
        console.log(`\n   === ${code}.json ===`);
        for (const item of missingReport[code]) {
          console.log(`   Dans ${item.file} :`);
          for (const m of item.missing) {
            console.log(`      - ${m}`);
          }
        }
      }
    }
    console.log('\n=======================================');
    console.log('❌ Vérification échouée.');
  } else {
    console.log('\n=======================================');
    console.log('✅ Tout est parfait ! Toutes les options sont traduites.');
  }

  // Affichage des orphelines si demandé
  if (showOrphans) {
    const orphanReport = {};
    for (const code of LANGS) {
      const labels = langsData[code]?.csp?.option_labels ?? {};
      const orphans = Object.keys(labels).filter((k) => !allIniKeys.has(k));
      if (orphans.length > 0) orphanReport[code] = orphans;
    }

    if (Object.keys(orphanReport).length > 0) {
      console.log('\n⚠️  Clés orphelines (dans le JSON mais dans aucun des .ini analysés) :');
      for (const [code, orphans] of Object.entries(orphanReport)) {
        console.log(`   ${code}.json : ${orphans.length} clé(s)`);
      }
    } else {
      console.log('\n✅ Aucune clé orpheline trouvée.');
    }
  }
}

main().catch(console.error);