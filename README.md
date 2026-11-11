<div align="center">
  <img src="docs/screenshots/logo.png" alt="QARL Logo" width="200"/>
  <h1>QARL - Quick Assetto Race Launcher</h1>
  <p><i>Un launcher moderne, rapide et passionné pour Assetto Corsa.</i></p>
  
  [![Release](https://img.shields.io/github/v/release/camarade35/QARL?style=flat-square)](https://github.com/camarade35/QARL/releases)
  [![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)]()
</div>

---

**QARL est né d'une passion : celle d'Assetto Corsa.** 

Comme beaucoup d'entre vous, j'ai passé des heures à peaufiner mes setups, à installer des mods, et à chercher la météo parfaite pour une course de nuit sur le Nordschleife. Pendant des années, **Content Manager** a été notre maison, notre outil indispensable. C'est un chef-d'œuvre que nous devons tous à Ilja et à la communauté. 

Mais je voulais quelque chose de différent. Quelque chose de plus moderne, de plus épuré, avec une interface qui mette en valeur la beauté de nos voitures et de nos circuits, sans compromis sur la puissance. C'est ainsi qu'est né **QARL**.

Ce projet n'est pas un remplaçant, c'est un hommage. C'est un launcher fait par un passionné, pour les passionnés. 

## ✨ Fonctionnalités Principales

- **Interface Native Ultra-Rapide** : Développé avec Tauri (Rust) et Svelte, QARL est incroyablement léger et réactif.
- **Intégration Météo Avancée** : Support complet de CSP, Sol et Pure. Créez des plans météo complexes ou utilisez la météo dynamique.
- **Gestionnaire de Mods Intelligent** : Glissez-déposez vos archives (`.zip`, `.rar`, `.7z`), QARL analyse et installe tout au bon endroit (voitures, circuits, apps).
- **Multijoueur Simplifié** : Rejoignez des serveurs en un clic, filtrez par mods installés, et entrez vos mots de passe en toute sécurité.
- **Entièrement Personnalisable** : Thèmes (Carbon, Cloud), couleurs d'accentuation dynamiques, et choix de polices.
- **Auto-Update** : QARL se met à jour tout seul en arrière-plan pour toujours vous offrir la dernière version.

## 📸 Aperçu

<p align="center">
  <img src="docs/screenshots/interface.png" width="800" alt="Interface QARL"/>
  <br>
  <i>L'interface principale, épurée et moderne.</i>
</p>

<p align="center">
  <img src="docs/screenshots/weather.png" width="400" alt="Gestion météo"/>
  <img src="docs/screenshots/mods.png" width="400" alt="Gestionnaire de mods"/>
</p>

## 🚀 Installation

1. Rendez-vous dans l'onglet [**Releases**](https://github.com/camarade35/QARL/releases).
2. Téléchargez le dernier installateur `.exe` (ex: `QARL_x64-setup.exe`).
3. Lancez l'installation. 
4. Configurez vos chemins Assetto Corsa au premier lancement via l'assistant intégré.

## 🙏 Remerciements

Un immense merci à :
- **[Ilja Jusupov (gro-ove)](https://github.com/gro-ove)** et à l'équipe de **[Content Manager](https://acstuff.ru/app/)** pour avoir ouvert la voie et créé l'outil qui a révolutionné Assetto Corsa.
- **[La communauté Assetto Corsa](https://www.assettocorsa.net/)**, les créateurs de mods, et les développeurs de **[CSP](https://github.com/ac-custom-shaders-patch/acc-extension-config)**, **[Sol](https://www.racedepartment.com/downloads/sol.24914/)** et **[Pure](https://www.patreon.com/pure)**.

## 💻 Pour les Développeurs

Si vous souhaitez compiler QARL vous-même :
```bash
# Cloner le projet
git clone https://github.com/camarade35/QARL.git

# Installer les dépendances
npm install

# Lancer en mode développement
npm run tauri dev