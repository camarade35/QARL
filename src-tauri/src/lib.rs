use std::fs;
use std::path::Path;
use std::sync::Mutex;
use serde::Serialize;
use tauri::Emitter;
use tauri::Manager;
use tauri::Listener; // ⭐ Ajouté pour corriger l'erreur E0599
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use cpal::traits::{DeviceTrait, HostTrait};

// ⭐ Listes officielles des IDs de dossiers Kunos basées sur les captures
const KUNOS_CARS: &[&str] = &[
    "abarth500", "abarth500_s1", "alfa_romeo_giulietta_qv", "alfa_romeo_giulietta_qv_le",
    "bmw_1m", "bmw_1m_s3", "bmw_m3_e30", "bmw_m3_e30_drift", "bmw_m3_e30_dtm", "bmw_m3_e30_gra", "bmw_m3_e30_s1",
    "bmw_m3_e92", "bmw_m3_e92_drift", "bmw_m3_e92_s1", "bmw_m3_gt2", "bmw_z4", "bmw_z4_drift", "bmw_z4_gt3", "bmw_z4_s1",
    "ferrari_312t", "ferrari_458", "ferrari_458_gt2", "ferrari_458_s3", "ferrari_599xxevo", "ferrari_f40", "ferrari_f40_s3", "ferrari_laferrari",
    "ks_abarth500_assetto_corse", "ks_abarth_595ss", "ks_abarth_595ss_s1", "ks_abarth_595ss_s2",
    "ks_alfa_33_stradale", "ks_alfa_giulia_qv", "ks_alfa_mito_qv", "ks_alfa_romeo_4c", "ks_alfa_romeo_155_v6", "ks_alfa_romeo_gta",
    "ks_audi_a1s1", "ks_audi_r8_lms", "ks_audi_r8_lms_2016", "ks_audi_r8_plus", "ks_audi_r18_etron_quattro",
    "ks_audi_sport_quattro", "ks_audi_sport_quattro_rally", "ks_audi_sport_quattro_s1", "ks_audi_tt_cup", "ks_audi_tt_vln",
    "ks_bmw_m4", "ks_bmw_m4_akrapovic", "ks_bmw_m235i_racing", "ks_chevrolet_c7r", "ks_chevrolet_c7_stingray",
    "ks_ferrari_250_gto", "ks_ferrari_288_gto", "ks_ferrari_312_67", "ks_ferrari_330_p4", "ks_ferrari_488_challenge_evo",
    "ks_ferrari_488_gt3", "ks_ferrari_488_gt3_2020", "ks_ferrari_488_gtb", "ks_ferrari_812_superfast",
    "ks_ferrari_f138", "ks_ferrari_f2004", "ks_ferrari_fxx_k", "ks_ferrari_sf15t", "ks_ferrari_sf70h",
    "ks_ford_escort_mk1", "ks_ford_gt40", "ks_ford_mustang_2015", "ks_glickenhaus_scg003",
    "ks_lamborghini_aventador_sv", "ks_lamborghini_countach", "ks_lamborghini_countach_s1",
    "ks_lamborghini_gallardo_sl", "ks_lamborghini_gallardo_sl_s3", "ks_lamborghini_huracan_gt3",
    "ks_lamborghini_huracan_performante", "ks_lamborghini_huracan_st", "ks_lamborghini_miura_sv", "ks_lamborghini_sesto_elemento",
    "ks_lotus_3_eleven", "ks_lotus_25", "ks_lotus_72d",
    "ks_maserati_250f_6cyl", "ks_maserati_250f_12cyl", "ks_maserati_alfieri", "ks_maserati_gt_mc_gt4",
    "ks_maserati_levante", "ks_maserati_mc12_gt1", "ks_maserati_quattroporte",
    "ks_mazda_787b", "ks_mazda_miata", "ks_mazda_mx5_cup", "ks_mazda_mx5_nd", "ks_mazda_rx7_spirit_r", "ks_mazda_rx7_tuned",
    "ks_mclaren_570s", "ks_mclaren_650_gt3", "ks_mclaren_f1_gtr", "ks_mclaren_p1", "ks_mclaren_p1_gtr",
    "ks_mercedes_190_evo2", "ks_mercedes_amg_gt3", "ks_mercedes_c9",
    "ks_nissan_370z", "ks_nissan_gtr", "ks_nissan_gtr_gt3", "ks_nissan_skyline_r34", "ks_pagani_huayra_bc",
    "ks_porsche_718_boxster_s", "ks_porsche_718_boxster_s_pdk", "ks_porsche_718_cayman_s", "ks_porsche_718_spyder_rs",
    "ks_porsche_908_lh", "ks_porsche_911_carrera_rsr", "ks_porsche_911_gt1", "ks_porsche_911_gt3_cup_2017",
    "ks_porsche_911_gt3_rs", "ks_porsche_911_gt3_r_2016", "ks_porsche_911_r", "ks_porsche_911_rsr_2017",
    "ks_porsche_917_30", "ks_porsche_917_k", "ks_porsche_918_spyder", "ks_porsche_919_hybrid_2015",
    "ks_porsche_919_hybrid_2016", "ks_porsche_935_78_moby_dick", "ks_porsche_962c_longtail", "ks_porsche_962c_shorttail",
    "ks_porsche_991_carrera_s", "ks_porsche_991_turbo_s", "ks_porsche_cayenne", "ks_porsche_cayman_gt4_clubsport",
    "ks_porsche_cayman_gt4_std", "ks_porsche_macan", "ks_porsche_panamera", "ks_praga_r1",
    "ks_ruf_rt12r", "ks_ruf_rt12r_awd",
    "ks_toyota_ae86", "ks_toyota_ae86_drift", "ks_toyota_ae86_tuned", "ks_toyota_celica_st185", "ks_toyota_gt86",
    "ks_toyota_supra_mkiv", "ks_toyota_supra_mkiv_drift", "ks_toyota_supra_mkiv_tuned", "ks_toyota_ts040",
    "ktm_xbow_r", "lotus_2_eleven", "lotus_2_eleven_gt4", "lotus_49", "lotus_98t", "lotus_elise_sc", "lotus_elise_sc_s1",
    "lotus_elise_sc_s2", "lotus_evora_gtc", "lotus_evora_gte", "lotus_evora_gte_carbon", "lotus_evora_gx",
    "lotus_evora_s", "lotus_evora_s_s2", "lotus_exige_240", "lotus_exige_240_s3", "lotus_exige_s", "lotus_exige_scura",
    "lotus_exige_s_roadster", "lotus_exige_v6_cup", "lotus_exos_125", "lotus_exos_125_s1", "mclaren_mp412c",
    "mclaren_mp412c_gt3", "mercedes_sls", "mercedes_sls_gt3", "p4-5_2011", "pagani_huayra", "pagani_zonda_r",
    "ruf_yellowbird", "shelby_cobra_427sc", "tatuusfa1"
];

const KUNOS_TRACKS: &[&str] = &[
    "drift", "imola", "ks_barcelona", "ks_black_cat_county", "ks_brands_hatch", "ks_drag", "ks_highlands",
    "ks_laguna_seca", "ks_monza66", "ks_nordschleife", "ks_nurburgring", "ks_red_bull_ring", "ks_silverstone",
    "ks_silverstone1967", "ks_vallelunga", "ks_zandvoort", "magione", "monza", "mugello", "spa", "trento-bondone"
];


#[derive(Serialize, Clone)]
struct SkinInfo { name: String, image_path: String }

#[derive(Serialize, Clone)]
struct CarInfo {
    id: String, name: String, brand: String, image_path: String,
    flag_path: Option<String>, brand_logo_path: Option<String>,
    tags: Vec<String>, skins: Vec<SkinInfo>, is_kunos: bool,
    car_class: String,
    bhp: f32,
    pwratio: f32,
    year: i32,
}

#[derive(Serialize, Clone)]
struct TrackLayoutInfo {
    id: String, name: String,
    outline_path: Option<String>,
    preview_path: Option<String>,
}

#[derive(Serialize, Clone)]
struct TrackInfo {
    id: String, name: String, city: String, country: String,
    flag_path: Option<String>, preview_path: Option<String>,
    outline_path: Option<String>, tags: Vec<String>, layouts: Vec<TrackLayoutInfo>, is_kunos: bool,
}

fn extract_json_string(content: &str, key: &str) -> Option<String> {
    let search_lower = format!("\"{}\"", key.to_lowercase());
    let search_upper = format!("\"{}\"", key);
    
    let pos = content.find(&search_lower).or_else(|| content.find(&search_upper))?;
    
    let after_key = &content[pos + search_lower.len()..];
    let colon_pos = after_key.find(':')?;
    let after_colon = &after_key[colon_pos + 1..];
    
    let start_quote = after_colon.find('"')?;
    let value_str = &after_colon[start_quote + 1..];
    let end_quote = value_str.find('"')?;
    
    Some(value_str[..end_quote].to_string())
}

fn extract_json_tags(content: &str, key: &str) -> Vec<String> {
    let search = format!("\"{}\"", key);
    if let Some(pos) = content.find(&search) {
        let after = &content[pos + search.len()..];
        if let Some(colon) = after.find(':') {
            let after_colon = &after[colon + 1..];
            if let Some(start_bracket) = after_colon.find('[') {
                let after_bracket = &after_colon[start_bracket + 1..];
                if let Some(end_bracket) = after_bracket.find(']') {
                    let arr_str = &after_bracket[..end_bracket];
                    return arr_str.split(',')
                        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
        }
    }
    Vec::new()
}

fn get_valid_path_helper(p: std::path::PathBuf) -> Option<String> {
    if p.exists() { Some(p.to_string_lossy().replace("\\", "/")) } else { None }
}

// ⭐ Fonction magique pour trouver le dossier cfg peu importe le chemin choisi
fn resolve_cfg_dir(doc_path: &Path) -> std::path::PathBuf {
    if doc_path.ends_with("cfg") && doc_path.exists() {
        return doc_path.to_path_buf();
    }
    if doc_path.join("cfg").exists() {
        return doc_path.join("cfg");
    }
    // ⭐ Sécurité : Si le chemin se termine déjà par "Assetto Corsa", on ajoute JUSTE cfg
    if doc_path.file_name().unwrap_or_default().to_string_lossy().eq_ignore_ascii_case("Assetto Corsa") {
        return doc_path.join("cfg");
    }
    if doc_path.join("Assetto Corsa").join("cfg").exists() {
        return doc_path.join("Assetto Corsa").join("cfg");
    }
    doc_path.join("cfg")
}

fn get_exact_image(dir: &Path, name: &str) -> Option<String> {
    for ext in &["png", "jpg", "jpeg"] {
        let p = dir.join(format!("{}.{}", name, ext));
        if p.exists() { return Some(p.to_string_lossy().replace("\\", "/")); }
    }
    None
}

fn clean_track_id(id: &str) -> String {
    let mut name = id.replace("ks_", "").replace('_', " ");
    name = name.split_whitespace().map(|w| {
        let mut c = w.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str()
        }
    }).collect::<Vec<_>>().join(" ");
    name
}

#[tauri::command]
fn get_ac_cars(ac_path: String) -> Result<Vec<CarInfo>, String> {
    let cars_dir = Path::new(&ac_path).join("content").join("cars");
    if !cars_dir.exists() { return Err("Dossier introuvable".into()); }
    let mut cars = Vec::new();
    
    if let Ok(entries) = fs::read_dir(cars_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let id = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let mut name = id.clone();
                let mut brand = "Mod".to_string();
                let mut tags = Vec::new();
                let mut car_class = "Inconnue".to_string();
                let mut bhp = 0.0;
                let mut pwratio = 0.0;
                let mut year = 0;
                
                let ui_json_path = path.join("ui").join("ui_car.json");
                if let Ok(json_content) = fs::read_to_string(&ui_json_path) {
                    let clean_json = json_content.trim_start_matches('\u{feff}');
                    
                    if let Some(n) = extract_json_string(clean_json, "name") {
                        if !n.is_empty() { name = n; }
                    }
                    if let Some(b) = extract_json_string(clean_json, "brand") {
                        if !b.is_empty() { brand = b; }
                    }
                    if let Some(c) = extract_json_string(clean_json, "class") {
                        if !c.is_empty() { car_class = c; }
                    }
                    if let Some(b) = extract_json_string(clean_json, "bhp") {
                        let num_str: String = b.chars().filter(|c| c.is_digit(10) || *c == '.').collect();
                        bhp = num_str.parse::<f32>().unwrap_or(0.0);
                    }
                    if let Some(p) = extract_json_string(clean_json, "pwratio") {
                        let num_str: String = p.chars().filter(|c| c.is_digit(10) || *c == '.').collect();
                        pwratio = num_str.parse::<f32>().unwrap_or(0.0);
                    }
                    if let Some(y) = extract_json_string(clean_json, "year") {
                        let num_str: String = y.chars().filter(|c| c.is_digit(10)).collect();
                        year = num_str.parse::<i32>().unwrap_or(0);
                    }
                    tags = extract_json_tags(clean_json, "tags");
                }

                // ⭐ On cherche badge.png en priorité, sinon brand.png
                let brand_logo_path = get_valid_path_helper(path.join("ui").join("badge.png"))
                    .or_else(|| get_valid_path_helper(path.join("ui").join("brand.png")));
                    
                let flag_path = get_valid_path_helper(path.join("ui").join("country.png"));

                let mut skins = Vec::new();
                let mut default_image = "".to_string();
                let skins_dir = path.join("skins");
                
                if let Ok(skin_entries) = fs::read_dir(&skins_dir) {
                    let mut sorted_skins: Vec<_> = skin_entries.flatten().collect();
                    sorted_skins.sort_by_key(|e| e.file_name());

                    for skin in sorted_skins {
                        let skin_path = skin.path();
                        if skin_path.is_dir() {
                            let skin_name = skin_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                            if let Some(img) = get_valid_path_helper(skin_path.join("preview.jpg")).or_else(|| get_valid_path_helper(skin_path.join("preview.png"))) {
                                if default_image.is_empty() { default_image = img.clone(); }
                                skins.push(SkinInfo { name: skin_name, image_path: img });
                            }
                        }
                    }
                }
                
                // ⭐ On considère Kunos si l'ID commence par "ks_" OU s'il est dans la liste statique
                let is_kunos = id.starts_with("ks_") || KUNOS_CARS.contains(&id.as_str()); 
                cars.push(CarInfo { id, name, brand, car_class, bhp, pwratio, year, tags, is_kunos, skins, image_path: default_image, flag_path, brand_logo_path });
            }
        }
    }
    cars.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(cars)
}

#[tauri::command]
fn get_ac_tracks(ac_path: String) -> Result<Vec<TrackInfo>, String> {
    let tracks_dir = Path::new(&ac_path).join("content").join("tracks");
    if !tracks_dir.exists() { return Err("Dossier content/tracks introuvable".into()); }
    let mut tracks = Vec::new();

    if let Ok(entries) = fs::read_dir(tracks_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }

            let id = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let ui_dir = path.join("ui");
            let ui_json_path = ui_dir.join("ui_track.json");

            let mut name = clean_track_id(&id);
            let mut city = String::new();
            let mut country = String::new();
            let mut tags = Vec::new();
            
            // ⭐ Variables pour récupérer les infos depuis les layouts si la racine est vide
            let mut fallback_country = String::new();
            let mut fallback_tags: Vec<String> = Vec::new();

            if ui_json_path.exists() {
                if let Ok(json_content) = fs::read_to_string(&ui_json_path) {
                    let clean_json = json_content.trim_start_matches('\u{feff}');
                    
                    if let Some(n) = extract_json_string(clean_json, "name") {
                        if !n.is_empty() && n != id && !n.starts_with("ks_") { name = n; }
                    }
                    if let Some(c) = extract_json_string(clean_json, "city") { city = c; }
                    if let Some(c) = extract_json_string(clean_json, "country") { country = c; }
                    tags = extract_json_tags(clean_json, "tags");
                }
            }

            let flag_path = get_valid_path_helper(ui_dir.join("country.png"));
            let ui_outline = get_exact_image(&ui_dir, "outline");
            let ui_preview = get_exact_image(&ui_dir, "preview").or_else(|| get_exact_image(&ui_dir, "dlc_preview"));

            let mut layouts: Vec<TrackLayoutInfo> = Vec::new();
            let main_preview = ui_preview.clone();
            let main_outline = ui_outline.clone();

            if ui_outline.is_some() && ui_preview.is_some() {
                layouts.push(TrackLayoutInfo {
                    id: id.clone(),
                    name: "".to_string(),
                    outline_path: ui_outline.clone(),
                    preview_path: ui_preview.clone(),
                });
            } else {
                let search_dirs = vec![ui_dir.clone(), path.clone()];
                for d in search_dirs {
                    if let Ok(sub_entries) = fs::read_dir(d) {
                        for sub_entry in sub_entries.flatten() {
                            let sub_path = sub_entry.path();
                            if sub_path.is_dir() {
                                let folder_name = sub_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                                if folder_name == "ui" || folder_name == "skins" || folder_name == "extension" { continue; }

                                // ⭐ On évite de traiter deux fois le même layout
                                if layouts.iter().any(|l| l.id == folder_name) { continue; }

                                let mut display_name = folder_name.replace("layout_", "").replace('_', " ");

                                // ⭐ Recherche robuste du ui_track.json
                                let mut layout_json = sub_path.join("ui_track.json");
                                if !layout_json.exists() { layout_json = sub_path.join("ui").join("ui_track.json"); }
                                if !layout_json.exists() { layout_json = ui_dir.join(&folder_name).join("ui_track.json"); }
                                if !layout_json.exists() { layout_json = ui_dir.join(&folder_name).join("ui").join("ui_track.json"); }

                                if layout_json.exists() {
                                    if let Ok(bytes) = fs::read(&layout_json) {
                                        // ⭐ Gestion de TOUS les encodages (UTF-8, ANSI, UTF-16 LE/BE)
                                        let json_str = if bytes.starts_with(&[0xFF, 0xFE]) {
                                            let u16_slice: Vec<u16> = bytes[2..].chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
                                            String::from_utf16_lossy(&u16_slice)
                                        } else if bytes.starts_with(&[0xFE, 0xFF]) {
                                            let u16_slice: Vec<u16> = bytes[2..].chunks_exact(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect();
                                            String::from_utf16_lossy(&u16_slice)
                                        } else {
                                            String::from_utf8_lossy(&bytes).to_string()
                                        };

                                        // ⭐ Parsing fiable avec serde_json
                                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
                                            if let Some(n) = parsed.get("name").and_then(|v| v.as_str()) {
                                                if !n.is_empty() { display_name = n.to_string(); }
                                            }
                                            
                                            if fallback_country.is_empty() {
                                                if let Some(c) = parsed.get("country").and_then(|v| v.as_str()) {
                                                    if !c.is_empty() { fallback_country = c.to_string(); }
                                                }
                                            }
                                            
                                            if fallback_tags.is_empty() {
                                                if let Some(tags_arr) = parsed.get("tags").and_then(|v| v.as_array()) {
                                                    let t: Vec<String> = tags_arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                                                    if !t.is_empty() { fallback_tags = t; }
                                                }
                                            }
                                        }
                                    }
                                }

                                // ⭐ On cherche les images
                                let sub_outline = get_exact_image(&sub_path, "outline")
                                    .or_else(|| get_exact_image(&sub_path.join("ui"), "outline"));
                                    
                                let sub_preview = get_exact_image(&sub_path, "preview")
                                    .or_else(|| get_exact_image(&sub_path.join("ui"), "preview"))
                                    .or_else(|| get_exact_image(&sub_path, "dlc_preview"))
                                    .or_else(|| get_exact_image(&sub_path.join("ui"), "dlc_preview"));

                                if sub_outline.is_some() || sub_preview.is_some() {
                                    layouts.push(TrackLayoutInfo {
                                        id: folder_name,
                                        name: display_name.to_uppercase(),
                                        outline_path: sub_outline.or_else(|| main_outline.clone()),
                                        preview_path: sub_preview.or_else(|| main_preview.clone()),
                                    });
                                }
                            }
                        }
                    }
                }
            }

            layouts.sort_by(|a, b| a.name.cmp(&b.name));

            // ⭐ NOUVEAU : Si le pays ou les tags n'ont pas été trouvés à la racine, on prend ceux du layout
            if country.is_empty() && !fallback_country.is_empty() {
                country = fallback_country;
            }
            if tags.is_empty() && !fallback_tags.is_empty() {
                tags = fallback_tags;
            }

            // ⭐ On utilise la liste exacte
            let is_kunos = KUNOS_TRACKS.contains(&id.as_str());
            tracks.push(TrackInfo {
                id, name, city, country, flag_path, is_kunos,
                preview_path: main_preview,
                outline_path: main_outline,
                tags, layouts,
            });
        }
    }

    tracks.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(tracks)
}

#[derive(Serialize, Clone)]
struct WeatherInfo { id: String, name: String }

#[derive(Serialize, Clone)]
struct PurePlanStep {
    weather_id: String,
    weather_index: i64,
    duration: i32,
}

#[derive(Serialize, Clone)]
struct PurePlanInfo { 
    id: String, 
    name: String, 
    start_time: Option<String>,
    steps: Vec<PurePlanStep>,
    loop_plan: bool,
    time_mult: i32,
}

#[derive(Serialize, serde::Deserialize, Clone)]
struct PureCtrlSettings {
    autostart: bool,
    last_used: bool,
    live: bool,
    start_wetness: i32,
    start_puddles: i32,
}

// ⭐ Nettoie les IDs météo pour qu'ils correspondent aux presets de Pure
fn extract_pure_id(weather_name: &str) -> String {
    let raw_id = weather_name.replace("pure_", "").replace("sol_", "").replace("sfx_", "");
    if let Some(idx) = raw_id.find('_') {
        let before = &raw_id[..idx];
        if before.chars().all(|c| c.is_digit(10)) {
            return raw_id[idx+1..].to_string();
        }
    }
    raw_id
}

#[tauri::command]
fn get_ac_weathers(ac_path: String) -> Result<Vec<WeatherInfo>, String> {
    let mut weathers = Vec::new();
    let base = Path::new(&ac_path);

    // ⭐ 1. Scan des météos Kunos (content/weather)
    let content_weather_dir = base.join("content").join("weather");
    if content_weather_dir.exists() {
        if let Ok(entries) = fs::read_dir(content_weather_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let id = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let mut name = id.replace('_', " ");
                    if let Some(first_char) = name.chars().next() {
                        name = format!("{}{}", first_char.to_uppercase(), &name[first_char.len_utf8()..]);
                    }
                    weathers.push(WeatherInfo { id, name });
                }
            }
        }
    }

    // ⭐ 2. On ne scanne plus extension/weather car Sol/Pure copient leurs météos directement dans content/weather
    // C'est ce dossier que le jeu lit, on est donc sûr à 100% que ce sont de vraies météos.

    weathers.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(weathers)
}

#[tauri::command]
fn get_pure_plans(ac_path: String) -> Result<Vec<PurePlanInfo>, String> {
    let plans_dir = Path::new(&ac_path).join("extension").join("config-ext").join("PurePlanner").join("Plans");
    if !plans_dir.exists() { return Ok(Vec::new()); }
    
    // ⭐ LECTURE DES PRESETS POUR FAIRE LA CORRESPONDANCE INDEX -> NOM (ex: 20 -> "fog")
    let presets_path = Path::new(&ac_path).join("extension").join("config-ext").join("PurePlanner").join("WeatherPresets").join("Default_WeatherPresets.json");
    let presets: Vec<Vec<serde_json::Value>> = fs::read_to_string(&presets_path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default();

    let mut plans = Vec::new();
    
    // ⭐ Scan récursif pour inclure Stamp/, Daycycle/, etc.
    fn scan_dir(dir: &Path, base: &Path, plans: &mut Vec<PurePlanInfo>, presets: &[Vec<serde_json::Value>]) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    scan_dir(&path, base, plans, presets);
                } else if let Some(ext) = path.extension() {
                    if ext == "json" || ext == "ini" {
                        let relative_path = path.strip_prefix(base).unwrap_or(&path);
                        // On crée l'ID avec slash pour correspondre à ce qu'attend CM/Pure
                        let id = relative_path.to_string_lossy().replace("\\", "/");
                        
                        let file_name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                        let name = file_name.replace('_', " ");
                        
                                                // ⭐ On lit l'heure de début ET les étapes dans le JSON
                        let mut start_time = None;
                        let mut steps = Vec::new();
                        let mut loop_plan = false; // ⭐ NOUVEAU
                        let mut time_mult = 1;     // ⭐ NOUVEAU
                        
                        if ext == "json" {
                            if let Ok(content) = fs::read_to_string(&path) {
                                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                                    // ⭐ Lecture des paramètres de contrôle
                                    if let Some(ctrl) = parsed.get("control") {
                                        loop_plan = ctrl["loop"].as_bool().unwrap_or(false);
                                        time_mult = ctrl["timemulti"].as_i64().unwrap_or(1) as i32;
                                    }

                                    if let Some(container) = parsed["container"].as_array() {
                                        for (i, c) in container.iter().enumerate() {
                                            let index = c["data"]["weather"]["index"].as_i64().unwrap_or(15);
                                            let duration = (c["data"]["duration"].as_i64().unwrap_or(0) / 60) as i32;
                                            
                                            let weather_id = if let Some(qarl_id) = c["data"]["weather"]["__qarl_weather_id"].as_str() {
                                                qarl_id.to_string()
                                            } else {
                                                // ⭐ Si c'est un plan Pure natif, on traduit l'index en nom (ex: 20 -> "fog")
                                                let name = presets.iter()
                                                    .find(|p| p[1].as_i64().unwrap_or(-1) == index)
                                                    .and_then(|p| p[0].as_str())
                                                    .unwrap_or("clear");
                                                format!("{}", name) // On renvoie juste "fog", "windy", etc.
                                            };
                                                
                                            steps.push(PurePlanStep { weather_id, weather_index: index, duration });

                                            // On récupère l'heure de début de la première étape
                                            if i == 0 {
                                                if let Some(ts) = c["data"]["timestamp"].as_i64() {
                                                    let seconds_in_day = ts % 86400;
                                                    let h = seconds_in_day / 3600;
                                                    let m = (seconds_in_day % 3600) / 60;
                                                    start_time = Some(format!("{:02}:{:02}", h, m));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        plans.push(PurePlanInfo { id, name, start_time, steps, loop_plan, time_mult });
                    }
                }
            }
        }
    }
    
    scan_dir(&plans_dir, &plans_dir, &mut plans, &presets);
    plans.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(plans)
}

#[tauri::command]
fn load_pure_plan(ac_path: String, plan_id: String) -> Result<serde_json::Value, String> {
    // ⭐ On nettoie l'ID au cas où il contient déjà .json (ce qui est le cas depuis get_pure_plans)
    let clean_id = plan_id.replace(".json", "");
    let path = Path::new(&ac_path).join("extension").join("config-ext").join("PurePlanner").join("Plans").join(format!("{}.json", clean_id));
    if !path.exists() { return Err("Plan introuvable".into()); }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "planName": plan_id, "data": json }))
}

#[derive(serde::Deserialize, Clone)]
struct PureStep {
    weather: String,
    duration: i32,
}

#[tauri::command]
fn save_custom_pure_plan(
    ac_path: String,
    plan_name: String,
    steps: Vec<PureStep>,
    session_start_time: String,
    time_mult: i32,
    loop_plan: bool,
) -> Result<String, String> {
    let base = Path::new(&ac_path);
    let presets_path = base.join("extension").join("config-ext").join("PurePlanner").join("WeatherPresets").join("Default_WeatherPresets.json");
    
    // 1. Lire le fichier JSON de Pure
    let content = fs::read_to_string(&presets_path).map_err(|e| format!("Impossible de lire Default_WeatherPresets.json: {}", e))?;
    let presets: Vec<Vec<serde_json::Value>> = serde_json::from_str(&content).map_err(|e| format!("Erreur parsing JSON: {}", e))?;

    let mut container = Vec::new();
    let mut cumulative_seconds: i64 = 0;

    // ⭐ Calcul d'un timestamp basé sur AUJOURD'HUI à minuit (UTC)
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let midnight = now - (now % 86400);
    
    // Convertir l'heure de début (ex "14:30") en timestamp de base
    let mut base_hour = 12;
    let mut base_minute = 0;
    if let Some((h, m)) = session_start_time.split_once(':') {
        base_hour = h.parse::<i32>().unwrap_or(12);
        base_minute = m.parse::<i32>().unwrap_or(0);
    }
    
    // On ajoute les heures et minutes à minuit aujourd'hui
    let base_timestamp = midnight + (base_hour as i64 * 3600) + (base_minute as i64 * 60);

    // 2. Construire les étapes
    for step in steps {
        // Trouver le preset correspondant dans le JSON
        // ⭐ CORRECTION : On utilise extract_pure_id pour gérer sol_xx, 3_clear, etc.
        let preset_id = extract_pure_id(&step.weather);
        let mut preset_data = presets.iter().find(|p| p[0].as_str().unwrap_or("") == preset_id);

        // Si on ne trouve pas (ex: custom), on prend 'clear' par défaut
        if preset_data.is_none() {
            preset_data = presets.iter().find(|p| p[0].as_str().unwrap_or("") == "clear");
        }

        if let Some(p) = preset_data {
            let duration_sec = (step.duration * 60) as i64;
            let current_time = base_timestamp + cumulative_seconds;

            // ⭐ Génération EXACTE du JSON attendu par Pure (avec toutes les clés)
            let weather_obj = serde_json::json!({
                "wind_strength": p[7].as_f64().unwrap_or(0.0),
                "rain_probability": p[2].as_f64().unwrap_or(0.0),
                "temp_road_dyn": true, "temp_air_dyn": true, "mist_dyn": true, "humidity_dyn": true,
                "rain_water_dyn": true, "rain_wetness_dyn": true, "rain_amount_dyn": true,
                "temp_air_range": 0, "wind_direction": 0, "humidity_range": 0,
                "rain_wetness": p[3].as_f64().unwrap_or(0.0), "rain_wetness_range": 0,
                "rain_variance_range": 0, "rain_probability_range": 0, "rain_amount_range": 0,
                "wind_strength_range": 0, "wind_direction_range": 0, "temp_road_range": 0,
                "mist": p[9].as_f64().unwrap_or(0.0),
                "temp_air": 26.0 + p[11].as_f64().unwrap_or(0.0),
                "index": p[1].as_i64().unwrap_or(15),
                "humidity": p[8].as_f64().unwrap_or(0.5),
                "rain_water": p[6].as_f64().unwrap_or(0.0),
                "trans_data_A": 0.5, "trans_data_B": 1, "trans_look_B": 1, "trans_look_A": 0.5,
                "trans_auto": true,
                "rain_variance": p[5].as_f64().unwrap_or(0.0),
                "rain_water_range": 0, "mist_range": 0,
                "temp_road": 32.0 + p[12].as_f64().unwrap_or(0.0),
                "rain_amount": p[4].as_f64().unwrap_or(0.0),
                "__qarl_weather_id": step.weather  // ⭐ LA LIGNE MAGIQUE
            });

            container.push(serde_json::json!({
                "data": {
                    "weather": weather_obj,
                    "duration": duration_sec,
                    "timestamp": current_time
                }
            }));
            cumulative_seconds += duration_sec;
        }
    }

    // 3. Construire l'objet global du plan
            let plan_data = serde_json::json!({
        "container": container,
        "control": {
            "type": 2, // ⭐ 2 = Timed plan (se cale sur l'heure)
            "timemulti": time_mult,
            "loop": loop_plan
        }
    });

    // 4. Écrire les fichiers (Archive et last_used.json)
    let pure_dir = base.join("extension").join("config-ext").join("PurePlanner");
    let safe_name = plan_name.replace(" ", "_").replace(|c: char| !c.is_alphanumeric() && c != '_', "");
    
    // ⭐ Sécurité : Si le nom ne contient que des caractères spéciaux, safe_name devient vide
    if safe_name.trim().is_empty() {
        return Err("Le nom du plan est invalide (caractères spéciaux uniquement).".into());
    }

    let file_name = format!("QARL_{}.json", safe_name);
    // ⭐ Sauvegarde dans le dossier Timed
    let archive_path = pure_dir.join("Plans").join("Timed").join(&file_name);
    
    if let Some(parent) = archive_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let json_string = serde_json::to_string_pretty(&plan_data).map_err(|e| e.to_string())?;
    
    fs::write(&archive_path, &json_string).map_err(|e| e.to_string())?;
    
    let last_used_root = pure_dir.join("last_used.json");
    let last_used_plans = pure_dir.join("Plans").join("last_used.json");
    fs::write(&last_used_root, &json_string).map_err(|e| e.to_string())?;
    fs::write(&last_used_plans, &json_string).map_err(|e| e.to_string())?;

    // ⭐ Retourne le bon chemin
    Ok(format!("Timed/{}", file_name)) // Pas besoin de changer ici car file_name contient déjà QARL_
}

#[tauri::command]
fn read_pure_ctrl(documents_path: String) -> Result<PureCtrlSettings, String> {
    // ⭐ On utilise le dossier cfg et on crée QARL_Presets/PureCtrl
    let cfg_dir = resolve_cfg_dir(Path::new(&documents_path));
    let qarl_dir = cfg_dir.join("QARL_Presets").join("PureCtrl");
    if !qarl_dir.exists() {
        let _ = fs::create_dir_all(&qarl_dir);
    }
    let config_path = qarl_dir.join("purectrl_config.json");
    
    let default_settings = PureCtrlSettings { autostart: false, last_used: false, live: false, start_wetness: 1, start_puddles: 1 };
    if !config_path.exists() { return Ok(default_settings); }
    if let Ok(content) = fs::read_to_string(&config_path) {
        if let Ok(settings) = serde_json::from_str::<PureCtrlSettings>(&content) { return Ok(settings); }
    }
    Ok(default_settings)
}

#[tauri::command]
fn write_pure_ctrl(documents_path: String, settings: PureCtrlSettings) -> Result<(), String> {
    // ⭐ On écrit dans QARL_Presets/PureCtrl
    let cfg_dir = resolve_cfg_dir(Path::new(&documents_path));
    let qarl_dir = cfg_dir.join("QARL_Presets").join("PureCtrl");
    if !qarl_dir.exists() {
        let _ = fs::create_dir_all(&qarl_dir);
    }
    let config_path = qarl_dir.join("purectrl_config.json");
    
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(config_path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Serialize, serde::Deserialize, Clone)]
struct HistoryEntry {
    id: String,
    date: String,
    car_id: String,
    car_name: String,
    car_image: String,
    skin_id: String,
    track_id: String,
    track_name: String,
    layout_id: String,
    session_data: serde_json::Value,
    weather_summary: String,
}

#[derive(Serialize, Clone)]
struct Capabilities {
    has_csp: bool,
    has_pure: bool,
    has_pure_weather: bool, // ⭐ Ajouté
    has_sol: bool,
    has_acs: bool,
    csp_version: String,
    sol_version: String,
    pure_version: String,
    pure_planner_version: String,
}

#[derive(Serialize, Clone)]
struct SteamUserInfo {
    steam_id: String,
    persona_name: String,
}

#[tauri::command]
fn detect_steam_id() -> Result<SteamUserInfo, String> {
    let output = std::process::Command::new("reg")
        .args(&["query", "HKCU\\Software\\Valve\\Steam", "/v", "SteamPath"])
        .output()
        .map_err(|e| e.to_string())?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let path_line = stdout.lines().find(|l| l.contains("SteamPath")).ok_or("Steam non trouvé")?;
    let steam_path = path_line.split("REG_SZ").nth(1).unwrap_or("").trim();
    
    if steam_path.is_empty() { return Err("Chemin Steam introuvable".into()); }
    
    let login_users_path = std::path::Path::new(steam_path).join("config").join("loginusers.vdf");
    if !login_users_path.exists() { return Err("loginusers.vdf introuvable".into()); }
    
    let content = std::fs::read_to_string(login_users_path).map_err(|e| e.to_string())?;
    
    let mut current_id = String::new();
    let mut current_name = String::new();
    let mut fallback_id = String::new();
    let mut fallback_name = String::new();
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        if trimmed.starts_with('"') && trimmed.matches('"').count() == 2 {
            let val = trimmed.trim_matches('"');
            if val.len() == 17 && val.starts_with("7656119") {
                current_id = val.to_string();
                current_name = String::new();
            }
        }
        
        if trimmed.to_lowercase().starts_with("\"personaname\"") {
            let parts: Vec<&str> = trimmed.split('"').collect();
            if parts.len() >= 4 {
                current_name = parts[3].to_string();
            }
        }
        
        if current_id.len() == 17 && trimmed.contains("\"MostRecent\"") && trimmed.contains("\"1\"") {
            return Ok(SteamUserInfo { 
                steam_id: current_id.clone(), 
                persona_name: if current_name.is_empty() { "Driver".to_string() } else { current_name.clone() } 
            });
        }
        
        if fallback_id.is_empty() && current_id.len() == 17 && !current_name.is_empty() {
            fallback_id = current_id.clone();
            fallback_name = current_name.clone();
        }
    }
    
    if !fallback_id.is_empty() { 
        return Ok(SteamUserInfo { steam_id: fallback_id, persona_name: fallback_name }); 
    }
    if !current_id.is_empty() {
        return Ok(SteamUserInfo { steam_id: current_id, persona_name: "Driver".to_string() });
    }
    
    Err("Aucun SteamID trouvé".into())
}

#[tauri::command]
fn list_ac_controller_files(documents_path: String) -> Result<Vec<String>, String> {
    let base = std::path::Path::new(&documents_path);
    let mut controllers_dir = base.join("Assetto Corsa").join("cfg").join("controllers");
    if !controllers_dir.exists() {
        controllers_dir = base.join("cfg").join("controllers");
    }

    if !controllers_dir.exists() {
        return Ok(Vec::new());
    }

    let mut iguids = Vec::new();
    if let Ok(entries) = fs::read_dir(controllers_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "ini") {
                if let Some(stem) = path.file_stem() {
                    iguids.push(stem.to_string_lossy().to_string());
                }
            }
        }
    }
    Ok(iguids)
}

// ⭐ Détection automatique du chemin AC (registry Windows + fallbacks Steam)
#[tauri::command]
fn detect_ac_path() -> Result<Option<String>, String> {
    // 1. Registry WOW6432Node (64-bit)
    if let Ok(output) = std::process::Command::new("reg")
        .args(&["query", "HKLM\\SOFTWARE\\WOW6432Node\\Valve\\Steam", "/v", "InstallPath"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = stdout.lines().find(|l| l.contains("InstallPath")) {
            if let Some(path) = line.split("REG_SZ").nth(1) {
                let steam_path = path.trim();
                let ac_path = std::path::Path::new(steam_path).join("steamapps").join("common").join("assettocorsa");
                if ac_path.join("acs.exe").exists() {
                    return Ok(Some(ac_path.to_string_lossy().replace("\\", "/")));
                }
            }
        }
    }

    // 2. Registry 32-bit
    if let Ok(output) = std::process::Command::new("reg")
        .args(&["query", "HKLM\\SOFTWARE\\Valve\\Steam", "/v", "InstallPath"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = stdout.lines().find(|l| l.contains("InstallPath")) {
            if let Some(path) = line.split("REG_SZ").nth(1) {
                let steam_path = path.trim();
                let ac_path = std::path::Path::new(steam_path).join("steamapps").join("common").join("assettocorsa");
                if ac_path.join("acs.exe").exists() {
                    return Ok(Some(ac_path.to_string_lossy().replace("\\", "/")));
                }
            }
        }
    }

    // 3. Chemins Steam Library standards
    let candidates = vec![
        "C:\\Program Files (x86)\\Steam\\steamapps\\common\\assettocorsa",
        "C:\\SteamLibrary\\steamapps\\common\\assettocorsa",
        "D:\\SteamLibrary\\steamapps\\common\\assettocorsa",
        "E:\\SteamLibrary\\steamapps\\common\\assettocorsa",
        "F:\\SteamLibrary\\steamapps\\common\\assettocorsa",
    ];
    for c in candidates {
        let p = std::path::Path::new(c);
        if p.join("acs.exe").exists() {
            return Ok(Some(p.to_string_lossy().replace("\\", "/")));
        }
    }

    Ok(None)
}

// ⭐ Détection du dossier Documents/Assetto Corsa (gère OneDrive)
#[tauri::command]
fn detect_documents_ac_path() -> Result<Option<String>, String> {
    let candidates: Vec<std::path::PathBuf> = vec![
        dirs::document_dir().unwrap_or_default().join("Assetto Corsa"),
        dirs::document_dir().unwrap_or_default().join("..").join("OneDrive").join("Documents").join("Assetto Corsa"),
        dirs::document_dir().unwrap_or_default().join("..").join("OneDrive - Personal").join("Documents").join("Assetto Corsa"),
        dirs::document_dir().unwrap_or_default().join("..").join("OneDrive - Business").join("Documents").join("Assetto Corsa"),
    ];

    for c in candidates {
        let cfg = c.join("cfg");
        if cfg.exists() {
            return Ok(Some(c.to_string_lossy().replace("\\", "/")));
        }
    }
    Ok(None)
}

// ⭐ Liste les presets constructeurs AC (Fanatec, Logitech, Thrustmaster…)
#[derive(Serialize, Clone)]
struct ControllerPreset {
    filename: String,
    name: String,
    product: String,
}

#[tauri::command]
fn list_controller_presets(ac_path: String) -> Result<Vec<ControllerPreset>, String> {
    let presets_dir = std::path::Path::new(&ac_path).join("cfg").join("controllers").join("presets");
    if !presets_dir.exists() {
        return Ok(Vec::new());
    }
    let mut presets = Vec::new();
    if let Ok(entries) = fs::read_dir(presets_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "ini") {
                let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let stem = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                let mut name = stem.clone();
                let mut product = String::new();
                if let Ok(content) = fs::read_to_string(&path) {
                    let mut in_controller = false;
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with('[') && trimmed.ends_with(']') {
                            in_controller = trimmed.eq_ignore_ascii_case("[CONTROLLER]");
                            continue;
                        }
                        if !in_controller { continue; }
                        if let Some((k, v)) = trimmed.split_once('=') {
                            let key = k.trim().to_uppercase();
                            let val = v.trim().to_string();
                            if key == "NAME" { name = val; }
                            else if key == "PRODUCT" { product = val; }
                        }
                    }
                }
                presets.push(ControllerPreset { filename, name, product });
            }
        }
    }
    presets.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(presets)
}

// ⭐ Statut complet de l'installation (composite)
#[derive(Serialize, Clone)]
struct SetupStatus {
    ac_path: Option<String>,
    ac_path_source: String,
    documents_path: Option<String>,
    documents_source: String,
    steam_id: Option<String>,
    is_ac_initialized: bool,
    has_controllers: bool,
    controllers_count: u32,
    has_csp: bool,
    has_sol: bool,
    has_pure: bool,
    csp_version: String,
    sol_version: String,
    pure_version: String,
    cars_count: u32,
    tracks_count: u32,
}

#[tauri::command]
fn detect_setup_status() -> Result<SetupStatus, String> {
    // 1. AC path
    let (ac_path, ac_path_source) = match detect_ac_path()? {
        Some(p) => (Some(p), "registry".to_string()),
        None => (None, "not_found".to_string()),
    };

    // 2. Documents path
    let (documents_path, documents_source) = match detect_documents_ac_path()? {
        Some(p) => (Some(p), "auto".to_string()),
        None => (None, "not_found".to_string()),
    };

    // 3. Steam ID (on extrait juste la string de l'objet SteamUserInfo)
    let steam_id = detect_steam_id().ok().map(|info| info.steam_id);

    // 4. AC initialisé + [CONTROLLERS] ?
    let mut is_ac_initialized = false;
    let mut has_controllers = false;
    let mut controllers_count = 0;
    if let Some(docs) = &documents_path {
        let cfg = std::path::Path::new(docs).join("cfg");
        let race_ini = cfg.join("race.ini");
        let controls_ini = cfg.join("controls.ini");
        is_ac_initialized = race_ini.exists() && controls_ini.exists();
        if controls_ini.exists() {
            if let Ok(content) = fs::read_to_string(&controls_ini) {
                let mut in_controllers = false;
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with('[') && trimmed.ends_with(']') {
                        in_controllers = trimmed.eq_ignore_ascii_case("[CONTROLLERS]");
                        continue;
                    }
                    if in_controllers && trimmed.starts_with("CON") {
                        controllers_count += 1;
                    }
                }
                has_controllers = controllers_count > 0;
            }
        }
    }

    // 5. CSP / Sol / Pure
    let mut has_csp = false;
    let mut has_sol = false;
    let mut has_pure = false;
    let mut csp_version = String::from("Inconnue");
    let mut sol_version = String::from("Inconnue");
    let mut pure_version = String::from("Inconnue");

    if let Some(ac) = &ac_path {
        let base = std::path::Path::new(ac);
        has_csp = base.join("extension").join("config").exists();
        has_pure = base.join("extension").join("config-ext").join("PurePlanner").exists();
        has_sol = base.join("extension").join("weather").join("sol").join("manifest.ini").exists();

        if has_csp {
            let manifest = base.join("extension").join("config").join("data_manifest.ini");
            if let Ok(content) = fs::read_to_string(&manifest) {
                for line in content.lines() {
                    if line.to_uppercase().starts_with("SHADERS_PATCH=") {
                        csp_version = line.split('=').nth(1).unwrap_or("Inconnue").trim().to_string();
                        break;
                    }
                }
            }
        }
        if has_sol {
            let manifest = base.join("extension").join("weather").join("sol").join("manifest.ini");
            if let Ok(content) = fs::read_to_string(&manifest) {
                for line in content.lines() {
                    if line.to_uppercase().starts_with("VERSION=") {
                        sol_version = line.split('=').nth(1).unwrap_or("Inconnue").trim().to_string();
                        break;
                    }
                }
            }
        }
        
        if has_pure {
            // ⭐ On teste les 3 emplacements possibles pour la version de Pure
            let path1 = base.join("extension").join("weather").join("pure").join("manifest.ini");
            let path2 = base.join("extension").join("weather-controllers").join("pureCtrl").join("manifest.ini");
            let path3 = base.join("apps").join("lua").join("Pure").join("manifest.ini");

            let v1 = read_manifest_version(&path1, "VERSION");
            let v2 = read_manifest_version(&path2, "VERSION");
            let v3 = read_manifest_version(&path3, "VERSION");

            if v1 != "Inconnue" {
                pure_version = v1;
            } else if v2 != "Inconnue" {
                pure_version = v2;
            } else if v3 != "Inconnue" {
                pure_version = v3;
            }
        }
    }

    // 6. Counts cars / tracks
    let mut cars_count = 0;
    let mut tracks_count = 0;
    if let Some(ac) = &ac_path {
        let base = std::path::Path::new(ac);
        if let Ok(entries) = fs::read_dir(base.join("content").join("cars")) {
            cars_count = entries.filter(|e| e.as_ref().map(|x| x.path().is_dir()).unwrap_or(false)).count() as u32;
        }
        if let Ok(entries) = fs::read_dir(base.join("content").join("tracks")) {
            tracks_count = entries.filter(|e| e.as_ref().map(|x| x.path().is_dir()).unwrap_or(false)).count() as u32;
        }
    }

    Ok(SetupStatus {
        ac_path,
        ac_path_source,
        documents_path,
        documents_source,
        steam_id,
        is_ac_initialized,
        has_controllers,
        controllers_count,
        has_csp,
        has_sol,
        has_pure,
        csp_version,
        sol_version,
        pure_version,
        cars_count,
        tracks_count,
    })
}

#[tauri::command]
fn check_ac_initialized(documents_path: Option<String>) -> Result<serde_json::Value, String> {
    // ⭐ Utilise le chemin passé par Svelte s'il existe, sinon fallback sur Documents
    let docs = if let Some(p) = documents_path {
        std::path::PathBuf::from(p)
    } else {
        dirs::document_dir().ok_or("Impossible de trouver le dossier Documents")?
    };

    let mut cfg_dir = docs.join("Assetto Corsa").join("cfg");
    // Au cas où l'utilisateur ait choisi directement le dossier "Assetto Corsa" ou "cfg"
    if !cfg_dir.exists() {
        cfg_dir = docs.join("cfg");
    }

    let race_ini = cfg_dir.join("race.ini");
    let controls_ini = cfg_dir.join("controls.ini");

    let has_race = race_ini.exists();
    let has_controls = controls_ini.exists();

    let mut has_controllers = false;
    let mut controllers_count = 0;

    if has_controls {
        if let Ok(content) = fs::read_to_string(&controls_ini) {
            let mut in_controllers = false;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    in_controllers = trimmed.eq_ignore_ascii_case("[CONTROLLERS]");
                    continue;
                }
                if in_controllers && trimmed.starts_with("CON") {
                    controllers_count += 1;
                }
            }
            has_controllers = controllers_count > 0;
        }
    }

    Ok(serde_json::json!({
        "success": true,
        "initialized": has_race && has_controls && has_controllers,
        "hasRace": has_race,
        "hasControls": has_controls,
        "hasControllers": has_controllers,
        "controllersCount": controllers_count,
        "cfgDir": cfg_dir.to_string_lossy().to_string()
    }))
}

fn read_manifest_version(path: &Path, specific_key: &str) -> String {
    if !path.exists() { return "Inconnue".to_string(); }
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            let trimmed = line.trim();
            // On sépare la ligne au signe '='
            let parts: Vec<&str> = trimmed.split('=').collect();
            if parts.len() >= 2 {
                let key = parts[0].trim().to_uppercase();
                let val = parts[1].trim();
                
                if key == specific_key.to_uppercase() || 
                   (specific_key == "VERSION" && key == "VERSION") {
                    return val.to_string();
                }
            }
        }
    }
    "Inconnue".to_string()
}

#[tauri::command]
fn check_ac_capabilities(ac_path: String) -> Result<Capabilities, String> {
    let base = Path::new(&ac_path);
    
    let has_acs = base.join("acs.exe").exists();
    let has_csp = base.join("extension").join("config").exists();
    let has_pure = base.join("extension").join("config-ext").join("PurePlanner").exists();
    let has_pure_weather = base.join("extension").join("weather").join("pure").exists() || 
                           base.join("extension").join("weather").join("Pure LCS").exists();
    // ⭐ Vérification stricte de Sol
    let has_sol = base.join("extension").join("weather").join("sol").join("manifest.ini").exists();

    // ⭐ Détection des versions
    // CSP : la clé est SHADERS_PATCH
    let csp_version = read_manifest_version(&base.join("extension").join("config").join("data_manifest.ini"), "SHADERS_PATCH")
        .replace("Inconnue", &read_manifest_version(&base.join("extension").join("manifest.ini"), "VERSION"));
    
    let sol_version = read_manifest_version(&base.join("extension").join("weather").join("sol").join("manifest.ini"), "VERSION");
    
    let pure_version = read_manifest_version(&base.join("apps").join("lua").join("Pure").join("manifest.ini"), "VERSION")
        .replace("Inconnue", &read_manifest_version(&base.join("extension").join("weather").join("pure").join("manifest.ini"), "VERSION"));
    
    let pure_planner_version = read_manifest_version(&base.join("apps").join("lua").join("PurePlanner").join("manifest.ini"), "VERSION");

    Ok(Capabilities { 
        has_csp, has_pure, has_pure_weather, has_sol, has_acs,
        csp_version, sol_version, pure_version, pure_planner_version
    })
}

// ⭐ Écrit le fichier d'état de PureCtrl pour dire à Pure quel plan charger
fn write_pure_ctrl_ini(cfg_dir: &Path, plan_path: &str, cm_weather: i32, pure_ctrl: &serde_json::Value) -> Result<(), String> {
    let dir = cfg_dir.join("extension").join("state").join("lua").join("wfx_controller");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("Erreur création dossier PureCtrl: {}", e))?;
    }
    let file_path = dir.join("pureCtrl__settings.ini");
    
    let mut lines = vec!["[SETTINGS]".to_string()];
    lines.push(format!("CM_WEATHER={}", cm_weather));
    
    let mut autostart = pure_ctrl["autostart"].as_bool().unwrap_or(true);
    let last_used = pure_ctrl["lastUsed"].as_bool().unwrap_or(false);
    let live = pure_ctrl["live"].as_bool().unwrap_or(false);
    let start_wetness = pure_ctrl["startWetness"].as_i64().unwrap_or(1);
    let start_puddles = pure_ctrl["startPuddles"].as_i64().unwrap_or(1);

    // ⭐ SÉCURITÉ : Si un plan est chargé, on force l'autostart, quoi qu'il arrive.
    if !plan_path.is_empty() {
        autostart = true;
    }

    // ⭐ On force l'écriture avec les espaces et les valeurs par défaut comme Pure Planner
    lines.push(format!("LIVE = {}", if live { "1" } else { "0" }));
    lines.push(format!("LAST_USED = {}", if last_used { "1" } else { "0" }));
    lines.push(format!("AUTOSTART = {}", if autostart { "1" } else { "0" }));
    
    if start_wetness > 1 { lines.push(format!("START_WETNESS = {}", start_wetness)); }
    if start_puddles > 1 { lines.push(format!("START_PUDDLES = {}", start_puddles)); }
    
    if !plan_path.is_empty() { lines.push(format!("PLAN = {}", plan_path)); }

    let content = lines.join("\n") + "\n";
    fs::write(&file_path, content).map_err(|e| format!("Erreur écriture pureCtrl__settings.ini: {}", e))?;
    Ok(())
}

// ⭐ Récupère la correspondance CM_WEATHER (indice d'icône) pour PureCtrl
fn get_pure_ctrl_cm_weather(weather_name: &str) -> i32 {
    let id = extract_pure_id(weather_name);
    match id.as_str() {
        "no_clouds" | "noclouds" => 1, "clear" => 2, "few_clouds" | "fewclouds" => 3,
        "scattered_clouds" | "scatteredclouds" => 4, "windy" => 5, "broken_clouds" | "brokenclouds" => 6,
        "overcast" | "overcast_clouds" | "overcastclouds" => 7, "haze" => 8, "mist" => 9, "fog" => 10,
        "light_drizzle" | "lightdrizzle" => 14, "drizzle" => 15, "light_rain" | "lightrain" => 17,
        "rain" => 18, "heavy_rain" | "heavyrain" => 19, _ => 2, // Par défaut : Clear
    }
}

// ⭐ NOUVEAU : Sauvegarde manuelle du fichier assists.ini
// ⭐ Dossier des presets d'assistances
fn get_assists_presets_dir(documents_path: &str) -> Result<std::path::PathBuf, String> {
    let cfg_dir = resolve_cfg_dir(Path::new(documents_path));
    let dir = cfg_dir.join("QARL_Presets").join("Assists");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(dir)
}

#[tauri::command]
fn list_assists_presets(documents_path: String) -> Result<Vec<String>, String> {
    let dir = get_assists_presets_dir(&documents_path)?;
    let mut presets = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(stem) = entry.path().file_stem() {
                if entry.path().extension().map_or(false, |e| e == "ini") {
                    presets.push(stem.to_string_lossy().to_string());
                }
            }
        }
    }
    presets.sort();
    Ok(presets)
}

#[tauri::command]
fn save_assists_preset(documents_path: String, preset_name: String, assists: serde_json::Value, fov: i32) -> Result<(), String> {
    let cfg_dir = resolve_cfg_dir(Path::new(&documents_path));
    let get_bool = |key: &str| assists[key].as_bool().unwrap_or(false) as i32;
    let get_int = |key: &str, def: i32| assists[key].as_i64().unwrap_or(def as i64) as i32;

    let content = format!(
r#"[ASSISTS]
IDEAL_LINE={}
AUTO_BLIP={}
STABILITY_CONTROL={}
AUTO_BRAKE=0
AUTO_SHIFTER={}
ABS={}
TRACTION_CONTROL={}
AUTO_CLUTCH={}
VISUALDAMAGE=100
DAMAGE={}
FUEL_RATE={}
TYRE_WEAR={}
TYRE_BLANKETS={}
SLIPSTREAM={}

[CAMERA]
FOV={}
"#,
        get_bool("idealLine"), get_bool("autoblip"), get_int("stability", 0), 
        get_bool("autoGearbox"), get_int("abs", 1), get_int("tc", 1), get_bool("autoClutch"),
        get_int("mechanicalDamage", 100), get_int("fuelRate", 1), get_int("tyreWear", 1), 
        get_bool("tyreBlankets"), get_int("slipstream", 1), fov
    );

    // 1. Sauvegarde du profil
    let presets_dir = get_assists_presets_dir(&documents_path)?;
    let safe_name = preset_name.trim().replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");
    if safe_name.is_empty() { return Err("Nom invalide".into()); }
    fs::write(presets_dir.join(format!("{}.ini", safe_name)), &content).map_err(|e| e.to_string())?;

    // 2. Application immédiate dans le fichier global d'AC
    fs::write(cfg_dir.join("assists.ini"), content).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
fn load_assists_preset(documents_path: String, preset_name: String) -> Result<serde_json::Value, String> {
    let cfg_dir = resolve_cfg_dir(Path::new(&documents_path));
    let presets_dir = get_assists_presets_dir(&documents_path)?;
    let file_path = presets_dir.join(format!("{}.ini", preset_name));
    
    if !file_path.exists() { return Err("Preset introuvable".into()); }
    let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    
    // On copie le preset vers le fichier actif d'AC
    fs::write(cfg_dir.join("assists.ini"), &content).map_err(|e| e.to_string())?;

        // On parse pour renvoyer à Svelte
    let mut result = serde_json::json!({});
    let mut in_camera_section = false;
    
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_camera_section = trimmed.eq_ignore_ascii_case("[CAMERA]");
            continue;
        }
        if trimmed.starts_with(';') || trimmed.is_empty() { continue; }
        
        if let Some((k, v)) = trimmed.split_once('=') {
            let val_str = v.split(';').next().unwrap_or("").trim();
            
            if in_camera_section && k.trim() == "FOV" {
                result["fov"] = serde_json::json!(val_str.parse::<i32>().unwrap_or(56));
                continue;
            }
            
            match k.trim() {
                "IDEAL_LINE" => result["idealLine"] = serde_json::json!(val_str == "1"),
                "AUTO_BLIP" => result["autoblip"] = serde_json::json!(val_str == "1"),
                "STABILITY_CONTROL" => result["stability"] = serde_json::json!(val_str.parse::<i32>().unwrap_or(0)),
                "AUTO_SHIFTER" => result["autoGearbox"] = serde_json::json!(val_str == "1"),
                "ABS" => result["abs"] = serde_json::json!(val_str.parse::<i32>().unwrap_or(1)),
                "TRACTION_CONTROL" => result["tc"] = serde_json::json!(val_str.parse::<i32>().unwrap_or(1)),
                "AUTO_CLUTCH" => result["autoClutch"] = serde_json::json!(val_str == "1"),
                "DAMAGE" => result["mechanicalDamage"] = serde_json::json!(val_str.parse::<i32>().unwrap_or(100)),
                "FUEL_RATE" => result["fuelRate"] = serde_json::json!(val_str.parse::<i32>().unwrap_or(1)),
                "TYRE_WEAR" => result["tyreWear"] = serde_json::json!(val_str.parse::<i32>().unwrap_or(1)),
                "TYRE_BLANKETS" => result["tyreBlankets"] = serde_json::json!(val_str == "1"),
                "SLIPSTREAM" => result["slipstream"] = serde_json::json!(val_str.parse::<i32>().unwrap_or(1)),
                _ => {}
            }
        }
    }
    Ok(result)
}

#[tauri::command]
fn delete_assists_preset(documents_path: String, preset_name: String) -> Result<(), String> {
    let presets_dir = get_assists_presets_dir(&documents_path)?;
    let file_path = presets_dir.join(format!("{}.ini", preset_name));
    
    if file_path.exists() {
        fs::remove_file(file_path).map_err(|e| format!("Erreur suppression: {}", e))?;
    }
    Ok(())
}

#[allow(non_snake_case)]
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct SessionData {
    #[serde(rename = "type")]
    session_type: String,
    laps: i32,
    practiceMinutes: i32,
    qualifyMinutes: i32,
    sessionMinutes: i32,
    opponentsCount: i32,
    weather: serde_json::Value,
    assists: serde_json::Value,
    startingPositionMode: Option<String>,
    customStartPos: Option<i32>,
    aiLevel: Option<i32>,
    aiLevelVar: Option<i32>,
    aiAggression: Option<i32>,
    aiAggressionVar: Option<i32>,
    fixedSetup: Option<bool>,
    ballast: Option<i32>,
    restrictor: Option<i32>,
    allowedTyresOut: Option<i32>,
    trackGrip: Option<i32>,
    windSpeed: Option<i32>,
    windDirection: Option<i32>,
    ghostCar: Option<bool>,
}

#[derive(serde::Deserialize, Clone)]
struct AIData {
    id: String,
    driver: String,
}

// ⭐ NOUVEAU : Gestion des fichiers ext_config.ini temporaires pour Pure
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct InjectedConfigs {
    paths: Vec<String>,
}

fn get_injected_configs_path() -> std::path::PathBuf {
    let docs = dirs::document_dir().unwrap_or_default();
    docs.join("Assetto Corsa").join("cfg").join("QARL_injected_configs.json")
}

#[tauri::command]
fn cleanup_injected_track_configs() {
    let json_path = get_injected_configs_path();
    if !json_path.exists() { return; }

    if let Ok(content) = fs::read_to_string(&json_path) {
        if let Ok(data) = serde_json::from_str::<InjectedConfigs>(&content) {
            for path_str in data.paths {
                let p = Path::new(&path_str);
                if p.exists() {
                    let _ = fs::remove_file(p);
                    println!("🧹 RUST : Fichier temporaire supprimé : {}", path_str);
                }
            }
        }
    }
    // On supprime le fichier de tracking après le nettoyage
    let _ = fs::remove_file(json_path);
}

fn inject_track_ext_config(app_handle: &tauri::AppHandle, ac_path: &str, track_id: &str) -> Result<(), String> {
    let track_ext_dir = Path::new(ac_path).join("content").join("tracks").join(track_id).join("extension");
    let config_path = track_ext_dir.join("ext_config.ini");

    // Si le fichier existe déjà, on ne touche à RIEN (config du moddeur)
    if config_path.exists() {
        return Ok(());
    }

    // ⭐ On lit le fichier depuis les ressources de Tauri
    let resource_path = app_handle.path().resolve("resources/track_ext_config.ini", tauri::path::BaseDirectory::Resource)
        .map_err(|e| format!("Chemin ressource introuvable: {}", e))?;

    if !resource_path.exists() {
        return Err(format!("Fichier template introuvable dans les ressources : {:?}", resource_path));
    }

    let pure_config = fs::read_to_string(&resource_path)
        .map_err(|e| format!("Erreur lecture template ext_config: {}", e))?;

    // Création du dossier extension s'il n'existe pas
    if !track_ext_dir.exists() {
        fs::create_dir_all(&track_ext_dir).map_err(|e| e.to_string())?;
    }

    // Écriture du fichier
    fs::write(&config_path, pure_config).map_err(|e| format!("Erreur d'injection ext_config: {}", e))?;
    println!("✨ RUST : Fichier ext_config.ini injecté pour la map {}", track_id);

    // Ajout dans le fichier de tracking
    let json_path = get_injected_configs_path();
    let mut tracking = if json_path.exists() {
        fs::read_to_string(&json_path).ok()
            .and_then(|c| serde_json::from_str::<InjectedConfigs>(&c).ok())
            .unwrap_or_default()
    } else {
        InjectedConfigs::default()
    };

    let path_string = config_path.to_string_lossy().to_string();
    if !tracking.paths.contains(&path_string) {
        tracking.paths.push(path_string);
        if let Ok(json) = serde_json::to_string_pretty(&tracking) {
            let _ = fs::write(json_path, json);
        }
    }

    Ok(())
}

#[tauri::command]
fn launch_race(
    app_handle: tauri::AppHandle, // ⭐ Ajouté ici
    ac_path: String,
    documents_path: String, // ⭐ NOUVEAU : respecte le documentsPath configuré
    car_id: String,
    skin_id: String,
    track_id: String,
    layout_id: String,
    session_data: SessionData,
    ai_grid: Vec<AIData>,
) -> Result<(), String> {
    // ⭐ Résolution du dossier cfg (respecte documents_path, fallback sur dirs si vide)
    let cfg_dir = if !documents_path.is_empty() {
        resolve_cfg_dir(Path::new(&documents_path))
    } else {
        let docs_fallback = dirs::document_dir().ok_or("Impossible de trouver le dossier Documents")?;
        docs_fallback.join("Assetto Corsa").join("cfg")
    };
    
    if !cfg_dir.exists() {
        fs::create_dir_all(&cfg_dir).map_err(|e| format!("Erreur création dossier cfg: {}", e))?;
    }

    let race_ini_path = cfg_dir.join("race.ini");

    let track_name = track_id.clone();
    let config_track = if layout_id.is_empty() 
        || layout_id == track_id 
        || layout_id.to_lowercase() == "défaut" 
        || layout_id.to_lowercase() == "default" 
    {
        String::new()
    } else {
        layout_id.clone()
    };

    // ⭐ GESTION DE LA MÉTÉO
    let mut weather_mode = session_data.weather["mode"].as_str().unwrap_or("static");
    let mut weather_name = session_data.weather["staticWeather"].as_str().unwrap_or("3_clear").to_string();
    let mut cm_weather = get_pure_ctrl_cm_weather(&weather_name);
    let mut plan_value = String::new();
    let mut use_schedule = false;

    // ⭐ FIX QARL : Le frontend envoie "pure_plan" mais Rust attend "existing_plan" ou "custom_plan"
    if weather_mode == "pure_plan" {
        // Si un plan existant est sélectionné, on le traite comme tel
        if !session_data.weather["existingPlan"].as_str().unwrap_or("").is_empty() {
            weather_mode = "existing_plan";
        } 
        // Sinon, si on a des étapes personnalisées, c'est un plan custom
        else if session_data.weather["customPlanSteps"].as_array().map_or(false, |a| !a.is_empty()) {
            weather_mode = "custom_plan";
        }
    }

    // ⭐ Si c'est le mode Schedule, on prépare la config
    if weather_mode == "schedule" {
        use_schedule = true;
        let state_dir = cfg_dir.join("extension").join("state").join("lua").join("weather_controller");
        
        if !state_dir.exists() {
            fs::create_dir_all(&state_dir).map_err(|e| format!("Erreur création dossier weather_controller: {}", e))?;
        }

        let slots = session_data.weather["scheduleSlots"].as_i64().unwrap_or(3);
        let time_per_slot = session_data.weather["scheduleTimePerSlot"].as_i64().unwrap_or(5);
        let random = session_data.weather["scheduleRandom"].as_bool().unwrap_or(false);

        let mut ini_content = String::from("[SETTINGS]\n");
        ini_content.push_str(&format!("SLOTS={}\n", slots));
        ini_content.push_str(&format!("TIME_PER_SLOT={}\n", time_per_slot));
        ini_content.push_str(&format!("RANDOM={}\n", if random { "1" } else { "0" }));

        for i in 0..slots {
            // On récupère l'index du slot (15 par défaut si non trouvé)
            let slot_val = session_data.weather["scheduleSlotValues"][i as usize].as_i64().unwrap_or(15);
            ini_content.push_str(&format!("SLOT_{}={}\n", i, slot_val));
        }

        let schedule_file = state_dir.join("base-schedule.ini");
        fs::write(&schedule_file, ini_content).map_err(|e| format!("Erreur écriture base-schedule.ini: {}", e))?;
        
        // On force l'index de la première météo pour l'icône de CM
        if let Some(first_slot) = session_data.weather["scheduleSlotValues"][0].as_i64() {
            cm_weather = first_slot as i32;
        }
    }

    let pure_dir = Path::new(&ac_path).join("extension").join("config-ext").join("PurePlanner");
    let presets_path = pure_dir.join("WeatherPresets").join("Default_WeatherPresets.json");
    
    let presets: Vec<Vec<serde_json::Value>> = fs::read_to_string(&presets_path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default();

    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let midnight = now - (now % 86400);
    let start_hour = session_data.weather["startHour"].as_i64().unwrap_or(14);
    let start_min = session_data.weather["startMinute"].as_i64().unwrap_or(0);
    let base_timestamp = midnight + (start_hour * 3600) + (start_min * 60);

    if weather_mode == "custom_plan" {
        // ⭐ Sécurité : Si le nom est vide, on lui donne un nom par défaut
        let mut plan_name = session_data.weather["customPlanName"].as_str().unwrap_or("MonPlan").to_string();
        if plan_name.trim().is_empty() {
            plan_name = "Plan_Auto".to_string();
        }
        
        let plan_loop = session_data.weather["customPlanLoop"].as_bool().unwrap_or(false);
        let time_mult = session_data.weather["timeMultiplier"].as_i64().unwrap_or(1) as i32;
        let mut container = Vec::new();
        let mut cumulative: i64 = 0;

        if let Some(steps) = session_data.weather["customPlanSteps"].as_array() {
            if let Some(first_step) = steps.first() {
                if let Some(w) = first_step["weather"].as_str() { weather_name = w.to_string(); }
            }
            for step in steps {
                let w_id = step["weather"].as_str().unwrap_or("clear");
                let duration = step["duration"].as_i64().unwrap_or(10);
                let pure_id = extract_pure_id(w_id);
                let mut preset_data = presets.iter().find(|p| p[0].as_str().unwrap_or("") == pure_id);
                if preset_data.is_none() { preset_data = presets.iter().find(|p| p[0].as_str().unwrap_or("") == "clear"); }

               if let Some(p) = preset_data {
                    let duration_sec = (duration * 60) as i64;
                    let weather_obj = serde_json::json!({
                        "wind_strength": p[7].as_f64().unwrap_or(0.0), "rain_probability": p[2].as_f64().unwrap_or(0.0),
                        "temp_road_dyn": true, "temp_air_dyn": true, "mist_dyn": true, "humidity_dyn": true,
                        "rain_water_dyn": true, "rain_wetness_dyn": true, "rain_amount_dyn": true,
                        "temp_air_range": 0, "wind_direction": 0, "humidity_range": 0,
                        "rain_wetness": p[3].as_f64().unwrap_or(0.0), "rain_wetness_range": 0,
                        "rain_variance_range": 0, "rain_probability_range": 0, "rain_amount_range": 0,
                        "wind_strength_range": 0, "wind_direction_range": 0, "temp_road_range": 0,
                        "mist": p[9].as_f64().unwrap_or(0.0), "temp_air": 26.0 + p[11].as_f64().unwrap_or(0.0),
                        "index": p[1].as_i64().unwrap_or(15), "humidity": p[8].as_f64().unwrap_or(0.5),
                        "rain_water": p[6].as_f64().unwrap_or(0.0), "trans_data_A": 0.5, "trans_data_B": 1,
                        "trans_look_B": 1, "trans_look_A": 0.5, "trans_auto": true,
                        "rain_variance": p[5].as_f64().unwrap_or(0.0), "rain_water_range": 0, "mist_range": 0,
                        "temp_road": 32.0 + p[12].as_f64().unwrap_or(0.0), "rain_amount": p[4].as_f64().unwrap_or(0.0),
                        "__qarl_weather_id": w_id
                    });
                    container.push(serde_json::json!({ "data": { "weather": weather_obj, "duration": duration_sec, "timestamp": base_timestamp + cumulative } }));
                    cumulative += duration_sec;
                }
            }
        }
        let plan_data = serde_json::json!({ "container": container, "control": { "type": 2, "timemulti": time_mult, "loop": plan_loop } }); // ⭐ Type 2
        let mut safe_name = plan_name.replace(" ", "_").replace(|c: char| !c.is_alphanumeric() && c != '_', "");
        
        // ⭐ Sécurité : Si le nettoyage a tout supprimé
        if safe_name.trim().is_empty() {
            safe_name = "Plan_Auto".to_string();
        }

        let file_name = format!("QARL_{}.json", safe_name);
        let archive_path = pure_dir.join("Plans").join("Timed").join(&file_name); // ⭐ Dossier Timed
        if let Some(parent) = archive_path.parent() { let _ = fs::create_dir_all(parent); }
        let json_str = serde_json::to_string_pretty(&plan_data).unwrap_or_default();
        let _ = fs::write(&archive_path, &json_str);
        let _ = fs::write(pure_dir.join("last_used.json"), &json_str);
        let _ = fs::write(pure_dir.join("Plans").join("last_used.json"), &json_str);
        plan_value = format!("Timed/QARL_{}", safe_name); // ⭐ Chemin Timed
    } else if weather_mode == "existing_plan" {
        let existing_plan = session_data.weather["existingPlan"].as_str().unwrap_or("");
        if !existing_plan.is_empty() {
            let clean_plan = existing_plan.replace(".json", "");
            let plan_path = pure_dir.join("Plans").join(format!("{}.json", clean_plan));
            if let Ok(content) = fs::read_to_string(&plan_path) {
                if let Ok(mut parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(ctrl) = parsed.get_mut("control") {
                        ctrl["timemulti"] = serde_json::json!(session_data.weather["timeMultiplier"].as_i64().unwrap_or(1));
                        ctrl["loop"] = serde_json::json!(session_data.weather["customPlanLoop"].as_bool().unwrap_or(false));
                    }
                    let json_str = serde_json::to_string_pretty(&parsed).unwrap_or_default();
                    let _ = fs::write(pure_dir.join("last_used.json"), &json_str);
                    let _ = fs::write(pure_dir.join("Plans").join("last_used.json"), &json_str);
                    plan_value = clean_plan.clone();
                }
            }
        }
    } else {
        if weather_name.starts_with("pure_") || weather_name.starts_with("sol_") {
            let pure_id = extract_pure_id(&weather_name);
            if let Some(p) = presets.iter().find(|p| p[0].as_str().unwrap_or("") == pure_id) {
                let plan_data = serde_json::json!({
                    "container": [{
                        "data": {
                            "weather": {
                                "wind_strength": p[7].as_f64().unwrap_or(0.0), "rain_probability": p[2].as_f64().unwrap_or(0.0),
                                "temp_road_dyn": true, "temp_air_dyn": true, "mist_dyn": true, "humidity_dyn": true,
                                "rain_water_dyn": true, "rain_wetness_dyn": true, "rain_amount_dyn": true,
                                "temp_air_range": 0, "wind_direction": 0, "humidity_range": 0,
                                "rain_wetness": p[3].as_f64().unwrap_or(0.0), "rain_wetness_range": 0,
                                "rain_variance_range": 0, "rain_probability_range": 0, "rain_amount_range": 0,
                                "wind_strength_range": 0, "wind_direction_range": 0, "temp_road_range": 0,
                                "mist": p[9].as_f64().unwrap_or(0.0), "temp_air": 26.0 + p[11].as_f64().unwrap_or(0.0),
                                "index": p[1].as_i64().unwrap_or(15), "humidity": p[8].as_f64().unwrap_or(0.5),
                                "rain_water": p[6].as_f64().unwrap_or(0.0), "trans_data_A": 0.5, "trans_data_B": 1,
                                "trans_look_B": 1, "trans_look_A": 0.5, "trans_auto": true,
                                "rain_variance": p[5].as_f64().unwrap_or(0.0), "rain_water_range": 0, "mist_range": 0,
                                "temp_road": 32.0 + p[12].as_f64().unwrap_or(0.0), "rain_amount": p[4].as_f64().unwrap_or(0.0)
                            },
                            "duration": 86400,
                            "timestamp": base_timestamp
                        }
                    }],
                    "control": { "type": 1, "timemulti": session_data.weather["timeMultiplier"].as_i64().unwrap_or(1), "loop": true }
                });
                let file_name = format!("QARL_Static_{}.json", pure_id);
                let archive_path = pure_dir.join("Plans").join("Daycycle").join(&file_name);
                if let Some(parent) = archive_path.parent() { let _ = fs::create_dir_all(parent); }
                let json_str = serde_json::to_string_pretty(&plan_data).unwrap_or_default();
                let _ = fs::write(&archive_path, &json_str);
                let _ = fs::write(pure_dir.join("last_used.json"), &json_str);
                let _ = fs::write(pure_dir.join("Plans").join("last_used.json"), &json_str);
                plan_value = format!("Daycycle/QARL_Static_{}", pure_id);
            }
        }
    }

    let pure_ctrl = &session_data.weather["pureCtrl"];
    let _ = write_pure_ctrl_ini(&cfg_dir, &plan_value, cm_weather, pure_ctrl);

    let time_mult = session_data.weather["timeMultiplier"].as_i64().unwrap_or(1) as i32;
    let start_hour = session_data.weather["startHour"].as_f64().unwrap_or(14.0) as f32;
    let start_minute = session_data.weather["startMinute"].as_f64().unwrap_or(0.0) as f32;
    let time_decimal: f32 = start_hour + (start_minute / 60.0);
    let sun_azimuth: f32 = (((time_decimal - 6.0) * 15.0) % 360.0 + 360.0) % 360.0;
    let sun_elevation: f32 = 16.0 * (time_decimal - 13.0);
    let ambient_temp: f32 = 26.0;
    let time_angle = (time_decimal - 6.0) * std::f32::consts::PI / 12.0;
    let temp_variation = time_angle.sin().max(0.0) * 15.0;
    let road_temp = ambient_temp + temp_variation;

    let custom_pos_str = session_data.customStartPos.unwrap_or(1).to_string();
    let start_pos = match session_data.startingPositionMode.as_deref() {
        Some("FIRST") => "1", Some("SECOND") => "2", Some("THIRD") => "3",
        Some("CUSTOM") => custom_pos_str.as_str(), Some("RANDOM") => "0", _ => "-1",
    };

    let assists = &session_data.assists;
    let get_bool = |key: &str| assists[key].as_bool().unwrap_or(false) as i32;
    let get_int = |key: &str, def: i32| assists[key].as_i64().unwrap_or(def as i64) as i32;

    let auto_gear = get_bool("autoGearbox");
    let auto_clutch = get_bool("autoClutch");
    let auto_blip = get_bool("autoblip");
    let ideal_line = get_bool("idealLine");
    let tc = get_int("tc", 1);
    let abs = get_int("abs", 1);
    let stability = get_int("stability", 0);
    let fuel_rate = get_int("fuelRate", 1);
    let damage = get_int("mechanicalDamage", 100);
    let tyre_wear = get_int("tyreWear", 1);
    let slipstream = get_int("slipstream", 1);
    let tyre_blankets = get_bool("tyreBlankets");
    let penalties = get_bool("penalties") as i32;

    // ⭐ NOUVEAU : Écriture du fichier assists.ini pour synchroniser les réglages globaux
    let assists_ini_content = format!(
r#"[ASSISTS]
IDEAL_LINE={ideal_line}
AUTO_BLIP={auto_blip}
STABILITY_CONTROL={stability}
AUTO_BRAKE=0
AUTO_SHIFTER={auto_gear}
ABS={abs}
TRACTION_CONTROL={tc}
AUTO_CLUTCH={auto_clutch}
VISUALDAMAGE=100
DAMAGE={damage}
FUEL_RATE={fuel_rate}
TYRE_WEAR={tyre_wear}
TYRE_BLANKETS={tyre_blankets}
SLIPSTREAM={slipstream}
"#,
        ideal_line = ideal_line,
        auto_blip = auto_blip,
        stability = stability,
        auto_gear = auto_gear,
        abs = abs,
        tc = tc,
        auto_clutch = auto_clutch,
        damage = damage,
        fuel_rate = fuel_rate,
        tyre_wear = tyre_wear,
        tyre_blankets = tyre_blankets,
        slipstream = slipstream
    );

    let assists_ini_path = cfg_dir.join("assists.ini");
    let _ = fs::write(&assists_ini_path, assists_ini_content);

    // ⭐ Nouvelles options
    let fixed_setup = session_data.fixedSetup.unwrap_or(false) as i32;
    let ballast = session_data.ballast.unwrap_or(0);
    let restrictor = session_data.restrictor.unwrap_or(0);
    let allowed_tyres_out = session_data.allowedTyresOut.unwrap_or(-1);
    let ghost_car = session_data.ghostCar.unwrap_or(false) as i32;
    let wind_speed = session_data.windSpeed.unwrap_or(0);
    let wind_direction = session_data.windDirection.unwrap_or(0);

    // ⭐ Logique d'adhérence
    let grip = session_data.trackGrip.unwrap_or(1);
    let (track_start, track_transfer) = match grip {
        0 => (90, 30),  // Faible
        2 => (100, 80), // Élevé
        _ => (100, 50), // Normal
    };
    
    // ⭐ Récupération des valeurs IA depuis l'interface
    let ai_level = session_data.aiLevel.unwrap_or(90);
    let ai_aggression = session_data.aiAggression.unwrap_or(50);

    // ⭐ On détermine le type de session AVANT de compter les voitures
    let session_type = session_data.session_type.as_str();
    
    // ⭐ On n'ajoute les IA QUE si c'est une COURSE
    let is_race = session_type == "RACE";
    
    let mut cars_ini = String::new();
    if is_race {
        for (i, ai) in ai_grid.iter().enumerate() {
            cars_ini.push_str(&format!(
                "[CAR_{}]\nMODEL={}\nMODEL_CONFIG=\nSKIN=\nDRIVER_NAME={}\nNATIONALITY=\nAI_LEVEL={}\n\n", 
                i + 1, ai.id, ai.driver, ai_level
            ));
        }
    }

    // ⭐ Si Practice ou Hotlap, on force 1 voiture (la nôtre)
    let total_cars = if is_race { ai_grid.len() + 1 } else { 1 };
    
    // ⭐ On force le nombre d'adversaires à 0 pour Practice et Hotlap
    let final_opponents = if is_race { session_data.opponentsCount } else { 0 };

    // ⭐ On choisit le bon contrôleur
    let weather_controller = if use_schedule { "base-schedule" } else { "pureCtrl" };

    // ⭐ CONSTRUCTION DYNAMIQUE DES SESSIONS
    let mut sessions_str = String::new();
    let mut session_idx = 0;
    let session_type = session_data.session_type.as_str();

    if session_type == "RACE" {
        if session_data.practiceMinutes > 0 {
            sessions_str.push_str(&format!(
                "[SESSION_{}]\nNAME=Essais libres\nTYPE=1\nDURATION_MINUTES={}\nSPAWN_SET=PIT\n\n",
                session_idx, session_data.practiceMinutes
            ));
            session_idx += 1;
        }
        if session_data.qualifyMinutes > 0 {
            sessions_str.push_str(&format!(
                "[SESSION_{}]\nNAME=Qualifications\nTYPE=2\nDURATION_MINUTES={}\nSPAWN_SET=PIT\n\n",
                session_idx, session_data.qualifyMinutes
            ));
            session_idx += 1;
        }
        sessions_str.push_str(&format!(
            "[SESSION_{}]\nNAME=Course\nTYPE=3\nLAPS={}\nSPAWN_SET=START\n\n",
            session_idx, session_data.laps
        ));
    } else if session_type == "PRACTICE" {
        sessions_str.push_str(&format!(
            "[SESSION_0]\nNAME=Practice\nTYPE=1\nDURATION_MINUTES={}\nSPAWN_SET=PIT\n\n",
            session_data.sessionMinutes
        ));
    } else if session_type == "HOTLAP" {
        sessions_str.push_str(&format!(
            "[SESSION_0]\nNAME=Hotlap\nTYPE=4\nDURATION_MINUTES={}\nSPAWN_SET=HOTLAP_START\n\n",
            session_data.sessionMinutes
        ));
    }

        let race_ini_content = format!(
r#"[RACE]
TRACK={track}
CONFIG_TRACK={config_track}
CARS={total_cars}
MODEL={car}
MODEL_CONFIG=
SKIN={skin}
AI_LEVEL={ai_level}
FIXED_SETUP={fixed_setup}
PENALTIES={penalties}
SUN_ANGLE={sun_azimuth:.2}
TIME_MULT={time_mult}
CLOUD_SPEED=0.200

[GHOST_CAR]
RECORDING={ghost_car}
PLAYING={ghost_car}
SECONDS_ADVANTAGE=0
LOAD=1
FILE=

[REPLAY]
FILENAME=
ACTIVE=0

[LIGHTING]
SUN_ANGLE={sun_elevation:.2}
TIME_MULT={time_mult}
CLOUD_SPEED=0.200
__CM_WEATHER_CONTROLLER={weather_controller}
__CM_WEATHER_TYPE={cm_weather}

[GROOVE]
VIRTUAL_LAPS=10
MAX_LAPS=30
STARTING_LAPS=0

[DYNAMIC_TRACK]
SESSION_START={track_start}
SESSION_TRANSFER={track_transfer}
RANDOMNESS=0
LAP_GAIN=1

[REMOTE]
ACTIVE=0
SERVER_IP=
SERVER_PORT=
NAME=
TEAM=
GUID=
REQUESTED_CAR=
PASSWORD=

[LAP_INVALIDATOR]
ALLOWED_TYRES_OUT={allowed_tyres_out}

{sessions_block}

[WEATHER]
NAME={weather}
WIND_SPEED_KMH={wind_speed}
WIND_DIRECTION_DEG={wind_direction}

[TEMPERATURE]
AMBIENT={ambient_temp:.1}
ROAD={road_temp:.1}

[WIND]
DIRECTION_DEG=-1
SPEED_KMH_MAX=40
SPEED_KMH_MIN=2

[ASSISTS]
ABS={abs}
AUTO_BLIP={auto_blip}
AUTO_CLUTCH={auto_clutch}
AUTO_GEAR={auto_gear}
DAMAGE_MULTIPLIER={damage}
FUEL_RATE={fuel_rate}
IDEAL_LINE={ideal_line}
SLIPSTREAM={slipstream}
STABILITY_CONTROL={stability}
TC={tc}
TYRE_BLANKETS={tyre_blankets}
TYRE_WEAR={tyre_wear}

[AI]
COUNT={opponents}
AI_LEVEL={ai_level}
AI_AGGRESSION={ai_aggression}

[CAR_0]
MODEL={car}
MODEL_CONFIG=
SKIN={skin}
DRIVER_NAME=
NATIONALITY=
AI_LEVEL={ai_level}
STARTING_POSITION={start_pos}
BALLAST={ballast}
RESTRICTOR={restrictor}

{cars_ia}
"#,
        track = track_name,
        config_track = config_track,
        car = car_id,
        skin = skin_id,
        start_pos = start_pos,
        penalties = penalties,
        fixed_setup = fixed_setup,
        ghost_car = ghost_car,
        track_start = track_start,
        track_transfer = track_transfer,
        allowed_tyres_out = allowed_tyres_out,
        wind_speed = wind_speed,
        wind_direction = wind_direction,
        ballast = ballast,
        restrictor = restrictor,
        ai_level = ai_level,
        ai_aggression = ai_aggression,
        sun_azimuth = sun_azimuth,
        sun_elevation = sun_elevation,
        time_mult = time_mult,
        sessions_block = sessions_str,
        weather = weather_name,
        cm_weather = cm_weather,
        abs = abs,
        auto_blip = auto_blip,
        auto_clutch = auto_clutch,
        auto_gear = auto_gear,
        damage = damage,
        fuel_rate = fuel_rate,
        ideal_line = ideal_line,
        slipstream = slipstream,
        stability = stability,
        tc = tc,
        tyre_blankets = tyre_blankets,
        tyre_wear = tyre_wear,
        opponents = final_opponents,
        total_cars = total_cars,
        cars_ia = cars_ini,
        ambient_temp = ambient_temp,
        road_temp = road_temp
    );

    fs::write(&race_ini_path, race_ini_content)
        .map_err(|e| format!("Erreur écriture race.ini: {}", e))?;

    // ⭐ SAUVEGARDE DANS L'HISTORIQUE
    let history_file = cfg_dir.join("QARL_history.json");
    let mut history: Vec<HistoryEntry> = if history_file.exists() {
        fs::read_to_string(&history_file)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let date_str = format!("{}", now); // On formatera côté Svelte pour faire joli

    let weather_summary = if weather_mode == "static" {
        weather_name.clone()
    } else if weather_mode == "existing_plan" {
        format!("Plan: {}", session_data.weather["existingPlan"].as_str().unwrap_or("Inconnu"))
    } else {
        format!("Plan: {}", session_data.weather["customPlanName"].as_str().unwrap_or("Perso"))
    };

    history.insert(0, HistoryEntry {
        id: format!("{}", now),
        date: date_str,
        car_id: car_id.clone(),
        car_name: car_id.clone(), // Idéalement le vrai nom, on fera au mieux
        car_image: "".to_string(), // Le frontend gérera l'image via l'ID
        skin_id: skin_id.clone(),
        track_id: track_id.clone(),
        track_name: track_id.clone(),
        layout_id: layout_id.clone(),
        session_data: serde_json::to_value(&session_data).unwrap_or(serde_json::json!({})),
        weather_summary,
    });

    // On garde max 50 entrées pour pas que le fichier grossisse à l'infini
    if history.len() > 50 { history.truncate(50); }

    if let Ok(json) = serde_json::to_string_pretty(&history) {
        let _ = fs::write(history_file, json);
    }

    // ⭐ Injection du fichier ext_config.ini si nécessaire
    if let Err(e) = inject_track_ext_config(&app_handle, &ac_path, &track_id) {
        println!("⚠️ RUST : Impossible d'injecter ext_config.ini : {}", e);
    }

    // ⭐ SUPPRESSION DU SWITCH AUTOMATIQUE
    // QARL ne touche plus à weather_fx.ini au lancement pour éviter les conflits avec Pure.

    let ac_exe = Path::new(&ac_path).join("acs.exe");
    if !ac_exe.exists() {
        return Err(format!("acs.exe introuvable dans le dossier : {}", ac_path));
    }

    let mut cmd = std::process::Command::new(ac_exe);
    cmd.current_dir(&ac_path);
    cmd.env("SteamAppId", "244210");
    cmd.env("SteamGameId", "244210");
    
    // ⭐ NOUVEAU : On attend la fermeture du jeu en arrière-plan
    let mut child = cmd.spawn().map_err(|e| format!("Erreur lors du lancement du jeu: {}", e))?;
    let handle = app_handle.clone();
    std::thread::spawn(move || {
        let _ = child.wait();
        let _ = handle.emit("ac-closed", ());
    });

    Ok(())
}

#[tauri::command]
fn read_race_history(documents_path: String) -> Result<Vec<HistoryEntry>, String> {
    let cfg_dir = if !documents_path.is_empty() {
        resolve_cfg_dir(Path::new(&documents_path))
    } else {
        let docs_fallback = dirs::document_dir().ok_or("Impossible de trouver le dossier Documents")?;
        docs_fallback.join("Assetto Corsa").join("cfg")
    };
    let history_file = cfg_dir.join("QARL_history.json");
    
    if !history_file.exists() { return Ok(Vec::new()); }
    
    let content = fs::read_to_string(history_file).map_err(|e| e.to_string())?;
    let history: Vec<HistoryEntry> = serde_json::from_str(&content).unwrap_or_default();
    Ok(history)
}

#[tauri::command]
fn clear_race_history(documents_path: String) -> Result<(), String> {
    let cfg_dir = if !documents_path.is_empty() {
        resolve_cfg_dir(Path::new(&documents_path))
    } else {
        let docs_fallback = dirs::document_dir().ok_or("Impossible de trouver le dossier Documents")?;
        docs_fallback.join("Assetto Corsa").join("cfg")
    };
    let history_file = cfg_dir.join("QARL_history.json");
    if history_file.exists() {
        fs::remove_file(history_file).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn delete_history_entry(documents_path: String, entry_id: String) -> Result<(), String> {
    let cfg_dir = if !documents_path.is_empty() {
        resolve_cfg_dir(Path::new(&documents_path))
    } else {
        let docs_fallback = dirs::document_dir().ok_or("Impossible de trouver le dossier Documents")?;
        docs_fallback.join("Assetto Corsa").join("cfg")
    };
    let history_file = cfg_dir.join("QARL_history.json");
    
    if history_file.exists() {
        let content = fs::read_to_string(&history_file).map_err(|e| e.to_string())?;
        let mut history: Vec<HistoryEntry> = serde_json::from_str(&content).unwrap_or_default();
        history.retain(|h| h.id != entry_id);
        let json = serde_json::to_string_pretty(&history).map_err(|e| e.to_string())?;
        fs::write(history_file, json).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn delete_pure_plan(ac_path: String, plan_id: String) -> Result<(), String> {
    let plans_dir = Path::new(&ac_path).join("extension").join("config-ext").join("PurePlanner").join("Plans");
    let file_path = plans_dir.join(plan_id);
    if file_path.exists() {
        fs::remove_file(file_path).map_err(|e| format!("Erreur suppression: {}", e))?;
    }
    Ok(())
}

#[derive(Serialize, serde::Deserialize, Clone)]
struct ModInstallEntry {
    id: String,
    mod_name: String,
    mod_type: String, // "Car" ou "Track"
    path_installed: String, // Chemin relatif installé (ex: content/cars/mon_mod)
    date: String,
    #[serde(default)] // ⭐ Permet de lire les anciens JSON sans ce champ
    exists: bool,
}

// use std::io::{BufRead, BufReader}; // ⭐ Supprimé car inutilisé
use std::process::{Command, Stdio};

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[tauri::command]
async fn install_mod_from_archive(
    app_handle: tauri::AppHandle,
    ac_path: String, 
    documents_path: String, // ⭐ NOUVEAU
    archive_path: String,
    on_event: tauri::ipc::Channel<serde_json::Value>
) -> Result<String, String> {
    let ac_root = Path::new(&ac_path);
    
    // 1. Créer un dossier temporaire unique
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let temp_dir = std::env::temp_dir().join(format!("qarl_mod_{}", now));
    if temp_dir.exists() { let _ = std::fs::remove_dir_all(&temp_dir); }
    std::fs::create_dir_all(&temp_dir).map_err(|e| format!("Erreur création dossier temp: {}", e))?;

    // 2. Trouver l'exécutable 7za.exe dans les ressources de Tauri
    let seven_zip_path = app_handle.path().resolve("resources/7za.exe", tauri::path::BaseDirectory::Resource)
        .map_err(|e| format!("7za.exe introuvable dans les ressources: {}", e))?;

    let _ = on_event.send(serde_json::json!({ "type": "start", "total": 100.0 }));

    // 3. Lancer 7za.exe pour extraire dans le dossier temporaire
    let mut child = Command::new(seven_zip_path)
        .args(["x", &archive_path, &format!("-o{}", temp_dir.to_string_lossy()), "-y", "-bsp1"])
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Erreur lors du lancement de 7za.exe: {}", e))?;

        let mut stdout = child.stdout.take().ok_or("Impossible de lire la sortie de 7za.exe")?;
    let mut stderr = child.stderr.take(); // ⭐ On capture aussi les erreurs pour éviter les deadlocks
    let mut last_percent = 0.0;
    let mut current_str = String::new();
    let mut buffer = [0u8; 1024];

    use std::io::Read;

    // 4. Lire la sortie en temps réel (gestion des \r de 7za)
    loop {
        match stdout.read(&mut buffer) {
            Ok(0) => break, // Fin de l'extraction
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buffer[..n]);
                current_str.push_str(&chunk);
                
                // ⭐ On sépare sur \r ET \n pour attraper les mises à jour de 7za
                let parts: Vec<&str> = current_str.split(|c| c == '\n' || c == '\r').collect();
                
                // On traite toutes les lignes complètes
                for part in &parts[..parts.len().saturating_sub(1)] {
                    if part.contains('%') {
                        let p_parts: Vec<&str> = part.split('%').collect();
                        if !p_parts.is_empty() {
                            let num_str = p_parts[0].trim();
                            // On nettoie les caractères parasites éventuels
                            let clean_num: String = num_str.chars().filter(|c| c.is_numeric() || *c == '.').collect();
                            
                            if let Ok(p) = clean_num.parse::<f64>() {
                                if p > last_percent {
                                    last_percent = p;
                                    let file_name = if p_parts.len() > 1 {
                                        p_parts[1].split('-').nth(1).unwrap_or("Extraction...").trim()
                                    } else {
                                        "Extraction..."
                                    };
                                    
                                    let _ = on_event.send(serde_json::json!({
                                        "type": "progress",
                                        "percent": p,
                                        "current_file": file_name
                                    }));
                                }
                            }
                        }
                    }
                }
                // On garde le reste incomplet dans le buffer pour la prochaine lecture
                current_str = parts.last().unwrap_or(&"").to_string();
            }
            Err(_) => break,
        }
    }
    
    // ⭐ On vide stderr à la fin pour éviter que le processus ne bloque s'il y a eu beaucoup d'erreurs
    if let Some(mut err) = stderr.take() {
        let mut err_buf = Vec::new();
        let _ = err.read_to_end(&mut err_buf);
    }

    let status = child.wait().map_err(|e| format!("Erreur d'attente de 7za.exe: {}", e))?;
    if !status.success() {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err("L'extraction a échoué (archive corrompue ou mot de passe requis).".into());
    }

    // 5. Analyse intelligente du dossier temporaire
    let known_dirs = ["content", "extension", "apps", "system", "cfg", "launcher", "plugins", "sdk"];
    let mut standard_roots = Vec::new();
    let mut detected_cars = Vec::new();   // (Nom du dossier, Chemin source)
    let mut detected_tracks = Vec::new(); // (Nom du dossier, Chemin source)

    fn analyze_dir(dir: &Path, ac_root: &Path, known: &[&str], depth: u32, std_roots: &mut Vec<std::path::PathBuf>, cars: &mut Vec<(String, std::path::PathBuf)>, tracks: &mut Vec<(String, std::path::PathBuf)>) {
        if depth > 3 { return; } // On limite la recherche pour éviter de scanner tout le disque temporaire

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                    
                    // Cas A : C'est un dossier standard AC
                    if known.contains(&name.as_str()) {
                        if let Some(parent) = path.parent() {
                            std_roots.push(parent.to_path_buf());
                        }
                        continue;
                    }

                    // Cas B : C'est une voiture brute (cherche ui/ui_car.json)
                    let car_json = path.join("ui").join("ui_car.json");
                    if car_json.exists() {
                        cars.push((entry.file_name().to_string_lossy().to_string(), path.clone()));
                        continue;
                    }

                    // Cas C : C'est un circuit brut (cherche ui/ui_track.json)
                    let track_json = path.join("ui").join("ui_track.json");
                    if track_json.exists() {
                        tracks.push((entry.file_name().to_string_lossy().to_string(), path.clone()));
                        continue;
                    }

                    // Si c'est un dossier racine (ex: "MonMod"), on continue de chercher à l'intérieur
                    analyze_dir(&path, ac_root, known, depth + 1, std_roots, cars, tracks);
                }
            }
        }
    }

    analyze_dir(&temp_dir, ac_root, &known_dirs, 0, &mut standard_roots, &mut detected_cars, &mut detected_tracks);
    
    // Dédoublonnage et tri des racines standards
    standard_roots.sort();
    standard_roots.dedup();
    standard_roots.sort_by_key(|p| p.components().count());

    let mut total_copied = 0;

    // 6. Priorité 1 : Copie des structures standards (Apps, Extension, Content complet)
    if !standard_roots.is_empty() {
        let mod_root = &standard_roots[0];
        if let Ok(entries) = std::fs::read_dir(mod_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let is_known_dir = path.is_dir() && known_dirs.contains(&name.to_lowercase().as_str());
                let is_root_file = path.is_file() && !name.starts_with('.');
                
                if is_known_dir || is_root_file {
                    // ⭐ NOUVEAU : Si c'est le dossier content, on scanne les voitures et circuits pour l'historique
                    if name.eq_ignore_ascii_case("content") && path.is_dir() {
                        if let Ok(sub_entries) = std::fs::read_dir(&path) {
                            for sub in sub_entries.flatten() {
                                let sub_name = sub.file_name().to_string_lossy().to_string();
                                if sub_name.eq_ignore_ascii_case("cars") {
                                    if let Ok(car_entries) = std::fs::read_dir(sub.path()) {
                                        for car in car_entries.flatten() {
                                            detected_cars.push((car.file_name().to_string_lossy().to_string(), car.path()));
                                        }
                                    }
                                } else if sub_name.eq_ignore_ascii_case("tracks") {
                                    if let Ok(track_entries) = std::fs::read_dir(sub.path()) {
                                        for track in track_entries.flatten() {
                                            detected_tracks.push((track.file_name().to_string_lossy().to_string(), track.path()));
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let dest = ac_root.join(&name);
                    if path.is_dir() {
                        let _ = copy_dir_all(&path, &dest);
                    } else {
                        let _ = std::fs::copy(&path, &dest);
                    }
                    total_copied += 1;
                }
            }
        }
    } 
    
    // 7. Priorité 2 : Copie des voitures et circuits bruts (si trouvés)
    if !detected_cars.is_empty() {
        let cars_dir = ac_root.join("content").join("cars");
        let _ = std::fs::create_dir_all(&cars_dir);
        for (_name, path) in &detected_cars {
            let dest = cars_dir.join(path.file_name().unwrap());
            let _ = copy_dir_all(path, &dest);
            total_copied += 1;
        }
    }

    if !detected_tracks.is_empty() {
        let tracks_dir = ac_root.join("content").join("tracks");
        let _ = std::fs::create_dir_all(&tracks_dir);
        for (_name, path) in &detected_tracks {
            let dest = tracks_dir.join(path.file_name().unwrap());
            let _ = copy_dir_all(path, &dest);
            total_copied += 1;
        }
    }

    // 8. Nettoyage du dossier temporaire
    let _ = std::fs::remove_dir_all(&temp_dir);

    // 9. Vérification de sécurité : a-t-on vraiment copié quelque chose ?
    if total_copied == 0 {
        return Err("❌ Arborescence non reconnue. Aucun fichier voiture, circuit ou dossier standard (apps, extension) n'a été trouvé dans cette archive.".into());
    }

    // 10. Sauvegarde dans l'historique
    let cfg_dir = if !documents_path.is_empty() {
        resolve_cfg_dir(Path::new(&documents_path))
    } else {
        let docs_fallback = dirs::document_dir().ok_or("Dossier Documents introuvable")?;
        docs_fallback.join("Assetto Corsa").join("cfg")
    };
    let _ = std::fs::create_dir_all(&cfg_dir);
    let history_file = cfg_dir.join("QARL_mods_history.json");
    
    let mut history: Vec<ModInstallEntry> = if history_file.exists() {
        std::fs::read_to_string(&history_file).ok().and_then(|c| serde_json::from_str(&c).ok()).unwrap_or_default()
    } else { Vec::new() };

    let mod_name = Path::new(&archive_path).file_stem().unwrap_or_default().to_string_lossy().to_string();

    // Enregistrement des voitures dans l'historique
    for (car_name, _) in detected_cars {
        history.insert(0, ModInstallEntry {
            id: format!("{}_{}", now, car_name),
            mod_name: car_name.clone(),
            mod_type: "mods.type_car".into(),
            path_installed: format!("content/cars/{}", car_name),
            date: now.to_string(),
            exists: true,
        });
    }

    // Enregistrement des circuits dans l'historique
    for (track_name, _) in detected_tracks {
        history.insert(0, ModInstallEntry {
            id: format!("{}_{}", now, track_name),
            mod_name: track_name.clone(),
            mod_type: "mods.type_track".into(),
            path_installed: format!("content/tracks/{}", track_name),
            date: now.to_string(),
            exists: true,
        });
    }
    
    if history.len() > 100 { history.truncate(100); }
    let _ = std::fs::write(history_file, serde_json::to_string_pretty(&history).unwrap_or_default());

    Ok(format!("✅ Mod '{}' installé avec succès !", mod_name))
}

#[tauri::command]
fn get_mods_history(ac_path: String, documents_path: String) -> Result<Vec<ModInstallEntry>, String> {
    let cfg_dir = if !documents_path.is_empty() {
        resolve_cfg_dir(Path::new(&documents_path))
    } else {
        let docs_fallback = dirs::document_dir().ok_or("Dossier Documents introuvable")?;
        docs_fallback.join("Assetto Corsa").join("cfg")
    };
    let history_file = cfg_dir.join("QARL_mods_history.json");
    if !history_file.exists() { return Ok(Vec::new()); }
    let content = std::fs::read_to_string(history_file).map_err(|e| e.to_string())?;
    let mut history: Vec<ModInstallEntry> = serde_json::from_str(&content).unwrap_or_default();
    
    // ⭐ On met à jour le champ "exists" pour chaque mod
    for m in history.iter_mut() {
        let full_path = Path::new(&ac_path).join(&m.path_installed);
        m.exists = full_path.exists();
    }
    
    Ok(history)
}

#[tauri::command]
fn open_folder(ac_path: String, documents_path: String, folder_type: String, folder_id: String) -> Result<(), String> {
    let cfg_dir = resolve_cfg_dir(Path::new(&documents_path));
    let ac_docs_dir = if cfg_dir.ends_with("cfg") { cfg_dir.parent().unwrap_or(&cfg_dir) } else { &cfg_dir };

    let path = match folder_type.as_str() {
        "cars" => Path::new(&ac_path).join("content").join("cars").join(folder_id),
        "tracks" => Path::new(&ac_path).join("content").join("tracks").join(folder_id),
        "mod" => Path::new(&ac_path).join(folder_id),
        "cfg" => cfg_dir,
        "absolute" => Path::new(&folder_id).to_path_buf(),
        "screens" => ac_docs_dir.join("screens").join(folder_id),
        "replay" => ac_docs_dir.join("replay").join(folder_id),
        _ => return Err("Type inconnu".into()),
    };

    println!("🔍 RUST : Tentative d'ouverture du chemin : {:?}", path);

    if !path.exists() {
        println!("❌ RUST : Le dossier n'existe pas physiquement sur le disque.");
        return Err("Le dossier n'existe pas.".into());
    }

    #[cfg(target_os = "windows")]
    {
        // ⭐ On transforme le chemin mixte (C:\...\content/cars/x) en vrai chemin Windows (C:\...\content\cars\x)
        let clean_path = path.to_string_lossy().replace("/", "\\");
        std::process::Command::new("explorer").arg(&clean_path).spawn().map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&path).spawn().map_err(|e| e.to_string())?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(&path).spawn().map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Serialize, Clone)]
struct OnlineServer {
    name: String,
    ip: String, 
    port: u16,  
    track: String,
    players: u32,
    max_players: u32,
    status: String,
    country: Option<String>,
    join_url: String,
    cars: Vec<String>,
    description: Option<String>,
}

#[derive(Serialize, Clone)]
struct OnlineServersResponse {
    servers: Vec<OnlineServer>,
    has_next: bool,
}

#[tauri::command]
async fn get_online_servers(page: u32) -> Result<OnlineServersResponse, String> {
    use scraper::{Html, Selector};

    // On cible la page demandée de gs4u.net
    let url = format!("https://www.gs4u.net/en/assettocorsa/page-{}", page);
    let resp = reqwest::get(url)
        .await
        .map_err(|e| format!("Erreur réseau: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Erreur HTTP: {}", resp.status()));
    }

    let body = resp.text().await.map_err(|e| format!("Erreur lecture HTML: {}", e))?;
    let document = Html::parse_document(&body);

    // Sélecteurs CSS basés sur ton ancien main.js
    let row_selector = Selector::parse("tr[data-server-id]").unwrap();
    let name_selector = Selector::parse("td.servername a").unwrap();
    let players_selector = Selector::parse("td.players").unwrap();
    let ipport_selector = Selector::parse("td.ipport a").unwrap();
    let map_selector = Selector::parse("td.mapname a").unwrap();
    let country_selector = Selector::parse("td.contryname span").unwrap();

    let mut servers = Vec::new();

    for row in document.select(&row_selector) {
        // Nom du serveur
        let raw_name = row.select(&name_selector)
            .next()
            .and_then(|el| el.value().attr("title"))
            .unwrap_or("Serveur inconnu")
            .to_string();
        
        // ⭐ Nettoyage du nom (on retire le texte parasite de gs4u)
        let name = raw_name.replace("Go to page with detailed information about the server: ", "");

        // Joueurs (ex: "16&nbsp;/&nbsp;32" -> on nettoie)
        let players_text = row.select(&players_selector)
            .next()
            .map(|el| el.text().collect::<String>().replace("&nbsp;", " "))
            .unwrap_or_default();
        
        let mut players = 0;
        let mut max_players = 0;
        if let Some((p, m)) = players_text.split_once('/') {
            players = p.trim().parse::<u32>().unwrap_or(0);
            max_players = m.trim().parse::<u32>().unwrap_or(0);
        }

        // IP:Port (ex: "46.4.115.244:8390")
        let ipport_text = row.select(&ipport_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        let (ip, port) = if let Some((i, p)) = ipport_text.split_once(':') {
            (i.to_string(), p.parse::<u16>().unwrap_or(0))
        } else {
            ("0.0.0.0".to_string(), 0)
        };

        // Circuit (Map)
        let raw_track = row.select(&map_selector)
            .next()
            .and_then(|el| el.value().attr("title"))
            .unwrap_or("Circuit inconnu")
            .to_string();
            
        // ⭐ Nettoyage du texte parasite de GS4u
        let mut clean_track = raw_track.replace("Search server with a map: ", "");
        
        // ⭐ Nettoyage des chemins complexes envoyés par certains serveurs (ex: "csp/.../ks_nordschleife")
        if clean_track.contains('/') {
            if let Some(last_part) = clean_track.split('/').last() {
                clean_track = last_part.to_string();
            }
        }
        let track = clean_track;

        // Pays (on cherche la classe 'flagicon16 xx' comme 'fr', 'de', etc.)
        let country = row.select(&country_selector)
            .next()
            .and_then(|el| {
                // On cherche la classe qui fait exactement 2 lettres (ex: "fr", "de", "us")
                for class in el.value().classes() {
                    if class.len() == 2 && class != "16" && class.chars().all(|c| c.is_ascii_alphabetic()) {
                        return Some(class.to_uppercase());
                    }
                }
                None
            });

        if port > 0 {
            servers.push(OnlineServer {
                name,
                ip,
                port,
                track,
                players,
                max_players,
                status: "En ligne".to_string(),
                country,
                join_url: String::new(), // On n'en a pas besoin
                cars: Vec::new(),        
                description: None,
            });
        }
    }

    // ⭐ Détection de la page suivante
    let next_page_str = format!("page-{}", page + 1);
    let has_next = body.contains(&next_page_str);

    Ok(OnlineServersResponse { servers, has_next })
}

#[tauri::command]
async fn get_server_details(ip: String, http_port: u16) -> Result<serde_json::Value, String> {
    // On prépare un client avec un timeout plus court (2s) et un User-Agent de navigateur
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(2)) 
        .build()
        .unwrap_or_default();

    let candidates = vec![http_port, http_port.wrapping_add(1), http_port.wrapping_sub(1)];

    // ⭐ Fonction interne pour tester un port spécifique
    async fn test_port(client: reqwest::Client, ip: String, port: u16) -> Result<serde_json::Value, String> {
        if port == 0 { return Err("Port invalide".into()); }

        // 1. On tente /api/details EN PRIORITÉ
        let url_details = format!("http://{}:{}/api/details", ip, port);
        if let Ok(resp) = client.get(&url_details).send().await {
            if resp.status().is_success() {
                if let Ok(text) = resp.text().await {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        return Ok(json);
                    }
                }
            }
        }

        // 2. On tente /INFO en fallback
        let url_info = format!("http://{}:{}/INFO", ip, port);
        if let Ok(resp) = client.get(&url_info).send().await {
            if resp.status().is_success() {
                if let Ok(text) = resp.text().await {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        return Ok(json);
                    }
                }
            }
        }

        Err("Introuvable".into())
    }

    // ⭐ On lance les 3 tests EN PARALLÈLE
    let (res1, res2, res3) = tokio::join!(
        test_port(client.clone(), ip.clone(), candidates[0]),
        test_port(client.clone(), ip.clone(), candidates[1]),
        test_port(client.clone(), ip.clone(), candidates[2])
    );

    // ⭐ On retourne le premier résultat qui a réussi
    res1.or(res2).or(res3).or_else(|_| Err("Serveur injoignable. Vérifie qu'il est en ligne.".to_string()))
}

#[tauri::command]
fn join_server(
    app_handle: tauri::AppHandle,
    ac_path: String,
    documents_path: String, // ⭐ NOUVEAU : respecte le documentsPath configuré
    ip: String,
    port: u16,
    http_port: u16,
    car_id: String,
    skin_id: String,
    online_name: String,
    steam_id: String,
    server_name: String,
    track: String,
    track_config: String,
    password: String,
    features: Vec<String>,
) -> Result<(), String> {
    // ⭐ Résolution du dossier cfg (respecte documents_path, fallback sur dirs si vide)
    let cfg_dir = if !documents_path.is_empty() {
        resolve_cfg_dir(Path::new(&documents_path))
    } else {
        let docs_fallback = dirs::document_dir().ok_or("Impossible de trouver le dossier Documents")?;
        docs_fallback.join("Assetto Corsa").join("cfg")
    };
    
    if !cfg_dir.exists() {
        fs::create_dir_all(&cfg_dir).map_err(|e| format!("Erreur création dossier cfg: {}", e))?;
    }

    let race_ini_path = cfg_dir.join("race.ini");

    // On lit le race.ini existant pour ne pas tout écraser
    let mut content = if race_ini_path.exists() {
        fs::read_to_string(&race_ini_path).unwrap_or_default()
    } else {
        String::new()
    };

    // ⭐ NETTOYAGE : On supprime les sections [CAR_X] (IA) pour le multijoueur
    let mut cleaned_content = String::new();
    let mut in_car_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[CAR_") && trimmed != "[CAR_0]" {
            in_car_section = true;
            continue;
        } else if trimmed.starts_with('[') {
            in_car_section = false;
        }
        
        if !in_car_section {
            cleaned_content.push_str(line);
            cleaned_content.push_str("\r\n");
        }
    }
    content = cleaned_content;

    // ⭐ On prépare la section [REMOTE] avec le mot de passe et les features.
    // ⚠️ CRLF (\r\n) est OBLIGATOIRE pour qu'AC lise correctement le fichier.
    let mut remote_section = format!(
        "[REMOTE]\r\nACTIVE=1\r\nSERVER_IP={}\r\nSERVER_PORT={}\r\nSERVER_NAME={}\r\nSERVER_HTTP_PORT={}\r\nREQUESTED_CAR={}\r\nREQUESTED_SKIN={}\r\nGUID={}\r\nNAME={}\r\n__CM_EXTENDED=1\r\n",
        ip, port, server_name, http_port, car_id, skin_id, steam_id, online_name
    );

    if !password.is_empty() {
        remote_section.push_str(&format!("PASSWORD={}\r\n", password));
    }
    if !features.is_empty() {
        remote_section.push_str(&format!("__FEATURES={}\r\n", features.join(",")));
    }
    remote_section.push_str("\r\n");

    // Remplacer ou ajouter [REMOTE]
    if content.contains("[REMOTE]") {
        let parts: Vec<&str> = content.split("[REMOTE]").collect();
        let rest = parts[1];
        let next_section = rest.find('[').unwrap_or(rest.len());
        content = format!("{}{}{}", parts[0], remote_section, &rest[next_section..]);
    } else {
        content.push_str(&remote_section);
    }

    // On s'assure que le circuit et la voiture sont bien dans [RACE]
    let race_section = format!(
        "[RACE]\r\nTRACK={}\r\nCONFIG_TRACK={}\r\nMODEL={}\r\nSKIN={}\r\nCARS=1\r\n\r\n",
        track, track_config, car_id, skin_id
    );

    if content.contains("[RACE]") {
        let parts: Vec<&str> = content.split("[RACE]").collect();
        let rest = parts[1];
        let next_section = rest.find('[').unwrap_or(rest.len());
        content = format!("{}{}{}", parts[0], race_section, &rest[next_section..]);
    } else {
        content = format!("{}{}", race_section, content);
    }

    fs::write(&race_ini_path, content).map_err(|e| format!("Erreur écriture race.ini: {}", e))?;

    // Lancement du jeu avec les variables Steam (OBLIGATOIRE pour le multi)
    let ac_exe = Path::new(&ac_path).join("acs.exe");
    if !ac_exe.exists() {
        return Err(format!("acs.exe introuvable dans le dossier : {}", ac_path));
    }

    let mut cmd = std::process::Command::new(ac_exe);
    cmd.current_dir(&ac_path);
    cmd.env("SteamAppId", "244210");
    cmd.env("SteamGameId", "244210");
    
    // ⭐ On attend la fermeture du jeu en arrière-plan (pour le Focus Auto)
    let mut child = cmd.spawn().map_err(|e| format!("Erreur lors du lancement du jeu: {}", e))?;
    let handle = app_handle.clone();
    std::thread::spawn(move || {
        let _ = child.wait();
        let _ = handle.emit("ac-closed", ());
    });

    Ok(())
}

// ⭐ Commandes pour les contrôles (Volant / Manette / Clavier)
#[derive(Serialize, Clone)]
struct ProfileInfo {
    name: String,
    path: String,
    profile_type: String, // "savedsetup" | "controller"
}

#[tauri::command]
fn get_controller_profiles(documents_path: String) -> Result<Vec<ProfileInfo>, String> {
    let base_path = Path::new(&documents_path);
    let mut cfg_dir = base_path.join("cfg");
    if !cfg_dir.exists() {
        cfg_dir = base_path.join("Assetto Corsa").join("cfg");
    }
    let mut profiles = Vec::new();
    let controllers_base = cfg_dir.join("controllers");

    // 1. Dossier savedsetups (profils utilisateur)
    let setups_dir = controllers_base.join("savedsetups");
    if setups_dir.exists() {
        if let Ok(entries) = fs::read_dir(setups_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "ini") {
                    profiles.push(ProfileInfo {
                        name: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                        path: path.to_string_lossy().to_string(),
                        profile_type: "savedsetup".to_string(),
                    });
                }
            }
        }
    }

    // 2. Dossier presets (profils constructeurs comme Logitech, Fanatec, etc.)
    let presets_dir = controllers_base.join("presets");
    if presets_dir.exists() {
        if let Ok(entries) = fs::read_dir(presets_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "ini") {
                    profiles.push(ProfileInfo {
                        name: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                        path: path.to_string_lossy().to_string(),
                        profile_type: "controller".to_string(),
                    });
                }
            }
        }
    }

    // 3. Optionnel : Ancien format (parfois les setups sont directement dans cfg/savedsetups)
    let legacy_setups_dir = cfg_dir.join("savedsetups");
    if legacy_setups_dir.exists() {
        if let Ok(entries) = fs::read_dir(legacy_setups_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "ini") {
                    profiles.push(ProfileInfo {
                        name: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                        path: path.to_string_lossy().to_string(),
                        profile_type: "savedsetup".to_string(),
                    });
                }
            }
        }
    }

    Ok(profiles)
}

#[tauri::command]
fn read_ini_file(documents_path: String, filename: String) -> Result<String, String> {
    let base_path = Path::new(&documents_path);
    
    // On cherche dans plusieurs chemins possibles pour être sûr
    let paths_to_try = vec![
        base_path.join(&filename),
        base_path.join("cfg").join(&filename),
        base_path.join("Assetto Corsa").join("cfg").join(&filename),
    ];

    for file_path in paths_to_try {
        if file_path.exists() {
            return fs::read_to_string(&file_path).map_err(|e| format!("Erreur de lecture: {}", e));
        }
    }

    Err(format!("Fichier introuvable : {}", filename))
}

#[tauri::command]
fn list_ppfilters(ac_path: String) -> Result<Vec<String>, String> {
    let ppfilters_dir = Path::new(&ac_path).join("system").join("cfg").join("ppfilters");
    
    if !ppfilters_dir.exists() {
        return Err("Dossier ppfilters introuvable".to_string());
    }

    let mut filters = Vec::new();
    if let Ok(entries) = fs::read_dir(ppfilters_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "ini") {
                if let Some(stem) = path.file_stem() {
                    filters.push(stem.to_string_lossy().to_string());
                }
            }
        }
    }
    filters.sort();
    Ok(filters)
}

#[tauri::command]
fn write_ini_file(documents_path: String, filename: String, content: String) -> Result<(), String> {
    let base_path = Path::new(&documents_path);
    let mut cfg_dir = base_path.join("cfg");
    
    if !cfg_dir.exists() {
        cfg_dir = base_path.join("Assetto Corsa").join("cfg");
    }
    
    // On crée le dossier s'il n'existe pas
    if !cfg_dir.exists() {
        fs::create_dir_all(&cfg_dir).map_err(|e| format!("Erreur création dossier: {}", e))?;
    }
    
    let file_path = cfg_dir.join(&filename);
    
    // On fait un backup au cas où
    if file_path.exists() {
        let backup_path = file_path.with_extension("ini.bak");
        let _ = fs::copy(&file_path, backup_path);
    }

    fs::write(&file_path, content).map_err(|e| format!("Erreur d'écriture: {}", e))?;
    
    Ok(())
}

#[tauri::command]
fn read_controls_ini(documents_path: String, device_type: String) -> Result<String, String> {
    let filename = match device_type.as_str() {
        "x360" => "x360.ini",
        "keyboard" => "keyboard.ini",
        _ => "controls.ini",
    };
    
    let base_path = Path::new(&documents_path);
    
    // ⭐ On teste plusieurs chemins possibles pour être robuste
    let paths_to_try = vec![
        base_path.join(filename), // Si tu as choisi le dossier .../cfg
        base_path.join("cfg").join(filename), // Si tu as choisi le dossier .../Assetto Corsa
        base_path.join("Assetto Corsa").join("cfg").join(filename), // Si tu as choisi .../Documents
    ];

    for file_path in paths_to_try {
        if file_path.exists() {
            return fs::read_to_string(&file_path).map_err(|e| format!("Erreur de lecture: {}", e));
        }
    }

    // Si on ne trouve vraiment rien
    Ok(String::new())
}

#[tauri::command]
fn read_specific_ini(file_path: String) -> Result<String, String> {
    let path = Path::new(&file_path);
    if path.exists() {
        fs::read_to_string(path).map_err(|e| format!("Erreur de lecture: {}", e))
    } else {
        Err("Fichier introuvable".into())
    }
}

#[tauri::command]
fn write_controls_ini(documents_path: String, device_type: String, content: String, save_as_name: Option<String>, apply_to_game: bool) -> Result<(), String> {
    let filename = match device_type.as_str() {
        "x360" => "x360.ini",
        "keyboard" => "keyboard.ini",
        _ => "controls.ini",
    };
    
    let base_path = Path::new(&documents_path);
    let mut cfg_dir = base_path.join("cfg");
    if !cfg_dir.exists() {
        cfg_dir = base_path.join("Assetto Corsa").join("cfg");
    }
    if !cfg_dir.exists() {
        fs::create_dir_all(&cfg_dir).map_err(|e| format!("Erreur création dossier: {}", e))?;
    }
    
    // On sauvegarde dans savedsetups si un nom est fourni
    if let Some(name) = save_as_name {
        if !name.is_empty() {
            // ⭐ CORRECTION : Le bon chemin est controllers/savedsetups
            let setups_dir = cfg_dir.join("controllers").join("savedsetups");
            if !setups_dir.exists() {
                fs::create_dir_all(&setups_dir).map_err(|e| e.to_string())?;
            }
            let file_path = setups_dir.join(format!("{}.ini", name));
            fs::write(&file_path, &content).map_err(|e| format!("Erreur d'écriture du profil: {}", e))?;
        }
    }

    // On écrit dans le fichier actif UNIQUEMENT si apply_to_game est vrai
    if apply_to_game {
        let file_path = cfg_dir.join(filename);
        fs::write(&file_path, content).map_err(|e| format!("Erreur d'écriture: {}", e))?;
    }
    
    Ok(())
}

#[derive(Default)]
struct WatcherState(Mutex<Option<RecommendedWatcher>>);

#[tauri::command]
fn start_content_watcher(ac_path: String, app_handle: tauri::AppHandle, state: tauri::State<'_, WatcherState>) -> Result<(), String> {
    let mut state_lock = state.0.lock().unwrap();
    if state_lock.is_some() { return Ok(()); } // Déjà lancé

    let cars_path = Path::new(&ac_path).join("content").join("cars");
    let tracks_path = Path::new(&ac_path).join("content").join("tracks");

    let handle = app_handle.clone();
    let mut watcher = notify::recommended_watcher(move |res| {
        if let Ok(_) = res {
            // Émettre un événement au frontend
            let _ = handle.emit("content-changed", ());
        }
    }).map_err(|e| e.to_string())?;

    if cars_path.exists() {
        let _ = watcher.watch(&cars_path, RecursiveMode::Recursive);
    }
    if tracks_path.exists() {
        let _ = watcher.watch(&tracks_path, RecursiveMode::Recursive);
    }

    *state_lock = Some(watcher);
    Ok(())
}

// ⭐ Structure pour lire les fichiers de mapping de Content Manager
#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct ControllerMapping {
    name: String,
    axis: Option<serde_json::Value>,
    buttons: Option<serde_json::Value>,
}

#[tauri::command]
fn get_controller_mapping(app_handle: tauri::AppHandle, vendor_id: String, product_id: String) -> Result<Option<ControllerMapping>, String> {
    // ⭐ On teste les deux formats possibles : ProductVendor et VendorProduct
    let file_name_pv = format!("{}{}-0000-0000-0000-504944564944.json", product_id.to_uppercase(), vendor_id.to_uppercase());
    let file_name_vp = format!("{}{}-0000-0000-0000-504944564944.json", vendor_id.to_uppercase(), product_id.to_uppercase());
    
    let mut resource_path = app_handle.path().resolve(&format!("resources/controllers/{}", file_name_pv), tauri::path::BaseDirectory::Resource)
        .map_err(|e| e.to_string())?;

    if !resource_path.exists() {
        resource_path = app_handle.path().resolve(&format!("resources/controllers/{}", file_name_vp), tauri::path::BaseDirectory::Resource)
            .map_err(|e| e.to_string())?;
    }

    println!("🔍 RUST: Recherche du fichier de mapping : {:?}", resource_path);

    if !resource_path.exists() {
        println!("❌ RUST: Fichier de mapping NON TROUVÉ à cet emplacement.");
        return Ok(None);
    }

    println!("✅ RUST: Fichier de mapping TROUVÉ !");

    let content = std::fs::read_to_string(resource_path).map_err(|e| e.to_string())?;
    
    // CM met parfois des commentaires "// ..." au début du JSON, on les nettoie
    let clean_json = content.lines().filter(|l| !l.trim().starts_with("//")).collect::<Vec<_>>().join("\n");

    let mapping: ControllerMapping = serde_json::from_str(&clean_json).map_err(|e| format!("Erreur JSON: {}", e))?;
    
    Ok(Some(mapping))
}

// ⭐ Structure pour renvoyer les infos des périphériques DirectInput
#[derive(Serialize, Clone)]
struct DInputDeviceInfo {
    name: String,
    pguid: String,
    iguid: String,
    dinput_index: u32, // ⭐ L'index de l'ordre d'énumération de Windows
}

// ⭐ Entrée de la section [CONTROLLERS] de controls.ini
#[derive(Serialize, Clone)]
struct ControllerEntry {
    con_index: u32,
    name: String,
    iguid: String,
    pguid: String,
}

#[tauri::command]
fn get_directinput_devices() -> Result<Vec<DInputDeviceInfo>, String> {
    let mut devices = Vec::new();

    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::Devices::HumanInterfaceDevice::*;
        use windows::Win32::Foundation::*;
        use windows::Win32::System::Com::*;
        use windows::Win32::System::LibraryLoader::GetModuleHandleW;
        use windows::core::Interface;

        // Initialiser COM
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let mut dinput_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        let hinst = GetModuleHandleW(None).unwrap();

        // ⭐ CORRECTION E0283 : On passe hinst directement, sans .into()
        if DirectInput8Create(
            hinst,
            0x0800, // DIRECTINPUT_VERSION
            &IDirectInput8W::IID,
            &mut dinput_ptr,
            None,
        ).is_ok() {
            if !dinput_ptr.is_null() {
                let di = IDirectInput8W::from_raw(dinput_ptr);

                // Callback pour énumérer les périphériques
                unsafe extern "system" fn enum_callback(
                    lpddi: *mut DIDEVICEINSTANCEW, 
                    pv_ref: *mut std::ffi::c_void,
                ) -> BOOL {
                    // ⭐ CORRECTION E0133 : unsafe explicite pour les déréférencements (Rust 2024)
                    let devices = unsafe { &mut *(pv_ref as *mut Vec<DInputDeviceInfo>) };
                    let inst = unsafe { &*lpddi };

                    // Extraire le nom et nettoyer les caractères nuls de Windows
                    let raw_name = String::from_utf16_lossy(&inst.tszInstanceName);
                    let name = raw_name.replace('\u{0}', "").trim().to_string();

                    let pguid = format!(
                        "{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
                        inst.guidProduct.data1,
                        inst.guidProduct.data2,
                        inst.guidProduct.data3,
                        inst.guidProduct.data4[0], inst.guidProduct.data4[1],
                        inst.guidProduct.data4[2], inst.guidProduct.data4[3],
                        inst.guidProduct.data4[4], inst.guidProduct.data4[5],
                        inst.guidProduct.data4[6], inst.guidProduct.data4[7]
                    );

                    let iguid = format!(
                        "{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
                        inst.guidInstance.data1,
                        inst.guidInstance.data2,
                        inst.guidInstance.data3,
                        inst.guidInstance.data4[0], inst.guidInstance.data4[1],
                        inst.guidInstance.data4[2], inst.guidInstance.data4[3],
                        inst.guidInstance.data4[4], inst.guidInstance.data4[5],
                        inst.guidInstance.data4[6], inst.guidInstance.data4[7]
                    );

                    // ⭐ L'index dinput = nombre de devices déjà poussés
                    let dinput_index = devices.len() as u32;
                    devices.push(DInputDeviceInfo { name, pguid, iguid, dinput_index });
                    TRUE
                }

                let mut dev_list: Vec<DInputDeviceInfo> = Vec::new();
                let _ = di.EnumDevices(
                    DI8DEVCLASS_GAMECTRL,
                    Some(enum_callback),
                    &mut dev_list as *mut _ as *mut std::ffi::c_void,
                    DIEDFL_ATTACHEDONLY,
                );
                devices = dev_list;
            }
        }
        let _ = CoUninitialize();
    }

    Ok(devices)
}

// ⭐ Résout le chemin exact de controls.ini selon les 3 conventions possibles
fn resolve_controls_ini_path(documents_path: &str) -> Option<std::path::PathBuf> {
    let base = Path::new(documents_path);
    let paths_to_try = vec![
        base.join("controls.ini"),
        base.join("cfg").join("controls.ini"),
        base.join("Assetto Corsa").join("cfg").join("controls.ini"),
    ];
    for p in paths_to_try {
        if p.exists() { return Some(p); }
    }
    None
}

// ⭐ Parse uniquement la section [CONTROLLERS] de controls.ini
#[tauri::command]
fn parse_controllers_section(documents_path: String) -> Result<Vec<ControllerEntry>, String> {
    let path = resolve_controls_ini_path(&documents_path)
        .ok_or("controls.ini introuvable")?;
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;

    // On accumule par index (HashMap), puis on trie à la fin
    let mut map: std::collections::HashMap<u32, (String, String, String)> =
        std::collections::HashMap::new();
    let mut in_section = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = trimmed.eq_ignore_ascii_case("[CONTROLLERS]");
            continue;
        }
        if !in_section { continue; }
        if trimmed.is_empty() || trimmed.starts_with(';') { continue; }

        if let Some((key, val)) = trimmed.split_once('=') {
            let key = key.trim();
            let val = val.trim().to_string();

            if let Some(idx_str) = key.strip_prefix("CON") {
                if let Ok(idx) = idx_str.parse::<u32>() {
                    map.entry(idx).or_insert((String::new(), String::new(), String::new())).0 = val;
                }
            } else if let Some(idx_str) = key.strip_prefix("__IGUID") {
                if let Ok(idx) = idx_str.parse::<u32>() {
                    map.entry(idx).or_insert((String::new(), String::new(), String::new())).1 = val;
                }
            } else if let Some(idx_str) = key.strip_prefix("PGUID") {
                if let Ok(idx) = idx_str.parse::<u32>() {
                    map.entry(idx).or_insert((String::new(), String::new(), String::new())).2 = val;
                }
            }
        }
    }

    let mut out: Vec<ControllerEntry> = map.into_iter()
        .map(|(idx, (name, iguid, pguid))| ControllerEntry {
            con_index: idx, name, iguid, pguid,
        })
        .collect();
    out.sort_by_key(|e| e.con_index);
    Ok(out)
}

// ⭐ Garantit qu'un périphérique est présent dans [CONTROLLERS].
// Retourne son CONn (existant ou nouvellement créé).
#[tauri::command]
fn ensure_controller_registered(
    documents_path: String,
    pguid: String,
    iguid: String,
    name: String,
) -> Result<u32, String> {
    let path = resolve_controls_ini_path(&documents_path)
        .ok_or("controls.ini introuvable")?;

    // 1. Match par PGUID + IGUID (les 2 sont nécessaires car doublon PGUID possible)
    let existing = parse_controllers_section(documents_path.clone())?;
    for e in &existing {
        if e.pguid.eq_ignore_ascii_case(&pguid) && e.iguid.eq_ignore_ascii_case(&iguid) {
            return Ok(e.con_index);
        }
    }

    // 2. Aucun match → on ajoute à la fin de [CONTROLLERS]
    let next_idx = existing.iter().map(|e| e.con_index).max().map(|m| m + 1).unwrap_or(0);

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let eol = if content.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // Trouve la ligne de fin de [CONTROLLERS] (début de la section suivante, ou fin fichier)
    let mut in_section = false;
    let mut insert_at: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if in_section {
                insert_at = Some(i);   // On insère AVANT la section suivante
                break;
            }
            if trimmed.eq_ignore_ascii_case("[CONTROLLERS]") {
                in_section = true;
            }
        }
    }
    if insert_at.is_none() {
        if in_section {
            insert_at = Some(lines.len());  // [CONTROLLERS] est la dernière section
        } else {
            return Err("[CONTROLLERS] introuvable dans controls.ini".into());
        }
    }
    let insert_idx = insert_at.unwrap();

    // Backup avant écriture
    let backup = path.with_extension("ini.bak");
    let _ = fs::copy(&path, backup);

    // Insère les 3 lignes dans l'ordre CON / __IGUID / PGUID
    let new_lines = vec![
        format!("CON{}={}", next_idx, name),
        format!("__IGUID{}={}", next_idx, iguid),
        format!("PGUID{}={}", next_idx, pguid),
    ];
    for (offset, line) in new_lines.into_iter().enumerate() {
        lines.insert(insert_idx + offset, line);
    }

    let new_content = lines.join(eol) + eol;
    fs::write(&path, new_content).map_err(|e| format!("Erreur d'écriture: {}", e))?;

    Ok(next_idx)
}

// ⭐ Les fichiers data_* sont ignorés pour l'interface de configuration
const CSP_IGNORED_PREFIXES: &[&str] = &["data_"];

// ⭐ Récupère la liste des templates de manière synchrone pour les fonctions internes
fn get_csp_templates_list(ac_path: &str) -> Vec<String> {
    let mut templates = Vec::new();
    if ac_path.is_empty() { return templates; }
    let config_dir = Path::new(ac_path).join("extension").join("config");
    if !config_dir.exists() { return templates; }

    if let Ok(entries) = fs::read_dir(config_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "ini") {
                let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                if !CSP_IGNORED_PREFIXES.iter().any(|p| filename.starts_with(p)) && filename != "data_manifest.ini" {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if content.contains("[BASIC]") || content.contains("[ℹ]") {
                            templates.push(filename);
                        }
                    }
                }
            }
        }
    }
    templates
}

// ⭐ NOUVELLES STRUCTURES POUR LE PARSER ROBUSTE

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Choice {
    Separator,
    Value { value: String, label: String },
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ConditionExpr {
    Eq { key: String, value: String },
    Truthy { key: String },
    Not { expr: Box<ConditionExpr> },
    And { left: Box<ConditionExpr>, right: Box<ConditionExpr> },
    Or { left: Box<ConditionExpr>, right: Box<ConditionExpr> },
}

#[derive(Serialize, Clone)]
struct CspOption {
    key: String,
    value: String,
    label: String,
    option_type: String,
    min: Option<f32>,
    max: Option<f32>,
    step: Option<f32>,
    choices: Vec<Choice>,
    condition: Option<ConditionExpr>,
    is_hidden: bool,
    is_percentage: bool,
    file_filter: Option<String>, // ⭐ NOUVEAU
}

#[derive(Serialize, Clone)]
struct CspSection {
    name: String,
    is_enabled: bool,
    has_enabled_toggle: bool, // ⭐ Nouveau : y a-t-il un vrai switch ?
    options: Vec<CspOption>,
}

#[derive(Serialize, Clone)]
struct CspModuleData {
    filename: String,
    fullname: String,
    description: String,
    sections: Vec<CspSection>,
}

#[derive(Serialize, Clone)]
struct CspModuleInfo {
    filename: String,
    default_enabled: bool,
}

// ⭐ NOUVELLE FONCTION : Scan dynamique des templates
#[tauri::command]
fn list_csp_templates(ac_path: String) -> Result<Vec<CspModuleInfo>, String> {
    let config_dir = Path::new(&ac_path).join("extension").join("config");
    if !config_dir.exists() {
        return Err("Dossier extension/config introuvable".into());
    }

    let mut templates = Vec::new();
    if let Ok(entries) = fs::read_dir(config_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "ini") {
                let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                // On ignore les fichiers data_* et le manifest
                if !CSP_IGNORED_PREFIXES.iter().any(|p| filename.starts_with(p)) && filename != "data_manifest.ini" {
                    // On vérifie si le fichier contient une section [BASIC] ou [ℹ]
                    if let Ok(content) = fs::read_to_string(&path) {
                        if content.contains("[BASIC]") || content.contains("[ℹ]") {
                            // ⭐ On analyse le template pour trouver la valeur par défaut de ENABLED
                            let mut default_enabled = true;
                            let mut in_basic = false;
                            for line in content.lines() {
                                let trimmed = line.trim();
                                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                                    let sec_name = trimmed.trim_matches(|c| c == '[' || c == ']');
                                    in_basic = sec_name.eq_ignore_ascii_case("BASIC");
                                } else if in_basic && trimmed.to_uppercase().starts_with("ENABLED=") {
                                    if let Some(val) = trimmed.split('=').nth(1) {
                                        let clean_val = val.split(';').next().unwrap_or("").trim();
                                        if clean_val == "0" {
                                            default_enabled = false;
                                        }
                                    }
                                }
                            }
                            
                            templates.push(CspModuleInfo {
                                filename,
                                default_enabled,
                            });
                        }
                    }
                }
            }
        }
    }
    templates.sort_by(|a, b| a.filename.cmp(&b.filename));
    Ok(templates)
}

// ⭐ Recherche intelligente du dossier extension, peu importe la racine choisie
fn resolve_csp_dir(documents_path: &str) -> std::path::PathBuf {
    let base = std::path::Path::new(documents_path);
    
    if base.join("extension").exists() { return base.join("extension"); }
    if base.join("cfg").join("extension").exists() { return base.join("cfg").join("extension"); }
    
    // ⭐ Sécurité : Si le chemin se termine déjà par "Assetto Corsa", on ajoute JUSTE cfg
    if base.file_name().unwrap_or_default().to_string_lossy().eq_ignore_ascii_case("Assetto Corsa") {
        return base.join("cfg").join("extension");
    }
    
    let fallback = base.join("Assetto Corsa").join("cfg").join("extension");
    if fallback.exists() { return fallback; }
    
    fallback
}

fn get_csp_template_path(ac_path: &str) -> std::path::PathBuf {
    std::path::Path::new(ac_path)
        .join("extension")
        .join("config")
}

// ⭐ Copie le fichier template vers le dossier user s'il n'existe pas
fn ensure_user_csp_file(documents_path: &str, ac_path: &str, filename: &str) -> Result<std::path::PathBuf, String> {
    let user_dir = resolve_csp_dir(documents_path);
    let user_path = user_dir.join(filename);
    
    // Si le fichier existe déjà, on le retourne
    if user_path.exists() {
        return Ok(user_path);
    }
    
    // Si le template existe, on crée un fichier utilisateur VIDE (pour le diff)
    if !ac_path.is_empty() && std::path::Path::new(ac_path).exists() {
        let tpl_path = get_csp_template_path(ac_path).join(filename);
        if tpl_path.exists() {
            if !user_dir.exists() {
                let _ = fs::create_dir_all(&user_dir);
            }
            // On crée un fichier vide, pas une copie du template !
            fs::write(&user_path, "").map_err(|e| format!("Erreur création fichier user : {}", e))?;
            return Ok(user_path);
        }
    }
    
    Err(format!("Template introuvable : {}", filename))
}

// (Fonction list_csp_modules supprimée, remplacée par list_csp_templates)

// (Structure CspModuleContent supprimée car inutilisée)

// ⭐ HELPERS POUR LE PARSING DES HINTS

// Découpe une ligne en (Key, Value, Vec<Hints>)
fn parse_ini_line(line: &str) -> Option<(String, String, Vec<String>)> {
    let (main_part, rest) = match line.split_once(';') {
        Some((m, r)) => (m, Some(r)),
        None => (line, None),
    };
    
    let (key, val) = main_part.split_once('=')?;
    let key = key.trim().to_string();
    if key.is_empty() { return None; }
    
    let val = val.trim().to_string();
    let mut hints = Vec::new();
    
    if let Some(r) = rest {
        // On remet les ";" pour bien séparer les hints (ex: ";; from 0 to 100")
        let full_rest = format!(";{}", r);
        for h in full_rest.split(';').skip(1) {
            let clean = h.trim();
            if !clean.is_empty() {
                hints.push(clean.to_string());
            }
        }
    }
    
    Some((key, val, hints))
}

// ⭐ Petit helper pour extraire le PREMIER nombre valide d'une chaîne
// Ex: "5, round to 1" -> 5.0 | "4 car(s)" -> 4.0
fn extract_first_number(s: &str) -> Option<f32> {
    let mut num_str = String::new();
    let mut has_digit = false;
    for c in s.chars() {
        if c.is_numeric() || c == '.' || (c == '-' && !has_digit) {
            num_str.push(c);
            if c.is_numeric() { has_digit = true; }
        } else if has_digit {
            break; // On s'arrête dès qu'on a fini le premier nombre
        }
    }
    num_str.parse::<f32>().ok()
}

// Extrait un range (ex: "from 0 m/s to 100" -> (0.0, 100.0))
fn extract_range(hint: &str) -> Option<(f32, f32)> {
    let h = hint.to_lowercase();
    if let Some(from_idx) = h.find("from ") {
        if let Some(to_idx) = h.find(" to ") {
            if to_idx > from_idx {
                let min_str = &h[from_idx + 5..to_idx];
                let max_str = &h[to_idx + 4..];
                
                let min = extract_first_number(min_str)?;
                let max = extract_first_number(max_str)?;
                
                return Some((min, max));
            }
        }
    }
    None
}

// ⭐ NOUVEAU : Découpe intelligemment la chaîne en ignorant les virgules dans les guillemets/parenthèses
fn split_safe(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut depth = 0;

    for c in s.chars() {
        match c {
            '"' => { in_quotes = !in_quotes; current.push(c); }
            '(' => { depth += 1; current.push(c); }
            ')' => { depth -= 1; current.push(c); }
            ',' if !in_quotes && depth == 0 => {
                result.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(c),
        }
    }
    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }
    result
}

fn extract_choices(hint: &str) -> Option<Vec<Choice>> {
    let mut choices = Vec::new();
    let mut valid_count = 0;
    
    // ⭐ On normalise les guillemets intelligents et les "or"
    let normalized = hint
        .replace('“', "\"")
        .replace('”', "\"")
        .replace(" or ", ", ");

    if normalized.contains(" for ") || normalized.contains(" is ") || normalized.contains("__separator__") {
        let parts = split_safe(&normalized);
        for part in parts {
            let part = part.as_str();
            if part == "__separator__" {
                choices.push(Choice::Separator);
                continue;
            }
            
            let mut value = String::new();
            let mut label = String::new();
            
            if let Some((v, l)) = part.split_once(" for ") {
                value = v.trim().trim_matches('"').to_string();
                label = l.trim().trim_matches('"').to_string();
                valid_count += 1;
            } else if let Some((v, l)) = part.split_once(" is ") {
                label = v.trim().trim_matches('"').to_string();
                value = l.trim().trim_matches('"').to_string();
                valid_count += 1;
            } else {
                let clean_part = part.trim_matches('"').trim_matches('\'').trim();
                if !clean_part.is_empty() && clean_part.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_' || c == '-' || c == ' ') {
                    choices.push(Choice::Value { value: clean_part.to_string(), label: clean_part.to_string() });
                    valid_count += 1;
                } else {
                    return None;
                }
            }
            
            if !value.is_empty() && !label.is_empty() {
                choices.push(Choice::Value { value, label });
            }
        }
    } else {
        // ... (la suite de la fonction actuelle)
        let parts = split_safe(&normalized);
        let mut all_uppercase = true;
        for part in &parts {
            if part.is_empty() { all_uppercase = false; break; }
            let clean = part.trim_matches('"').trim_matches('\'');
            if !clean.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_' || c == '-') {
                all_uppercase = false;
                break;
            }
        }
        
        if all_uppercase && parts.len() >= 2 {
            for part in parts {
                let val = part.trim_matches('"').trim_matches('\'').to_string();
                choices.push(Choice::Value { value: val.clone(), label: val });
                valid_count += 1;
            }
        }
    }
    
    if valid_count >= 2 || (choices.len() > 0 && choices.iter().any(|c| matches!(c, Choice::Separator))) {
        Some(choices)
    } else {
        None
    }
}

// Parse une condition string en AST (ex: "A=1 and (B or C)")
fn parse_condition(cond_str: &str) -> Option<ConditionExpr> {
    let s = cond_str.trim();
    if s.is_empty() { return None; }
    
    // Gestion du OR (priorité basse)
    if let Some(idx) = s.to_lowercase().find(" or ") {
        let left = parse_condition(&s[..idx])?;
        let right = parse_condition(&s[idx+4..])?;
        return Some(ConditionExpr::Or { left: Box::new(left), right: Box::new(right) });
    }
    
    // Gestion du AND (priorité haute)
    if let Some(idx) = s.to_lowercase().find(" and ") {
        let left = parse_condition(&s[..idx])?;
        let right = parse_condition(&s[idx+5..])?;
        return Some(ConditionExpr::And { left: Box::new(left), right: Box::new(right) });
    }
    
    // Gestion du NOT
    if s.to_lowercase().starts_with("not ") || s.starts_with('!') {
        let rest = if s.starts_with('!') { &s[1..] } else { &s[4..] };
        let inner = parse_condition(rest.trim())?;
        return Some(ConditionExpr::Not { expr: Box::new(inner) });
    }
    
    // Gestion des parenthèses
    if s.starts_with('(') && s.ends_with(')') {
        return parse_condition(&s[1..s.len()-1]);
    }
    
    // Gestion de EQ (KEY=VALUE ou KEY:VALUE)
    if let Some((k, v)) = s.split_once('=') {
        return Some(ConditionExpr::Eq { key: k.trim().to_string(), value: v.trim().to_string() });
    }
    if let Some((k, v)) = s.split_once(':') {
        return Some(ConditionExpr::Eq { key: k.trim().to_string(), value: v.trim().to_string() });
    }
    
    // Sinon c'est un Truthy (ex: "AUTOMATIC_SPLITS")
    Some(ConditionExpr::Truthy { key: s.to_string() })
}


// ⭐ Transforme une clé brute en texte lisible (ex: OPACITY2 -> Opacity 2)
fn humanize_key(key: &str) -> String {
    let mut out = String::new();
    let mut prev_was_digit = false;
    for (i, c) in key.chars().enumerate() {
        if c == '_' {
            out.push(' ');
            prev_was_digit = false;
            continue;
        }
        if i == 0 {
            out.push(c);
        } else if c.is_uppercase() || (c.is_ascii_digit() && !prev_was_digit) {
            out.push(' ');
            out.push(c);
        } else {
            out.push(c.to_ascii_lowercase());
        }
        prev_was_digit = c.is_ascii_digit();
    }
    let mut chars = out.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[tauri::command]
fn read_csp_module_data(documents_path: String, ac_path: String, filename: String) -> Result<CspModuleData, String> {
    // 1. Lire le TEMPLATE (source de vérité pour la structure)
    let tpl_path = Path::new(&ac_path).join("extension").join("config").join(&filename);
    if !tpl_path.exists() {
        return Err(format!("Template introuvable : {}", filename));
    }
    let tpl_content = fs::read_to_string(&tpl_path).map_err(|e| e.to_string())?;

    // 2. Lire le fichier UTILISATEUR (s'il existe) pour les valeurs modifiées
    let user_dir = resolve_csp_dir(&documents_path);
    let user_path = user_dir.join(&filename);
    let user_content = if user_path.exists() {
        fs::read_to_string(&user_path).unwrap_or_default()
    } else {
        String::new()
    };

    // 3. Charger les valeurs du user
    let mut user_values: std::collections::HashMap<String, std::collections::HashMap<String, String>> = std::collections::HashMap::new();
    let mut current_user_sec = String::new();
    for line in user_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_user_sec = trimmed.trim_matches(|c| c == '[' || c == ']').to_string();
        } else if !current_user_sec.is_empty() && !trimmed.is_empty() && !trimmed.starts_with(';') {
            if let Some((k, v)) = trimmed.split_once('=') {
                user_values.entry(current_user_sec.clone()).or_default().insert(k.trim().to_string(), v.trim().split(';').next().unwrap_or("").trim().to_string());
            }
        }
    }

    let mut fullname = String::new();
    let mut description = String::new();
    let mut sections: Vec<CspSection> = Vec::new();
    let mut current_section: Option<CspSection> = None;
    
    let mut in_meta = false;
    let mut skip_section = false; // Pour les sections commentées

    let lines: Vec<&str> = tpl_content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() { i += 1; continue; }

        // Détection des sections (même commentées)
        if line.starts_with('[') && line.ends_with(']') {
            let sec_name = line.trim_matches(|c| c == '[' || c == ']');
            
            if sec_name == "ℹ" {
                in_meta = true;
                skip_section = true;
                i += 1;
                continue;
            }
            
            in_meta = false;
            skip_section = false;
            
            // Règle de masquage
            if sec_name.contains("DEV") || sec_name.contains("_DEV") || sec_name.starts_with("__") || sec_name.contains("HIDDEN") {
                skip_section = true;
            }

            // Détection du "; hidden" juste après le titre de la section
            if i + 1 < lines.len() {
                let next_line = lines[i+1].trim();
                if next_line == "; hidden" || next_line == ";; hidden" {
                    skip_section = true;
                    i += 1; // On saute la ligne ; hidden
                }
            }

                        if !skip_section {
                // ⭐ FIX : On sauvegarde la section précédente AVANT d'écraser la variable
                if let Some(sec) = current_section.take() {
                    if !sec.options.is_empty() || sec.has_enabled_toggle {
                        sections.push(sec);
                    }
                }

                let mut has_toggle = false;
                let mut tpl_enabled_val = true;
                
                for j in (i + 1)..lines.len() {
                    let l = lines[j].trim();
                    if l.starts_with('[') && l.ends_with(']') { break; } // Fin de section
                    if l.to_uppercase().starts_with("ENABLED=") && !l.starts_with(';') {
                        has_toggle = true;
                        tpl_enabled_val = l.split('=').nth(1).unwrap_or("1").split(';').next().unwrap_or("1").trim() != "0";
                        break;
                    }
                }

                let is_enabled = user_values.get(sec_name)
                    .and_then(|m| m.get("ENABLED"))
                    .map(|v| v == "1")
                    .unwrap_or(tpl_enabled_val);

                current_section = Some(CspSection {
                    name: sec_name.to_string(),
                    is_enabled,
                    has_enabled_toggle: has_toggle,
                    options: Vec::new(),
                });
            }
            i += 1;
            continue;
        }

        if in_meta {
            if line.starts_with("FULLNAME=") {
                fullname = line.split('=').nth(1).unwrap_or("").trim().to_string();
            } else if line.starts_with("DESCRIPTION=") {
                description = line.split('=').nth(1).unwrap_or("").trim().to_string();
            }
            i += 1;
            continue;
        }

        // Détection des options
        if !skip_section && current_section.is_some() {
            if !line.starts_with(';') && !line.starts_with('#') {
                if let Some((key, val, hints)) = parse_ini_line(line) {
                    if key == "ENABLED" || key == "ENABLED2" || key.starts_with("__") { 
                        i += 1; 
                        continue; 
                    }

                    let mut label = String::new();
                    let mut option_type = "text".to_string();
                    let mut min = None;
                    let mut max = None;
                    let mut step = None; // ⭐ AJOUTÉ ICI
                    let mut choices = Vec::new();
                    let mut condition = None;
                    let mut is_hidden = false;
                    let mut file_filter = None;
                    let mut lua_filter_dir = None; // ⭐ NOUVEAU

                    // ⭐ On vérifie d'abord si c'est un booléen (sécurité absolue)
                    let is_bool = hints.iter().any(|h| h.trim() == "1 or 0");
                    let mut is_percentage = false;

                    // Analyse des hints
                    for hint in hints {
                        if hint == "hidden" { is_hidden = true; continue; }
                        if hint == "1 or 0" { continue; } // Déjà géré par is_bool
                        
                        if hint.to_lowercase().contains("perc") { is_percentage = true; }

                        // ⭐ DÉTECTION DES FICHIERS
                        if hint.starts_with("file (") && hint.ends_with(")") {
                            option_type = "file".to_string();
                            file_filter = Some(hint[6..hint.len()-1].to_string());
                            continue;
                        }

                        // ⭐ DÉTECTION ROBUSTE DU DOSSIER DES FILTRES LUA
                        let hint_lower = hint.to_lowercase();
                        if hint_lower.starts_with("look for ") && hint_lower.contains(" in ") {
                            if let Some(idx) = hint_lower.find(" in ") {
                                let dir = hint[idx + 4..].trim();
                                if !dir.is_empty() {
                                    lua_filter_dir = Some(dir.to_string());
                                    continue;
                                }
                            }
                        }
                        
                        // On ignore les mots-clés parasites pour le label
                        if ["new", "updated", "round", "perc.", "perc"].contains(&hint.to_lowercase().as_str()) { continue; }
                        
                        // Range
                        if let Some((mn, mx)) = extract_range(&hint) {
                            if option_type == "text" { option_type = "number".to_string(); }
                            min = Some(mn);
                            max = Some(mx);

                            // ⭐ Détection de la précision (step)
                            let h_lower = hint.to_lowercase();
                            if h_lower.contains("round to ") {
                                let parts: Vec<&str> = h_lower.split("round to ").collect();
                                if parts.len() > 1 {
                                    let step_str: String = parts[1].chars().filter(|c| c.is_numeric() || *c == '.').collect();
                                    if let Ok(s) = step_str.parse::<f32>() { step = Some(s); }
                                }
                            } else if h_lower.contains("round") {
                                step = Some(1.0);
                            }
                            continue;
                        }

                        // Choix (Droplist) - Seulement si ce n'est pas un booléen
                        if !is_bool {
                            if let Some(ch) = extract_choices(&hint) {
                                option_type = "select".to_string();
                                choices = ch;
                                continue;
                            }
                        }

                        // Conditions
                        let cond_start = hint.to_lowercase();
                        if cond_start.contains("visible with ") {
                            let cond_str = &hint[hint.to_lowercase().find("visible with ").unwrap() + 13..];
                            condition = parse_condition(cond_str);
                        } else if cond_start.contains("hidden with ") {
                            let cond_str = &hint[hint.to_lowercase().find("hidden with ").unwrap() + 12..];
                            if let Some(c) = parse_condition(cond_str) {
                                condition = Some(ConditionExpr::Not { expr: Box::new(c) });
                            }
                        } else if cond_start.contains("only with ") {
                            let cond_str = &hint[hint.to_lowercase().find("only with ").unwrap() + 10..];
                            condition = parse_condition(cond_str);
                        } else if cond_start.contains("not available with ") {
                            let cond_str = &hint[hint.to_lowercase().find("not available with ").unwrap() + 19..];
                            if let Some(c) = parse_condition(cond_str) {
                                condition = Some(ConditionExpr::Not { expr: Box::new(c) });
                            }
                        } else if label.is_empty() {
                            // ⭐ On rejette seulement les hints qui COMMENCENT par un mot-clé technique
                            let lower_hint = hint.to_lowercase();
                            let is_metadata = 
                                lower_hint.starts_with("from ") ||
                                lower_hint.starts_with("visible with ") ||
                                lower_hint.starts_with("hidden with ") ||
                                lower_hint.starts_with("only with ") ||
                                lower_hint.starts_with("not available with ") ||
                                lower_hint.starts_with("file (") ||
                                lower_hint.starts_with("look for ") ||
                                lower_hint == "hidden" || lower_hint == "new" || 
                                lower_hint == "updated" || lower_hint == "1 or 0" ||
                                lower_hint.starts_with("round") ||
                                lower_hint == "perc" || lower_hint == "perc.";
                            
                            if !is_metadata {
                                label = hint.trim().trim_matches('"').to_string();
                            }
                        }
                    }

                    // ⭐ Si on a trouvé un dossier de filtres Lua, on scanne !
                    if let Some(dir) = lua_filter_dir {
                        let full_path = Path::new(&ac_path).join(&dir);
                        if full_path.exists() {
                            if let Ok(entries) = fs::read_dir(full_path) {
                                let mut folder_choices = Vec::new();
                                for entry in entries.flatten() {
                                    if entry.path().is_dir() {
                                        if let Some(name) = entry.file_name().to_str() {
                                            folder_choices.push(Choice::Value { value: name.to_string(), label: name.to_string() });
                                        }
                                    }
                                }
                                if !folder_choices.is_empty() {
                                    folder_choices.sort_by(|a, b| match (a, b) {
                                        (Choice::Value { value: v1, .. }, Choice::Value { value: v2, .. }) => v1.cmp(v2),
                                        _ => std::cmp::Ordering::Equal,
                                    });
                                    choices = folder_choices;
                                    option_type = "select".to_string();
                                }
                            }
                        }
                    }

                    // Si pas de label, on humanise la clé
                    if label.is_empty() || label == "hidden" {
                        label = humanize_key(&key);
                    }

                    if is_bool {
                        option_type = "bool".to_string();
                        choices.clear(); // On nettoie les fausses listes de choix
                    }

                    // ⭐ Calcul automatique du "step" si non spécifié (ex: de 0.1 à 1.2 -> pas de 0.1)
                    if step.is_none() {
                        if let (Some(mn), Some(mx)) = (min, max) {
                            let count_dec = |v: f32| -> u32 {
                                let s = v.to_string();
                                if let Some(idx) = s.find('.') { (s.len() - idx - 1) as u32 } else { 0 }
                            };
                            let max_dec = count_dec(mn).max(count_dec(mx));
                            step = Some(match max_dec {
                                0 => 1.0,
                                1 => 0.1,
                                2 => 0.01,
                                _ => 0.001,
                            });
                        }
                    }

                    // Valeur finale
                    let final_val = user_values.get(&current_section.as_ref().unwrap().name)
                        .and_then(|m| m.get(&key))
                        .cloned()
                        .unwrap_or(val);

                    // Détection auto si c'est un nombre (et qu'on n'a pas de range)
                    if option_type == "text" && final_val.parse::<f32>().is_ok() && !final_val.contains(',') {
                        option_type = "number".to_string();
                    }

                    current_section.as_mut().unwrap().options.push(CspOption {
                        key,
                        value: final_val,
                        label,
                        option_type,
                        min,
                        max,
                        step: step,
                        choices,
                        condition,
                        is_hidden,
                        is_percentage,
                        file_filter,
                    });
                }
            }
        }
        i += 1;
    }

        // ⭐ On sauvegarde la toute dernière section du fichier
    if let Some(sec) = current_section {
        if !sec.options.is_empty() || sec.has_enabled_toggle {
            sections.push(sec);
        }
    }

    // ⭐ Nettoyage : on vire les sections vides MAIS on garde celles qui ont juste un interrupteur (has_enabled_toggle)
    // Sinon la section [BASIC] d'un module qui n'a que "ENABLED=1" disparaîtrait de l'interface !
    sections.retain(|s: &CspSection| !s.options.is_empty() || s.has_enabled_toggle);

    Ok(CspModuleData {
        filename,
        fullname,
        description,
        sections,
    })
}

#[tauri::command]
fn set_csp_module_enabled(
    documents_path: String,
    ac_path: String,
    filename: String,
    section: String,
    enabled: bool,
    _default_enabled: bool, // ⭐ On ne l'utilise plus, on le lit direct dans le template
) -> Result<(), String> {
    // ⭐ 1. On cherche la VRAIE valeur par défaut dans le template
    let mut true_default = true;
    let tpl_path = Path::new(&ac_path).join("extension").join("config").join(&filename);
    if let Ok(tpl_content) = fs::read_to_string(&tpl_path) {
        let mut in_target_sec = false;
        for line in tpl_content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let sec_name = trimmed.trim_matches(|c| c == '[' || c == ']');
                in_target_sec = sec_name.eq_ignore_ascii_case(&section);
            } else if in_target_sec && trimmed.to_uppercase().starts_with("ENABLED=") {
                true_default = trimmed.split('=').nth(1).unwrap_or("1").split(';').next().unwrap_or("1").trim() != "0";
                break;
            }
        }
    }

    let file_path = ensure_user_csp_file(&documents_path, &ac_path, &filename)?;
    let content = fs::read_to_string(&file_path).unwrap_or_default();
    
    let normalized_content = content.replace("\r\n", "\n");
    let mut lines: Vec<String> = normalized_content.lines().map(|s| s.to_string()).collect();
    let section_header = format!("[{}]", section);

    let mut section_start_idx = None;
    let mut section_end_idx = lines.len();

    // 1. Trouver les limites de la section
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with(&section_header) {
            section_start_idx = Some(i);
        } else if section_start_idx.is_some() && trimmed.starts_with('[') && trimmed.ends_with(']') {
            section_end_idx = i;
            break;
        }
    }

    // 2. Appliquer les règles de CM (Diff)
    if let Some(start_idx) = section_start_idx {
        let mut enabled_idx = None;
        for i in (start_idx + 1)..section_end_idx {
            let trimmed = lines[i].trim();
            if trimmed.starts_with("ENABLED=") {
                enabled_idx = Some(i);
                break;
            }
        }

        if enabled == true_default {
            // On revient à la valeur par défaut
            if let Some(idx) = enabled_idx {
                lines.remove(idx);
                // ⭐ CORRECTION MAJEURE ICI : On doit ajuster la fin de section car on a supprimé une ligne !
                if section_end_idx > start_idx {
                    section_end_idx -= 1;
                }
            }
            
            // Vérifier si la section est vide
            let mut is_empty = true;
            for i in (start_idx + 1)..section_end_idx {
                if i < lines.len() && !lines[i].trim().is_empty() {
                    is_empty = false;
                    break;
                }
            }
            
            // Si elle est vide, on supprime la section entière
            if is_empty {
                // ⭐ SECURITÉ : On s'assure de ne pas dépasser la taille du tableau
                let safe_end = section_end_idx.min(lines.len());
                lines.drain(start_idx..safe_end);
                while start_idx < lines.len() && lines[start_idx].trim().is_empty() {
                    lines.remove(start_idx);
                }
            }
        } else {
            // On s'écarte du défaut : on ajoute/modifie ENABLED
            let val_str = if enabled { "ENABLED=1" } else { "ENABLED=0" };
            if let Some(idx) = enabled_idx {
                lines[idx] = val_str.to_string();
            } else {
                lines.insert(start_idx + 1, val_str.to_string());
            }
        }
    } else {
        // La section n'existe pas. On l'ajoute UNIQUEMENT si on s'écarte du défaut
        if enabled != true_default { // ⭐ Utilisation de true_default ici
            if !lines.is_empty() && !lines.last().unwrap().trim().is_empty() {
                lines.push("".to_string());
            }
            lines.push(section_header);
            lines.push(if enabled { "ENABLED=1" } else { "ENABLED=0" }.to_string());
        }
    }

    let final_content = lines.join("\r\n") + "\r\n";
    
    if file_path.exists() {
        let backup_path = file_path.with_extension("ini.bak");
        let _ = fs::copy(&file_path, backup_path);
    }
    
    fs::write(&file_path, final_content).map_err(|e| format!("Erreur d'écriture : {}", e))?;
    Ok(())
}

#[tauri::command]
fn write_csp_module(documents_path: String, ac_path: String, filename: String, content: String) -> Result<(), String> {
    let templates = get_csp_templates_list(&ac_path);
    if !templates.contains(&filename) {
        return Err(format!("Fichier non autorisé : {}", filename));
    }
    
    let file_path = ensure_user_csp_file(&documents_path, &ac_path, &filename)?;
    
    if file_path.exists() {
        let backup_path = file_path.with_extension("ini.bak");
        let _ = fs::copy(&file_path, backup_path);
    }

    fs::write(&file_path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(serde::Deserialize)]
struct IniUpdate {
    section: String,
    key: String,
    value: String,
}

#[tauri::command]
fn update_ini_values(
    documents_path: String,
    ac_path: String,
    filename: String,
    updates: Vec<IniUpdate>,
) -> Result<(), String> {
    let file_path = ensure_user_csp_file(&documents_path, &ac_path, &filename)?;
    let existing_content = fs::read_to_string(&file_path).unwrap_or_default();
    let normalized = existing_content.replace("\r\n", "\n");
    let mut lines: Vec<String> = normalized.lines().map(|s| s.to_string()).collect();

    // ⭐ 1. Lire le template pour connaître les valeurs par défaut
    let tpl_path = Path::new(&ac_path).join("extension").join("config").join(&filename);
    let mut default_values: std::collections::HashMap<String, std::collections::HashMap<String, String>> = std::collections::HashMap::new();
    
    if let Ok(tpl_content) = fs::read_to_string(tpl_path) {
        let mut current_sec = String::new();
        for line in tpl_content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_sec = trimmed.trim_matches(|c| c == '[' || c == ']').to_string();
            } else if !current_sec.is_empty() && !trimmed.is_empty() && !trimmed.starts_with(';') {
                if let Some((k, v)) = trimmed.split_once('=') {
                    let key = k.trim().to_string();
                    let val = v.trim().split(';').next().unwrap_or("").trim().to_string();
                    default_values.entry(current_sec.clone()).or_default().insert(key, val);
                }
            }
        }
    }

    // ⭐ 2. Appliquer les modifications (Logique de DIFF)
    for update in updates {
        let section_header = format!("[{}]", update.section);
        let mut section_start = None;
        let mut section_end = lines.len();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with(&section_header) {
                section_start = Some(i);
            } else if section_start.is_some() && trimmed.starts_with('[') && trimmed.ends_with(']') {
                section_end = i;
                break;
            }
        }

        let is_default = default_values.get(&update.section)
            .and_then(|m| m.get(&update.key))
            .map(|v| v == &update.value)
            .unwrap_or(false);

        if let Some(start) = section_start {
            let mut key_found = false;
            for i in (start + 1)..section_end {
                let trimmed = lines[i].trim();
                if trimmed.starts_with(&format!("{}=", update.key)) {
                    if is_default {
                        // ⭐ Si c'est la valeur par défaut, on SUPPRIME la ligne du fichier user
                        lines.remove(i);
                        section_end -= 1; // On ajuste la fin de section
                    } else {
                        // Sinon on met à jour la valeur
                        let comment = if let Some(pos) = trimmed.find(';') {
                            &trimmed[pos..]
                        } else {
                            ""
                        };
                        lines[i] = format!("{}={}{}", update.key, update.value, comment);
                    }
                    key_found = true;
                    break;
                }
            }
            
            // ⭐ Si la clé n'existe pas et qu'on doit la créer (car différente du défaut)
            if !key_found && !is_default {
                lines.insert(section_end, format!("{}={}", update.key, update.value));
                section_end += 1;
            }

            // ⭐ 3. Nettoyage : Si la section est devenue vide, on la supprime
            let mut section_is_empty = true;
            for i in (start + 1)..section_end {
                if i < lines.len() && !lines[i].trim().is_empty() {
                    section_is_empty = false;
                    break;
                }
            }
            if section_is_empty {
                lines.drain(start..section_end);
                while start < lines.len() && lines[start].trim().is_empty() {
                    lines.remove(start);
                }
            }

        } else {
            // La section n'existe pas encore, on l'ajoute seulement si on s'écarte du défaut
            if !is_default {
                if !lines.is_empty() && !lines.last().unwrap().trim().is_empty() {
                    lines.push("".to_string());
                }
                lines.push(section_header);
                lines.push(format!("{}={}", update.key, update.value));
            }
        }
    }

    let final_content = lines.join("\r\n") + "\r\n";
    if file_path.exists() {
        let backup_path = file_path.with_extension("ini.bak");
        let _ = fs::copy(&file_path, backup_path);
    }
    fs::write(&file_path, final_content).map_err(|e| format!("Erreur d'écriture : {}", e))?;
    Ok(())
}

// ⭐ GESTION DES PRESETS CSP

// ⭐ Trouve le dossier QARL_Presets en se basant sur le dossier extension (qui est toujours dans cfg)
fn get_qarl_presets_dir(documents_path: &str) -> std::path::PathBuf {
    let csp_dir = resolve_csp_dir(documents_path);
    // Si csp_dir est ".../cfg/extension", le parent est ".../cfg"
    if let Some(cfg_dir) = csp_dir.parent() {
        cfg_dir.join("QARL_Presets").join("CSP")
    } else {
        // Fallback au cas où
        std::path::Path::new(documents_path).join("QARL_Presets").join("CSP")
    }
}

#[tauri::command]
fn list_csp_presets(documents_path: String) -> Result<Vec<String>, String> {
    let dir = get_qarl_presets_dir(&documents_path);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut presets = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "ini") {
                if let Some(stem) = path.file_stem() {
                    presets.push(stem.to_string_lossy().to_string());
                }
            }
        }
    }
    presets.sort();
    Ok(presets)
}

#[tauri::command]
fn save_csp_preset(documents_path: String, ac_path: String, preset_name: String) -> Result<(), String> {
    let user_dir = resolve_csp_dir(&documents_path);
    let tpl_dir = if !ac_path.is_empty() { get_csp_template_path(&ac_path) } else { std::path::PathBuf::new() };
    
    let presets_dir = get_qarl_presets_dir(&documents_path);
    if !presets_dir.exists() {
        fs::create_dir_all(&presets_dir).map_err(|e| e.to_string())?;
    }
    
    let mut output = String::new();
    let mut affected = Vec::new();
    
        // 1. Lecture de tous les fichiers autorisés (scan dynamique)
    let templates = get_csp_templates_list(&ac_path);
    for filename in templates {
        let user_path = user_dir.join(&filename);
        let tpl_path = tpl_dir.join(&filename); // ⭐ Le & est important ici
        
        let path_to_use = if user_path.exists() { 
            user_path 
        } else if tpl_path.exists() { 
            tpl_path 
        } else { 
            continue; 
        };
        
        if let Ok(content) = fs::read_to_string(&path_to_use) {
            let content_trimmed = content.trim();
            if content_trimmed.is_empty() { continue; }
            
            // On extrait le nom du module (ex: "extra_fx.ini" -> "EXTRA_FX")
            let module_name = filename.replace(".ini", "").to_uppercase();
            
            // On ajoute les sections à la liste affected
            // Un fichier peut contenir plusieurs sections [EXTRA_FX:BASIC], [EXTRA_FX:SSAO], etc.
            let mut has_section = false;
                        for line in content_trimmed.lines() {
                let l = line.trim();
                if l.starts_with('[') && l.ends_with(']') {
                    let sec_name = l.trim_matches(|c| c == '[' || c == ']');
                    if !sec_name.starts_with("ℹ") && !sec_name.starts_with("META") && !sec_name.starts_with("HEADER") && !sec_name.starts_with("__") {
                        affected.push(sec_name.to_string());
                        has_section = true;
                    }
                }
            }
            
            if has_section {
                output.push_str(&format!("\n; --- {} ---\n", module_name));
                output.push_str(content_trimmed);
                output.push_str("\n\n");
            }
        }
    }
    
    // 2. Création de l'en-tête
    let header = format!(
        "[CONFIGURATION]\nVERSION=1\nAFFECTED_SECTIONS={}\n",
        affected.join(",")
    );
    
    let final_content = format!("{}{}", header, output);
    let file_path = presets_dir.join(format!("{}.ini", preset_name));
    fs::write(&file_path, final_content).map_err(|e| format!("Erreur écriture preset: {}", e))?;
    
    Ok(())
}

// ⭐ Fonction utilitaire pour appliquer un contenu de preset (utilisée par load et import)
fn apply_csp_preset_content(documents_path: &str, ac_path: &str, content: &str) -> Result<(), String> {
    // Dictionnaire temporaire pour stocker le contenu par fichier
    let mut file_buffers: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    
    let mut current_module = String::new();
    let mut affected_modules: Vec<String> = Vec::new();
    
    // ⭐ 1. On extrait la liste AFFECTED_SECTIONS
    let mut in_config = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.eq_ignore_ascii_case("[CONFIGURATION]") {
            in_config = true;
            continue;
        }
        if in_config && trimmed.starts_with('[') {
            in_config = false;
        }
        if in_config && trimmed.to_uppercase().starts_with("AFFECTED_SECTIONS=") {
            let val = trimmed.split('=').nth(1).unwrap_or("");
            for m in val.split(',') {
                let mod_name = m.trim().to_lowercase();
                if !mod_name.is_empty() && !affected_modules.contains(&mod_name) {
                    affected_modules.push(mod_name);
                }
            }
        }
    }

    // ⭐ 2. NETTOYAGE PRÉALABLE (On désactive TOUS les modules CSP pour repartir d'une base saine)
    let templates = get_csp_templates_list(ac_path);
    for filename in &templates {
        if let Ok(path) = ensure_user_csp_file(documents_path, ac_path, filename) {
            if path.exists() {
                if let Ok(existing_content) = fs::read_to_string(&path) {
                    let cleaned = disable_basic_section(&existing_content);
                    let _ = fs::write(&path, cleaned);
                }
            }
        }
    }
    
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let sec_name = trimmed.trim_matches(|c| c == '[' || c == ']');
            
            // On ignore la section CONFIGURATION et les métadonnées de CM (ex: GENERAL:ℹ)
            if sec_name == "CONFIGURATION" || sec_name.contains('ℹ') { 
                current_module = String::new(); // ⭐ On reset le module pour ignorer ses valeurs
                continue; 
            }
            
            // ⭐ CORRECTION MAJEURE : On sépare le module de la section
            // Ex: "GENERAL:AUDIO" -> module = "GENERAL", section = "AUDIO"
            let mut parts = sec_name.splitn(2, ':');
            let module_part = parts.next().unwrap_or(sec_name).to_lowercase();
            let clean_sec_name = parts.next().unwrap_or(sec_name);
            
            current_module = format!("{}.ini", module_part);
            
            // On s'assure que le buffer existe
            let buf = file_buffers.entry(current_module.clone()).or_insert_with(String::new);
            
            // On écrit la section nettoyée (ex: [AUDIO])
            buf.push_str(&format!("[{}]\n", clean_sec_name));
            
            // ⭐ SÉCURITÉ : Si c'est une section BASIC, on s'assure qu'elle sera activée
            if clean_sec_name.eq_ignore_ascii_case("BASIC") {
                buf.push_str("ENABLED=1\n");
            }
            continue;
        }
        
        // On n'écrit que si on a un module valide (ce qui exclut les blocs ℹ)
        if !current_module.is_empty() && !trimmed.is_empty() {
            let buf = file_buffers.entry(current_module.clone()).or_insert_with(String::new);
            buf.push_str(trimmed);
            buf.push('\n');
        }
    }
    
    // On écrit les buffers dans les bons fichiers avec un vrai système de fusion (Merge)
    for (filename, buf_content) in file_buffers {
        if templates.contains(&filename) {
            let target_path = ensure_user_csp_file(documents_path, ac_path, &filename)?;
            
            // Backup
            if target_path.exists() {
                let backup_path = target_path.with_extension("ini.bak");
                let _ = fs::copy(&target_path, backup_path);
            }

            // ⭐ On lit le fichier existant pour le fusionner
            let existing_content = fs::read_to_string(&target_path).unwrap_or_default();
            let final_content = merge_ini_content(&existing_content, &buf_content);
            
            fs::write(&target_path, final_content).map_err(|e| format!("Erreur écriture {}: {}", filename, e))?;
        }
    }
    
    Ok(())
}

// (Fonction force_enable_basic_section supprimée car remplacée par set_csp_module_enabled)

// ⭐ NOUVELLE FONCTION : Désactive un module en enlevant la ligne ENABLED
fn disable_basic_section(existing: &str) -> String {
    let eol = if existing.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = existing.lines().map(|s| s.to_string()).collect();
    
    let mut in_basic = false;
    let mut enabled_idx = None;
    
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let sec_name = trimmed.trim_matches(|c| c == '[' || c == ']');
            in_basic = sec_name.eq_ignore_ascii_case("BASIC") || sec_name.to_uppercase().ends_with(":BASIC");
            continue;
        }
        if in_basic && trimmed.to_uppercase().starts_with("ENABLED") {
            enabled_idx = Some(i);
            break;
        }
    }
    
    if let Some(idx) = enabled_idx {
        lines.remove(idx); // On supprime la ligne ENABLED
    }
    
    lines.join(eol) + eol
}

// ⭐ Nouvelle fonction utilitaire pour fusionner un patch INI dans un fichier existant
fn merge_ini_content(existing: &str, patch: &str) -> String {
    use std::collections::HashMap;
    
    // Parseur basique
    fn parse_ini(content: &str) -> (Vec<String>, HashMap<String, Vec<(String, String)>>) {
        let mut order = Vec::new();
        let mut sections: HashMap<String, Vec<(String, String)>> = HashMap::new();
        let mut current_sec = String::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_sec = trimmed.trim_matches(|c| c == '[' || c == ']').to_string();
                if !order.contains(&current_sec) {
                    order.push(current_sec.clone());
                }
                sections.entry(current_sec.clone()).or_default();
            } else if !current_sec.is_empty() && !trimmed.is_empty() && !trimmed.starts_with(';') {
                if let Some((k, v)) = line.split_once('=') {
                    let key = k.trim().to_string();
                    let val = v.trim().to_string();
                    
                    let sec_keys = sections.get_mut(&current_sec).unwrap();
                    if let Some(pos) = sec_keys.iter().position(|(ek, _)| ek == &key) {
                        sec_keys[pos].1 = val; // Remplace la valeur existante
                    } else {
                        sec_keys.push((key, val)); // Ajoute la nouvelle clé
                    }
                }
            }
        }
        (order, sections)
    }

    let (order, mut sections) = parse_ini(existing);
    let (patch_order, patch_sections) = parse_ini(patch);

    // On fusionne le patch dans les sections existantes
    for (sec, keys) in patch_sections {
        // On ignore les métadonnées CM qui commencent ou finissent par ℹ
        if sec.contains('ℹ') { continue; } 
        
        let existing_sec = sections.entry(sec.clone()).or_default();
        for (k, v) in keys {
            if let Some(pos) = existing_sec.iter().position(|(ek, _)| ek == &k) {
                existing_sec[pos].1 = v;
            } else {
                existing_sec.push((k, v));
            }
        }
    }

    // On reconstruit le fichier proprement
    let mut out = String::new();
    let mut all_sections = order.clone();
    for s in patch_order {
        if !s.contains('ℹ') && !all_sections.contains(&s) {
            all_sections.push(s);
        }
    }

    for sec in all_sections {
        if let Some(keys) = sections.get(&sec) {
            out.push_str(&format!("[{}]\n", sec));
            for (k, v) in keys {
                out.push_str(&format!("{}={}\n", k, v));
            }
            out.push('\n');
        }
    }
    
    out
}

#[tauri::command]
fn load_csp_preset(documents_path: String, ac_path: String, preset_name: String) -> Result<(), String> {
    let presets_dir = get_qarl_presets_dir(&documents_path);
    let file_path = presets_dir.join(format!("{}.ini", preset_name));
    
    if !file_path.exists() {
        return Err("Preset introuvable".into());
    }
    
    let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    apply_csp_preset_content(&documents_path, &ac_path, &content)
}

// ⭐ Nouvelle commande : Importer depuis un chemin absolu (fichier ou CM)
#[tauri::command]
fn import_csp_preset_from_path(documents_path: String, ac_path: String, source_path: String, save_to_qarl: bool) -> Result<(), String> {
    let src = Path::new(&source_path);
    if !src.exists() {
        return Err("Fichier source introuvable.".into());
    }
    
    let content = fs::read_to_string(src).map_err(|e| e.to_string())?;
    apply_csp_preset_content(&documents_path, &ac_path, &content)?;

    if save_to_qarl {
        let presets_dir = get_qarl_presets_dir(&documents_path);
        if !presets_dir.exists() {
            fs::create_dir_all(&presets_dir).map_err(|e| e.to_string())?;
        }
        let filename = src.file_name().unwrap_or_default();
        let dest = presets_dir.join(filename);
        let _ = fs::copy(src, dest); // On ignore l'erreur si le fichier existe déjà
    }

    Ok(())
}

// ⭐ Nouvelle commande : Lister les presets de Content Manager
#[tauri::command]
fn list_cm_presets() -> Result<Vec<serde_json::Value>, String> {
    let local_app_data = dirs::data_local_dir().ok_or("Impossible de trouver AppData/Local")?;
    let cm_dir = local_app_data.join("AcTools Content Manager").join("Presets").join("Custom Shaders Patch Presets");
    
    if !cm_dir.exists() {
        return Ok(Vec::new());
    }

    let mut presets = Vec::new();
    if let Ok(entries) = fs::read_dir(cm_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "ini") {
                if let Some(stem) = path.file_stem() {
                    presets.push(serde_json::json!({
                        "name": stem.to_string_lossy().to_string(),
                        "path": path.to_string_lossy().to_string()
                    }));
                }
            }
        }
    }
    presets.sort_by(|a, b| a["name"].as_str().unwrap_or("").cmp(b["name"].as_str().unwrap_or("")));
    Ok(presets)
}

#[tauri::command]
fn reset_single_csp_module(documents_path: String, ac_path: String, filename: String) -> Result<(), String> {
    let user_dir = resolve_csp_dir(&documents_path);
    let file_path = user_dir.join(&filename);
    
    if file_path.exists() {
        let _ = fs::remove_file(&file_path);
    }
    let backup_path = user_dir.join(format!("{}.bak", filename));
    if backup_path.exists() {
        let _ = fs::remove_file(&backup_path);
    }
    
    // On recrée un fichier vide pour que l'UI continue de le voir
    let _ = ensure_user_csp_file(&documents_path, &ac_path, &filename);
    Ok(())
}

#[tauri::command]
fn reset_csp_to_defaults(documents_path: String, ac_path: String) -> Result<(), String> {
    let user_dir = resolve_csp_dir(&documents_path);
    
    if !user_dir.exists() {
        return Ok(());
    }

    let templates = get_csp_templates_list(&ac_path);
    for filename in &templates {
        let file_path = user_dir.join(filename);
        if file_path.exists() {
            let _ = fs::remove_file(&file_path);
        }
        let backup_path = user_dir.join(format!("{}.bak", filename));
        if backup_path.exists() {
            let _ = fs::remove_file(&backup_path);
        }
    }
    
    // On recrée les fichiers depuis les templates pour que l'UI les voie immédiatement
    for filename in &templates {
        let _ = ensure_user_csp_file(&documents_path, &ac_path, filename);
    }
    
    Ok(())
}

// ⭐ Commande dédiée pour changer l'implémentation de Weather FX sans toucher au reste
#[tauri::command]
fn set_weather_fx_implementation(documents_path: String, ac_path: String, implementation: String) -> Result<(), String> {
    let filename = "weather_fx.ini";
    let file_path = ensure_user_csp_file(&documents_path, &ac_path, filename)?;

    // ⭐ SI ON DEMANDE DEFAULT : On écrase TOUT le fichier pour garantir qu'il n'y a AUCUN résidu de Pure
    if implementation == "Default" {
        let clean_content = "[BASIC]\nENABLED=1\nIMPLEMENTATION=Default\n";
        if file_path.exists() {
            let _ = fs::copy(&file_path, file_path.with_extension("ini.bak"));
        }
        fs::write(&file_path, clean_content).map_err(|e| e.to_string())?;
        return Ok(());
    }

    // Sinon (pour Pure / Pure LCS), on utilise le parseur intelligent pour préserver les autres options
    let content = fs::read_to_string(&file_path).unwrap_or_default();
    let eol = if content.contains("\r\n") { "\r\n" } else { "\n" };
    
    // ⭐ SÉCURITÉ ANTI-CRASH : Si le fichier est vide ou n'a pas de section [BASIC]
    // On écrase tout avec un contenu propre pour éviter le Panic d'index.
    if !content.contains("[BASIC]") {
        let clean_content = format!("[BASIC]\nCONTROLLER='pureCtrl static'\nENABLED=1\nIMPLEMENTATION={}\n", implementation);
        let _ = fs::copy(&file_path, file_path.with_extension("ini.bak"));
        fs::write(&file_path, clean_content).map_err(|e| e.to_string())?;
        return Ok(());
    }

    let mut new_lines: Vec<String> = Vec::new();
    let mut in_basic = false;
    let mut has_controller = false;
    let mut has_impl = false;

    for line in content.lines() {
        let trimmed = line.trim();
        
        // Gestion des sections
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let sec_name = trimmed.trim_matches(|c| c == '[' || c == ']');
            in_basic = sec_name.eq_ignore_ascii_case("BASIC");
        }

        // Traitement de la section BASIC pour réinjecter Pure
        if in_basic {
            if trimmed.to_uppercase().starts_with("CONTROLLER=") {
                new_lines.push("CONTROLLER='pureCtrl static'".to_string());
                has_controller = true;
                continue;
            }
            if trimmed.to_uppercase().starts_with("IMPLEMENTATION=") {
                new_lines.push(format!("IMPLEMENTATION={}", implementation));
                has_impl = true;
                continue;
            }
        }
        
        new_lines.push(line.to_string());
    }

    // On est sûr que [BASIC] existe à ce stade
    let basic_idx = new_lines.iter().position(|l| l.trim().eq_ignore_ascii_case("[BASIC]")).unwrap_or(0);

    // ⭐ Sécurité : Si on passe en Pure et qu'on n'a pas trouvé la ligne CONTROLLER, on l'ajoute
    if !has_controller {
        new_lines.insert(basic_idx + 1, "CONTROLLER='pureCtrl static'".to_string());
    }

    // ⭐ Sécurité : Si on n'a pas trouvé la ligne IMPLEMENTATION, on l'ajoute
    if !has_impl {
        // On recalcule l'index car l'insertion du CONTROLLER a pu décaler les lignes
        let basic_idx_final = new_lines.iter().position(|l| l.trim().eq_ignore_ascii_case("[BASIC]")).unwrap_or(0);
        new_lines.insert(basic_idx_final + 1, format!("IMPLEMENTATION={}", implementation));
    }

    let final_content = new_lines.join(eol) + eol;
    
    if file_path.exists() {
        let _ = fs::copy(&file_path, file_path.with_extension("ini.bak"));
    }
    fs::write(&file_path, final_content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_directinput_states() -> Result<Vec<serde_json::Value>, String> {
    let mut states = Vec::new();

    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::Devices::HumanInterfaceDevice::*;
        use windows::Win32::Foundation::*;
        use windows::Win32::System::Com::*;
        use windows::Win32::System::LibraryLoader::GetModuleHandleW;
        use windows::core::{Interface, GUID};

        // ⭐ CONSTRUCTION MANUELLE DU DIDATAFORMAT (c_dfDIJoystick2)
        let mut rgodf: Vec<DIOBJECTDATAFORMAT> = Vec::with_capacity(140);

        let axes_guids = [
            GUID::from_u128(0xA36D02E0_C9F3_11CF_BFC7_444553540000), // X
            GUID::from_u128(0xA36D02E1_C9F3_11CF_BFC7_444553540000), // Y
            GUID::from_u128(0xA36D02E2_C9F3_11CF_BFC7_444553540000), // Z
            GUID::from_u128(0xA36D02E3_C9F3_11CF_BFC7_444553540000), // Rx
            GUID::from_u128(0xA36D02E4_C9F3_11CF_BFC7_444553540000), // Ry
            GUID::from_u128(0xA36D02E5_C9F3_11CF_BFC7_444553540000), // Rz
            GUID::from_u128(0xA36D02E6_C9F3_11CF_BFC7_444553540000), // Slider0
            GUID::from_u128(0xA36D02E7_C9F3_11CF_BFC7_444553540000), // Slider1
        ];
        let axes_offsets = [0, 4, 8, 12, 16, 20, 24, 28];

        for i in 0..8 {
            rgodf.push(DIOBJECTDATAFORMAT {
                pguid: &axes_guids[i],
                dwOfs: axes_offsets[i],
                dwType: DIDFT_AXIS,
                dwFlags: 0,
            });
        }

        // POVs (4x)
        let guid_pov = GUID::from_u128(0xA36D02F2_C9F3_11CF_BFC7_444553540000);
        for i in 0..4 {
            rgodf.push(DIOBJECTDATAFORMAT {
                pguid: &guid_pov,
                dwOfs: 32 + (i * 4) as u32,
                dwType: DIDFT_POV,
                dwFlags: 0,
            });
        }

        // Boutons (128x)
        for i in 0..128 {
            rgodf.push(DIOBJECTDATAFORMAT {
                pguid: std::ptr::null(),
                dwOfs: 48 + i as u32,
                dwType: DIDFT_BUTTON,
                dwFlags: 0,
            });
        }

        let mut data_format = DIDATAFORMAT {
            dwSize: std::mem::size_of::<DIDATAFORMAT>() as u32,
            dwObjSize: std::mem::size_of::<DIOBJECTDATAFORMAT>() as u32,
            dwFlags: DIDF_ABSAXIS,
            dwDataSize: std::mem::size_of::<DIJOYSTATE2>() as u32,
            dwNumObjs: rgodf.len() as u32,
            rgodf: rgodf.as_mut_ptr(),
        };

        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let mut dinput_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        let hinst = GetModuleHandleW(None).unwrap();

        if DirectInput8Create(
            hinst,
            0x0800,
            &IDirectInput8W::IID,
            &mut dinput_ptr,
            None,
        ).is_ok() {
            if !dinput_ptr.is_null() {
                let di = IDirectInput8W::from_raw(dinput_ptr);

                unsafe extern "system" fn enum_callback(
                    lpddi: *mut DIDEVICEINSTANCEW, 
                    pv_ref: *mut std::ffi::c_void,
                ) -> BOOL {
                    unsafe {
                        let di_ptr = &mut *(pv_ref as *mut (IDirectInput8W, Vec<serde_json::Value>, *mut DIDATAFORMAT));
                        let di = &di_ptr.0;
                        let states = &mut di_ptr.1;
                        let c_df = di_ptr.2;
                        let inst = &*lpddi;

                        let iguid = format!(
                            "{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
                            inst.guidInstance.data1, inst.guidInstance.data2, inst.guidInstance.data3,
                            inst.guidInstance.data4[0], inst.guidInstance.data4[1],
                            inst.guidInstance.data4[2], inst.guidInstance.data4[3],
                            inst.guidInstance.data4[4], inst.guidInstance.data4[5],
                            inst.guidInstance.data4[6], inst.guidInstance.data4[7]
                        );

                        let mut device_opt: Option<IDirectInputDevice8W> = None;
                        if di.CreateDevice(&inst.guidInstance, &mut device_opt, None).is_ok() {
                            if let Some(device) = device_opt {
                                if device.SetDataFormat(c_df).is_ok() {
                                    let _ = device.Acquire();
                                    let mut js: DIJOYSTATE2 = std::mem::zeroed();
                                    
                                    if device.GetDeviceState(std::mem::size_of::<DIJOYSTATE2>() as u32, &mut js as *mut _ as *mut _).is_ok() {
                                        let axes = vec![
                                            js.lX, js.lY, js.lZ, js.lRx, js.lRy, js.lRz,
                                            js.rglSlider[0], js.rglSlider[1]
                                        ];
                                        let mut buttons = Vec::new();
                                        for i in 0..128 {
                                            buttons.push((js.rgbButtons[i] & 0x80) != 0);
                                        }
                                        
                                        states.push(serde_json::json!({
                                            "iguid": iguid,
                                            "axes": axes,
                                            "buttons": buttons
                                        }));
                                    }
                                }
                            }
                        }
                    }
                    TRUE
                }

                let mut data = (di.clone(), Vec::new(), &mut data_format as *mut DIDATAFORMAT);
                let _ = di.EnumDevices(
                    DI8DEVCLASS_GAMECTRL,
                    Some(enum_callback),
                    &mut data as *mut _ as *mut std::ffi::c_void,
                    DIEDFL_ATTACHEDONLY,
                );
                states = data.1;
            }
        }
        let _ = CoUninitialize();
    }

    Ok(states)
}

// ⭐ NOUVEAU : Lecture du FOV depuis camera_onboard.ini
#[tauri::command]
fn read_onboard_fov(documents_path: String) -> Result<i32, String> {
    let cfg_dir = resolve_cfg_dir(Path::new(&documents_path));
    let file_path = cfg_dir.join("camera_onboard.ini");
    
    if !file_path.exists() {
        return Ok(56); // Valeur par défaut
    }
    
    if let Ok(content) = fs::read_to_string(&file_path) {
        let mut in_mode = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                in_mode = trimmed.eq_ignore_ascii_case("[MODE]");
            } else if in_mode && trimmed.to_uppercase().starts_with("FOV=") {
                if let Some(val) = trimmed.split('=').nth(1) {
                    return Ok(val.trim().parse::<i32>().unwrap_or(56));
                }
            }
        }
    }
    Ok(56)
}

// ⭐ NOUVEAU : Écriture du FOV (préserve le reste du fichier)
#[tauri::command]
fn write_onboard_fov(documents_path: String, fov: i32) -> Result<(), String> {
    let cfg_dir = resolve_cfg_dir(Path::new(&documents_path));
    let file_path = cfg_dir.join("camera_onboard.ini");
    
    let mut lines: Vec<String> = if file_path.exists() {
        fs::read_to_string(&file_path)
            .unwrap_or_default()
            .lines()
            .map(|s| s.to_string())
            .collect()
    } else {
        vec!["[MODE]".to_string(), "IS_WORLD_ALIGNED=0".to_string()]
    };

    let mut in_mode = false;
    let mut fov_idx = None;
    let mut mode_idx = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_mode = trimmed.eq_ignore_ascii_case("[MODE]");
            if in_mode { mode_idx = Some(i); }
        } else if in_mode && trimmed.to_uppercase().starts_with("FOV=") {
            fov_idx = Some(i);
            break;
        }
    }

    let new_line = format!("FOV={}", fov);
    
    if let Some(idx) = fov_idx {
        lines[idx] = new_line;
    } else if let Some(idx) = mode_idx {
        lines.insert(idx + 1, new_line);
    } else {
        lines.insert(0, "[MODE]".to_string());
        lines.insert(1, "IS_WORLD_ALIGNED=0".to_string());
        lines.insert(2, new_line);
    }

    // Sécurité : Ajouter un saut de ligne à la fin si absent
    let eol = if lines.join("").contains("\r") { "\r\n" } else { "\n" };
    let final_content = lines.join(eol) + eol;
    
        fs::write(&file_path, final_content).map_err(|e| format!("Erreur écriture camera_onboard.ini: {}", e))?;
    Ok(())
}

#[tauri::command]
fn list_audio_devices() -> Result<Vec<String>, String> {
    let host = cpal::default_host();
    let mut names = Vec::new();
    
    if let Ok(devices) = host.output_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                names.push(name);
            }
        }
    }
    
    names.sort();
    names.dedup();
    Ok(names)
}

// ⭐ GESTION DES PRESETS DE SCHEDULE (CSP)

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct SchedulePresetData {
    slots: i32,
    time_per_slot: i32,
    random: bool,
    slot_values: Vec<i32>,
}

fn get_schedule_presets_dir(documents_path: &str) -> Result<std::path::PathBuf, String> {
    let cfg_dir = resolve_cfg_dir(Path::new(documents_path));
    let dir = cfg_dir.join("QARL_Presets").join("Schedule");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(dir)
}

#[tauri::command]
fn list_schedule_presets(documents_path: String) -> Result<Vec<String>, String> {
    let dir = get_schedule_presets_dir(&documents_path)?;
    let mut presets = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "json") {
                if let Some(stem) = path.file_stem() {
                    presets.push(stem.to_string_lossy().to_string());
                }
            }
        }
    }
    presets.sort();
    Ok(presets)
}

#[tauri::command]
fn save_schedule_preset(documents_path: String, name: String, data: SchedulePresetData) -> Result<(), String> {
    let dir = get_schedule_presets_dir(&documents_path)?;
    let safe_name = name.trim().replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "");
    if safe_name.is_empty() { return Err("Nom de preset invalide".into()); }
    
    let file_path = dir.join(format!("{}.json", safe_name));
    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    fs::write(file_path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn load_schedule_preset(documents_path: String, name: String) -> Result<SchedulePresetData, String> {
    let dir = get_schedule_presets_dir(&documents_path)?;
    let file_path = dir.join(format!("{}.json", name));
    if !file_path.exists() {
        return Err("Preset introuvable".into());
    }
    let content = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let data = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(data)
}

#[tauri::command]
fn delete_schedule_preset(documents_path: String, name: String) -> Result<(), String> {
    let dir = get_schedule_presets_dir(&documents_path)?;
    let file_path = dir.join(format!("{}.json", name));
    if file_path.exists() {
        fs::remove_file(file_path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn install_csp_archive(
    ac_path: String,
    archive_path: String,
    on_event: tauri::ipc::Channel<serde_json::Value>
) -> Result<String, String> {
    let ac_root = Path::new(&ac_path);
    let mut archive = unarc_rs::unified::ArchiveFormat::open_path(&archive_path)
        .map_err(|e| format!("Format d'archive non supporté ou corrompu: {}", e))?;

    // 1. Analyser l'archive pour trouver le préfixe (findAcRootInTemp)
    let mut entries_info = Vec::new();
    let mut min_prefix_len = usize::MAX;

    while let Some(entry) = archive.next_entry().map_err(|e| e.to_string())? {
        let name = entry.name().replace("\\", "/");
        entries_info.push(name.clone());

        // On cherche les marqueurs de racine AC (comme dans cspArchiveInstaller.js)
        let markers = ["/extension/", "/system/", "/content/weather/", "/apps/", "/dwrite.dll"];
        for m in &markers {
            if let Some(idx) = name.find(m) {
                let prefix = if idx == 0 { 0 } else { idx + 1 };
                if prefix < min_prefix_len { min_prefix_len = prefix; }
            }
        }
        
        // Si le fichier est exactement à la racine
        if name.starts_with("extension/") || name.starts_with("system/") || name.starts_with("content/weather/") || name.starts_with("apps/") || name.starts_with("dwrite.dll") {
            min_prefix_len = 0;
        }
    }

    if entries_info.is_empty() {
        return Err("L'archive est vide.".into());
    }

    // Si aucun préfixe n'est trouvé (ex: archive mal foutue), on extrait tel quel
    let strip_len = if min_prefix_len == usize::MAX { 0 } else { min_prefix_len };

    // 2. Extraction
    let total_files = entries_info.len() as f64;
    let mut extracted_count = 0.0;

    let _ = on_event.send(serde_json::json!({ "type": "start", "total": total_files }));

    let mut archive_2 = unarc_rs::unified::ArchiveFormat::open_path(&archive_path)
        .map_err(|e| format!("Erreur réouverture archive: {}", e))?;

    while let Some(entry) = archive_2.next_entry().map_err(|e| e.to_string())? {
        let entry_name = entry.name().replace("\\", "/");
        let entry_path = Path::new(&entry_name);

        // Sécurité anti Zip-Slip
        if entry_path.components().any(|c| c == std::path::Component::ParentDir) {
            continue;
        }

        // On retire le préfixe s'il y en a un
        let clean_name = if entry_name.len() > strip_len {
            &entry_name[strip_len..]
        } else {
            continue;
        };

        if clean_name.is_empty() { continue; }

        let out_path = ac_root.join(clean_name);

        if entry.is_directory() {
            std::fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut out_file = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
            archive_2.read_to(&entry, &mut out_file).map_err(|e| e.to_string())?;
        }

        extracted_count += 1.0;
        let percent = (extracted_count / total_files) * 100.0;
        let _ = on_event.send(serde_json::json!({
            "type": "progress",
            "percent": percent,
            "current_file": clean_name
        }));
    }

    // ⭐ Activer les apps Python de Sol si c'est Sol qui a été installé
    if entries_info.iter().any(|n| n.to_lowercase().contains("sol")) {
        if let Some(docs) = dirs::document_dir() {
            let cfg_dir = docs.join("Assetto Corsa").join("cfg");
            let ini_path = cfg_dir.join("python.ini");
            
            // On s'assure que le dossier existe
            let _ = std::fs::create_dir_all(&cfg_dir);
            
            let mut lines: Vec<String> = std::fs::read_to_string(&ini_path)
                .unwrap_or_default()
                .lines()
                .map(|s| s.to_string())
                .collect();
            let eol = if lines.join("").contains("\r") { "\r\n" } else { "\n" }; // Détection basique

            // ⭐ On s'assure que le fichier finit par un saut de ligne
            if !lines.is_empty() && !lines.last().unwrap().trim().is_empty() {
                lines.push(String::new());
            }

            let sol_apps = ["SOL_CONFIG", "SOL_CUSTOM_WEATHER", "SOL_PLANNER", "SOL_WEATHER"];
            for app in sol_apps {
                let header = format!("[{}]", app).to_uppercase();
                let mut found_section = false;
                let mut active_idx = None;
                let mut _section_end_idx = lines.len();

                // 1. Chercher la section et la ligne ACTIVE
                for (i, line) in lines.iter().enumerate() {
                    let trimmed = line.trim().to_uppercase();
                    if trimmed.starts_with('[') && trimmed.ends_with(']') {
                        if found_section {
                            _section_end_idx = i;
                            break;
                        }
                        if trimmed == header {
                            found_section = true;
                        }
                    } else if found_section && trimmed.starts_with("ACTIVE=") {
                        active_idx = Some(i);
                    }
                }

                // 2. Appliquer les modifications
                if found_section {
                    if let Some(idx) = active_idx {
                        lines[idx] = "ACTIVE=1".to_string();
                    } else {
                        // Insérer ACTIVE=1 juste après le header
                        let header_idx = lines.iter().position(|l| l.trim().to_uppercase() == header).unwrap();
                        lines.insert(header_idx + 1, "ACTIVE=1".to_string());
                    }
                } else {
                    // Ajouter la section complète à la fin
                    lines.push(header);
                    lines.push("ACTIVE=1".to_string());
                    lines.push(String::new());
                }
            }

            let final_content = lines.join(eol);
            let _ = std::fs::write(ini_path, final_content);
            println!("✅ Apps Python de Sol activées dans python.ini");
        }
    }

    // ⭐ Activer les apps Python de Sol si c'est Sol qui a été installé
    if entries_info.iter().any(|n| n.to_lowercase().contains("sol")) {
        if let Some(docs) = dirs::document_dir() {
            let cfg_dir = docs.join("Assetto Corsa").join("cfg");
            let ini_path = cfg_dir.join("python.ini");
            
            // On s'assure que le dossier existe
            let _ = std::fs::create_dir_all(&cfg_dir);
            
            let raw_content = std::fs::read_to_string(&ini_path).unwrap_or_default();
            let eol = if raw_content.contains("\r\n") { "\r\n" } else { "\n" };
            
            let mut lines: Vec<String> = raw_content.lines().map(|s| s.to_string()).collect();
            // ⭐ On s'assure que le fichier finit par un saut de ligne
            if !lines.is_empty() && !lines.last().unwrap().trim().is_empty() {
                lines.push(String::new());
            }

            let sol_apps = ["SOL_CONFIG", "SOL_CUSTOM_WEATHER", "SOL_PLANNER", "SOL_WEATHER"];
            for app in sol_apps {
                let header = format!("[{}]", app).to_uppercase();
                let mut found_section = false;
                let mut active_idx = None;
                let mut header_idx = None;

                // 1. Chercher la section et la ligne ACTIVE
                for (i, line) in lines.iter().enumerate() {
                    let trimmed = line.trim().to_uppercase();
                    if trimmed.starts_with('[') && trimmed.ends_with(']') {
                        if trimmed == header {
                            found_section = true;
                            header_idx = Some(i);
                        } else if found_section && active_idx.is_none() {
                            // On est sorti de la section sans trouver ACTIVE
                            break;
                        }
                    } else if found_section && trimmed.starts_with("ACTIVE=") {
                        active_idx = Some(i);
                        break;
                    }
                }

                // 2. Appliquer les modifications
                if found_section {
                    if let Some(idx) = active_idx {
                        lines[idx] = "ACTIVE=1".to_string();
                    } else if let Some(idx) = header_idx {
                        // Insérer ACTIVE=1 juste après le header
                        lines.insert(idx + 1, "ACTIVE=1".to_string());
                    }
                } else {
                    // Ajouter la section complète à la fin
                    lines.push(header);
                    lines.push("ACTIVE=1".to_string());
                    lines.push(String::new());
                }
            }

            let final_content = lines.join(eol) + eol;
            let _ = std::fs::write(ini_path, final_content);
            println!("✅ Apps Python de Sol activées dans python.ini");
        }
    }

    Ok("✅ Installation terminée avec succès !".into())
}

// ⭐ GESTION DES PRESETS VIDÉO

fn get_video_presets_dir(documents_path: &str) -> Result<std::path::PathBuf, String> {
    let cfg_dir = resolve_cfg_dir(Path::new(documents_path));
    let dir = cfg_dir.join("QARL_Presets").join("Video");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(dir)
}

#[tauri::command]
fn list_video_presets(documents_path: String) -> Result<Vec<String>, String> {
    let dir = get_video_presets_dir(&documents_path)?;
    let mut presets = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "ini") {
                if let Some(stem) = path.file_stem() {
                    presets.push(stem.to_string_lossy().to_string());
                }
            }
        }
    }
    presets.sort();
    Ok(presets)
}

#[tauri::command]
fn save_video_preset(documents_path: String, name: String, content: String) -> Result<(), String> {
    let dir = get_video_presets_dir(&documents_path)?;
    let safe_name = name.trim().replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");
    if safe_name.is_empty() { return Err("Nom de preset invalide".into()); }
    
    let file_path = dir.join(format!("{}.ini", safe_name));
    fs::write(file_path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn load_video_preset(documents_path: String, name: String) -> Result<String, String> {
    let dir = get_video_presets_dir(&documents_path)?;
    let file_path = dir.join(format!("{}.ini", name));
    if !file_path.exists() {
        return Err("Preset introuvable".into());
    }
    let raw_content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    
    let mut weather_fx = "Default".to_string();
    let mut clean_content = raw_content.clone();
    
    // ⭐ On extrait la section [WEATHER_FX] si elle existe
    if let Some(idx) = raw_content.find("[WEATHER_FX]") {
        let before = &raw_content[..idx];
        let after = &raw_content[idx..];
        
        for line in after.lines() {
            if line.to_uppercase().starts_with("IMPLEMENTATION=") {
                weather_fx = line.split('=').nth(1).unwrap_or("Default").trim().to_string();
                break;
            }
        }
        
        let next_sec = after[1..].find('[').map(|i| i + idx + 1).unwrap_or(raw_content.len());
        clean_content = format!("{}{}", before, &raw_content[next_sec..]);
    }

    // On écrit le contenu nettoyé dans le vrai fichier video.ini
    let base_path = Path::new(&documents_path);
    let mut cfg_dir = base_path.join("cfg");
    if !cfg_dir.exists() {
        cfg_dir = base_path.join("Assetto Corsa").join("cfg");
    }
    if !cfg_dir.exists() {
        return Err("Dossier cfg introuvable".into());
    }
    
    let video_ini_path = cfg_dir.join("video.ini");
    if video_ini_path.exists() {
        let _ = fs::copy(&video_ini_path, video_ini_path.with_extension("ini.bak"));
    }
    fs::write(&video_ini_path, clean_content).map_err(|e| e.to_string())?;
    
    Ok(weather_fx)
}

#[tauri::command]
fn delete_video_preset(documents_path: String, name: String) -> Result<(), String> {
    let dir = get_video_presets_dir(&documents_path)?;
    let file_path = dir.join(format!("{}.ini", name));
    if file_path.exists() {
        fs::remove_file(file_path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[derive(serde::Serialize, Clone)]
struct MediaFile {
    name: String,
    path: String,
    thumbnail_path: String, // ⭐ Nouveau
    size: u64,
    date: u64,
    file_type: String,
}

fn get_thumb_cache_dir() -> std::path::PathBuf {
    let mut dir = dirs::data_local_dir().unwrap_or_default();
    dir.push("QARL");
    dir.push("thumbnails");
    if !dir.exists() { let _ = fs::create_dir_all(&dir); }
    dir
}

#[tauri::command]
async fn get_media_files(documents_path: String, app_handle: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let base = Path::new(&documents_path);
    let mut screens = Vec::new();
    let mut replays = Vec::new();
    let mut missing_thumbs = Vec::new(); // ⭐ Nouveau

    // ⭐ Détection robuste : le chemin pointe-t-il déjà vers "Assetto Corsa" ou vers "Documents" ?
    let ac_docs = if base.join("cfg").exists() {
        base.to_path_buf()
    } else if base.join("Assetto Corsa").join("cfg").exists() {
        base.join("Assetto Corsa")
    } else {
        base.join("Assetto Corsa") // Fallback
    };

    let cache_dir = get_thumb_cache_dir();
    let screens_dir = ac_docs.join("screens");
    if screens_dir.exists() {
        if let Ok(entries) = fs::read_dir(screens_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |e| e == "jpg" || e == "png") {
                    let metadata = entry.metadata().unwrap();
                    let date = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                    
                    let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let stem = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                    let thumb_name = format!("thumb_{}_{}.jpg", stem, date);
                    let thumb_path = cache_dir.join(&thumb_name);

                    // ⭐ On ne génère PLUS la miniature ici (ça bloquait l'interface)
                    let final_thumb;
                    if thumb_path.exists() {
                        final_thumb = thumb_path.to_string_lossy().replace("\\", "/");
                    } else {
                        final_thumb = path.to_string_lossy().replace("\\", "/");
                        missing_thumbs.push((path.clone(), thumb_path.clone())); // On met en attente
                    }

                    screens.push(MediaFile {
                        name,
                        path: path.to_string_lossy().replace("\\", "/"),
                        thumbnail_path: final_thumb,
                        size: metadata.len(),
                        date,
                        file_type: "photo".to_string(),
                    });
                }
            }
        }
    }

    let replay_dir = ac_docs.join("replay");
    let temp_replay_dir = replay_dir.join("temp");

    // ⭐ NOUVEAU : On déplace automatiquement les replays de 'temp' vers le dossier principal
    if temp_replay_dir.exists() {
        if let Ok(entries) = fs::read_dir(&temp_replay_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |e| e == "acreplay") {
                    let file_name = path.file_name().unwrap_or_default();
                    let dest_path = replay_dir.join(file_name);
                    
                    // Si le fichier existe déjà à destination, on le supprime pour le remplacer
                    if dest_path.exists() {
                        let _ = fs::remove_file(&dest_path);
                    }
                    
                    // On déplace le fichier
                    let _ = fs::rename(&path, &dest_path);
                    
                    // Fallback si le rename échoue (ex: disques différents)
                    if !dest_path.exists() {
                        if fs::copy(&path, &dest_path).is_ok() {
                            let _ = fs::remove_file(&path);
                        }
                    }
                }
            }
        }
    }

    // Scan normal du dossier replay (qui contient maintenant les anciens fichiers de temp)
    if replay_dir.exists() {
        if let Ok(entries) = fs::read_dir(replay_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |e| e == "acreplay") {
                    let metadata = entry.metadata().unwrap();
                    let date = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                    replays.push(MediaFile {
                        name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                        path: path.to_string_lossy().replace("\\", "/"),
                        thumbnail_path: "".to_string(), // Pas de miniature pour les replays
                        size: metadata.len(),
                        date,
                        file_type: "replay".to_string(),
                    });
                }
            }
        }
    }

    // On trie du plus récent au plus vieux
    screens.sort_by(|a, b| b.date.cmp(&a.date));
    replays.sort_by(|a, b| b.date.cmp(&a.date));

    // ⭐ On génère les miniatures en arrière-plan (Thread séparé)
    if !missing_thumbs.is_empty() {
        let handle = app_handle.clone();
        std::thread::spawn(move || {
            for (original_path, thumb_path) in missing_thumbs {
                if let Ok(img) = image::open(&original_path) {
                    let thumb = img.thumbnail(300, 300);
                    let _ = thumb.save(&thumb_path);
                }
            }
            // On prévient l'interface que c'est prêt
            let _ = handle.emit("thumbs-ready", ());
        });
    }

    Ok(serde_json::json!({ "screens": screens, "replays": replays }))
}

#[tauri::command]
fn get_replay_info(file_path: String) -> Result<Vec<String>, String> {
    use std::io::Read;
    let file = fs::File::open(&file_path).map_err(|e| e.to_string())?;
    let mut reader = std::io::BufReader::new(file);
    let mut buffer = vec![0u8; 8192]; // On lit les 8 premiers Ko
    let bytes_read = reader.read(&mut buffer).map_err(|e| e.to_string())?;
    
    let mut strings = Vec::new();
    let mut current_string = String::new();

    for &b in &buffer[..bytes_read] {
        if b.is_ascii_graphic() || b == b'_' {
            current_string.push(b as char);
        } else {
            if current_string.len() >= 4 {
                let cleaned = current_string.trim().to_string();
                if !cleaned.is_empty() && !strings.contains(&cleaned) {
                    strings.push(cleaned);
                }
            }
            current_string.clear();
        }
    }

    // On filtre pour ne garder que ce qui ressemble à des IDs (lettres, chiffres, underscores)
    let filtered: Vec<String> = strings.into_iter()
        .filter(|s| s.chars().any(|c| c.is_ascii_alphabetic()) && s.len() < 50)
        .collect();

    Ok(filtered)
}

#[tauri::command]
fn launch_replay(app_handle: tauri::AppHandle, ac_path: String, replay_filename: String) -> Result<(), String> {
    let ac_exe = Path::new(&ac_path).join("acs.exe");
    
    if !ac_exe.exists() {
        return Err(format!("acs.exe introuvable dans le dossier : {}", ac_path));
    }

    // ⭐ 1. Modifier le race.ini pour injecter le replay
    if let Some(docs) = dirs::document_dir() {
        let cfg_dir = docs.join("Assetto Corsa").join("cfg");
        let race_ini_path = cfg_dir.join("race.ini");
        
        if race_ini_path.exists() {
            if let Ok(content) = fs::read_to_string(&race_ini_path) {
                let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
                let mut in_replay = false;
                let mut found_filename = false;
                let mut found_active = false;

                for i in 0..lines.len() {
                    let trimmed = lines[i].trim();
                    if trimmed.starts_with('[') && trimmed.ends_with(']') {
                        in_replay = trimmed.eq_ignore_ascii_case("[REPLAY]");
                    } else if in_replay {
                        if trimmed.to_uppercase().starts_with("FILENAME=") {
                            lines[i] = format!("FILENAME={}", replay_filename);
                            found_filename = true;
                        } else if trimmed.to_uppercase().starts_with("ACTIVE=") {
                            lines[i] = "ACTIVE=1".to_string();
                            found_active = true;
                        }
                    }
                }

                // Si la section [REPLAY] n'existait pas ou n'avait pas ces clés, on les ajoute à la fin
                if !found_filename {
                    lines.push(format!("FILENAME={}", replay_filename));
                }
                if !found_active {
                    lines.push("ACTIVE=1".to_string());
                }

                let new_content = lines.join("\r\n") + "\r\n";
                let _ = fs::write(&race_ini_path, new_content);
            }
        }
    }

    // ⭐ 2. Lancer AC SANS AUCUN ARGUMENT
    let mut cmd = std::process::Command::new(ac_exe);
    cmd.current_dir(&ac_path);
    cmd.env("SteamAppId", "244210");
    cmd.env("SteamGameId", "244210");
    
    // ⭐ On attend la fermeture du jeu en arrière-plan
    let mut child = cmd.spawn().map_err(|e| format!("Erreur lors du lancement du replay: {}", e))?;
    let handle = app_handle.clone();
    std::thread::spawn(move || {
        let _ = child.wait();
        let _ = handle.emit("ac-closed", ());
    });

    Ok(())
}

#[tauri::command]
fn delete_media_file(file_path: String) -> Result<(), String> {
    let path = Path::new(&file_path);
    if path.exists() {
        // ⭐ On tente de supprimer les miniatures associées
        if let Some(stem) = path.file_stem() {
            let stem_str = stem.to_string_lossy();
            let cache_dir = get_thumb_cache_dir();
            if let Ok(entries) = fs::read_dir(&cache_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with(&format!("thumb_{}_", stem_str)) {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }
        fs::remove_file(path).map_err(|e| format!("Erreur de suppression: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
fn write_specific_ini(file_path: String, content: String) -> Result<(), String> {
    let path = Path::new(&file_path);
    if path.exists() {
        let backup_path = path.with_extension("ini.bak");
        let _ = fs::copy(path, backup_path);
    } else if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Erreur création dossier: {}", e))?;
    }
    fs::write(path, content).map_err(|e| format!("Erreur d'écriture: {}", e))?;
    Ok(())
}

// ⭐ NOUVEAU : Extrait les hints (commentaires INI après ';') pour les tooltips
fn extract_hints_from_content(content: &str) -> std::collections::HashMap<String, String> {
    let mut hints = std::collections::HashMap::new();
    let mut current_section = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed.trim_matches(|c| c == '[' || c == ']').to_string();
            continue;
        }
        if let Some((key, _val, hint_parts)) = parse_ini_line(trimmed) {
            // Filtre des hints purement techniques qui n'apportent rien à l'UI
            let meaningful: Vec<&str> = hint_parts
                .iter()
                .map(|h| h.trim())
                .filter(|h| {
                    if h.is_empty() { return false; }
                    let hl = h.to_lowercase();
                    hl != "1 or 0" && hl != "hidden" && hl != "new" && hl != "updated" && hl != "mode" && hl != "active"
                })
                .collect();
            if !meaningful.is_empty() {
                hints.insert(
                    format!("{}.{}", current_section, key),
                    meaningful.join(" • "),
                );
            }
        }
    }
    hints
}

#[tauri::command]
fn get_ini_hints(file_path: String) -> Result<std::collections::HashMap<String, String>, String> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Ok(std::collections::HashMap::new());
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    Ok(extract_hints_from_content(&content))
}

#[tauri::command]
fn get_ini_hints_from_documents(
    documents_path: String,
    filename: String,
) -> Result<std::collections::HashMap<String, String>, String> {
    let base_path = Path::new(&documents_path);
    let paths_to_try = vec![
        base_path.join(&filename),
        base_path.join("cfg").join(&filename),
        base_path.join("Assetto Corsa").join("cfg").join(&filename),
    ];
    for file_path in paths_to_try {
        if file_path.exists() {
            let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
            return Ok(extract_hints_from_content(&content));
        }
    }
    Ok(std::collections::HashMap::new())
}

#[tauri::command]
fn set_ac_language(documents_path: String, language_code: String) -> Result<(), String> {
    let base_path = Path::new(&documents_path);
    
    // Résolution robuste du dossier cfg
    let mut cfg_dir = base_path.join("cfg");
    if !cfg_dir.exists() {
        cfg_dir = base_path.join("Assetto Corsa").join("cfg");
    }
    
    let file_path = cfg_dir.join("launcher.ini");
    
    if !file_path.exists() {
        return Err("launcher.ini introuvable dans le dossier AC.".into());
    }

    let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let eol = if content.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // ⭐ Sécurité : Assetto Corsa attend des minuscules (ex: fr, en, it)
    let clean_lang = language_code.to_lowercase();

    let mut in_window = false;
    let mut found = false;
    
    for line in lines.iter_mut() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_window = trimmed.eq_ignore_ascii_case("[WINDOW]");
        } else if in_window && trimmed.to_uppercase().starts_with("LANGUAGE=") {
            *line = format!("LANGUAGE={}", clean_lang);
            found = true;
            break;
        }
    }

    if !found {
        // Si la clé n'existe pas, on l'ajoute à la fin de [WINDOW] ou à la fin du fichier
        let mut insert_idx = lines.len();
        for (i, line) in lines.iter().enumerate() {
            if line.trim().eq_ignore_ascii_case("[WINDOW]") {
                insert_idx = i + 1;
                break;
            }
        }
        lines.insert(insert_idx, format!("LANGUAGE={}", clean_lang));
    }

    let new_content = lines.join(eol) + eol;
    
    // Backup
    let backup_path = file_path.with_extension("ini.bak");
    let _ = fs::copy(&file_path, backup_path);

    fs::write(&file_path, new_content).map_err(|e| format!("Erreur d'écriture launcher.ini: {}", e))?;
    Ok(())
}

#[tauri::command]
fn launch_ac_only() -> Result<(), String> {
    // ⭐ On utilise le protocole Steam pour lancer le jeu.
    // 244210 est l'AppID d'Assetto Corsa.
    let steam_url = "steam://rungameid/244210";

    #[cfg(target_os = "windows")]
    {
        // explorer gère très bien les protocoles URL et évite les soucis de parsing de cmd
        std::process::Command::new("explorer")
            .arg(steam_url)
            .spawn()
            .map_err(|e| format!("Erreur lors du lancement de Steam: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(steam_url)
            .spawn()
            .map_err(|e| format!("Erreur lors du lancement de Steam: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(steam_url)
            .spawn()
            .map_err(|e| format!("Erreur lors du lancement de Steam: {}", e))?;
    }

    Ok(())
}

#[derive(Serialize, Clone)]
struct PythonApp {
    name: String,
    active: bool,
}

#[tauri::command]
fn list_python_apps(ac_path: String, documents_path: String) -> Result<Vec<PythonApp>, String> {
    let apps_dir = Path::new(&ac_path).join("apps").join("python");
    if !apps_dir.exists() {
        return Ok(Vec::new());
    }

    let mut apps = Vec::new();

    // 1. Scanner les dossiers d'applications
    if let Ok(entries) = fs::read_dir(apps_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let app_name = entry.file_name().to_string_lossy().to_string();
                apps.push(PythonApp { name: app_name, active: false });
            }
        }
    }

    // 2. Lire python.ini pour voir lesquelles sont actives
    let base = Path::new(&documents_path);
    let mut ini_path = base.join("Assetto Corsa").join("cfg").join("python.ini");
    if !ini_path.exists() {
        ini_path = base.join("cfg").join("python.ini");
    }

    if ini_path.exists() {
        if let Ok(content) = fs::read_to_string(&ini_path) {
            let mut current_section = String::new();
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    current_section = trimmed.trim_matches(|c| c == '[' || c == ']').to_string();
                } else if current_section.to_lowercase() == "active" {
                    // Ignore
                } else if trimmed.to_uppercase().starts_with("ACTIVE=") {
                    let is_active = trimmed.split('=').nth(1).unwrap_or("0").trim() == "1";
                    if let Some(app) = apps.iter_mut().find(|a| a.name.eq_ignore_ascii_case(&current_section)) {
                        app.active = is_active;
                    }
                }
            }
        }
    }

    apps.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(apps)
}

#[tauri::command]
fn toggle_python_app(documents_path: String, app_name: String, enabled: bool) -> Result<(), String> {
    let base = Path::new(&documents_path);
    let mut ini_path = base.join("Assetto Corsa").join("cfg").join("python.ini");
    if !ini_path.exists() {
        ini_path = base.join("cfg").join("python.ini");
    }

    if !ini_path.exists() {
        return Err("python.ini introuvable".into());
    }

    let content = fs::read_to_string(&ini_path).map_err(|e| e.to_string())?;
    let eol = if content.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    let header = format!("[{}]", app_name.to_uppercase());
    let mut found_section = false;
    let mut active_idx = None;
    let mut header_idx = None;

    // 1. Chercher la section et la ligne ACTIVE
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if found_section && active_idx.is_none() {
                // On est sorti de la section sans trouver ACTIVE
                break;
            }
            if trimmed.eq_ignore_ascii_case(&header) {
                found_section = true;
                header_idx = Some(i);
            }
        } else if found_section && trimmed.to_uppercase().starts_with("ACTIVE=") {
            active_idx = Some(i);
            break;
        }
    }

    // 2. Appliquer la modification
    let new_val = if enabled { "ACTIVE=1" } else { "ACTIVE=0" };
    if found_section {
        if let Some(idx) = active_idx {
            lines[idx] = new_val.to_string();
        } else if let Some(idx) = header_idx {
            lines.insert(idx + 1, new_val.to_string());
        }
    } else {
        // Si la section n'existe pas du tout, on l'ajoute à la fin
        if !lines.is_empty() && !lines.last().unwrap().trim().is_empty() {
            lines.push(String::new());
        }
        lines.push(header);
        lines.push(new_val.to_string());
    }

    let final_content = lines.join(eol) + eol;
    let _ = fs::copy(&ini_path, ini_path.with_extension("ini.bak"));
    fs::write(&ini_path, final_content).map_err(|e| e.to_string())?;

    Ok(())
}

// ⭐ Ouvre les paramètres Windows pour l'association des applications
#[tauri::command]
fn open_windows_default_apps() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "ms-settings:defaultapps"])
            .spawn()
            .map_err(|e| format!("Erreur lors de l'ouverture des paramètres : {}", e))?;
    }
    Ok(())
}

// ⭐ Gestion du protocole acmanager://
fn get_acmanager_registry_cmd() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("reg")
            .args(&["query", "HKCU\\Software\\Classes\\acmanager\\shell\\open\\command", "/ve"])
            .output()
            .map_err(|e| e.to_string())?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        // On cherche la ligne qui contient REG_SZ
        for line in stdout.lines() {
            if line.contains("REG_SZ") {
                if let Some(val) = line.split("REG_SZ").nth(1) {
                    return Ok(val.trim().to_string());
                }
            }
        }
    }
    Ok(String::new())
}

fn get_backup_path() -> std::path::PathBuf {
    let mut path = dirs::data_dir().unwrap_or_default();
    path.push("QARL");
    let _ = fs::create_dir_all(&path);
    path.join("protocol_backup.json")
}

#[tauri::command]
fn get_protocol_status() -> Result<bool, String> {
    let cmd = get_acmanager_registry_cmd()?;
    let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_name = exe_path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
    
    // On vérifie si la commande pointe vers notre exécutable actuel
    Ok(cmd.to_lowercase().contains(&exe_name))
}

#[tauri::command]
fn associate_protocol() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let current_cmd = get_acmanager_registry_cmd()?;
        
        // 1. Backup de l'ancienne valeur si elle existe et n'est pas QARL
        if !current_cmd.is_empty() && !current_cmd.to_lowercase().contains("qarl.exe") {
            let backup_path = get_backup_path();
            if !backup_path.exists() {
                let _ = fs::write(&backup_path, serde_json::json!({ "old_cmd": current_cmd }).to_string());
            }
        }

        // 2. Obtention du chemin de l'exécutable actuel (qarl.exe)
        let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
        let new_cmd = format!("\"{}\" \"%1\"", exe_path.to_string_lossy());

        // ⭐ 3. Création de la clé parente (indispensable pour Windows)
        let _ = std::process::Command::new("reg")
            .args(&["add", "HKCU\\Software\\Classes\\acmanager", "/ve", "/t", "REG_SZ", "/d", "URL:QARL Protocol", "/f"])
            .output();

        // ⭐ 4. Ajout de la valeur "URL Protocol" (vide, mais obligatoire)
        let _ = std::process::Command::new("reg")
            .args(&["add", "HKCU\\Software\\Classes\\acmanager", "/v", "URL Protocol", "/t", "REG_SZ", "/d", "", "/f"])
            .output();

        // 5. Écriture de la commande
        let output = std::process::Command::new("reg")
            .args(&[
                "add", "HKCU\\Software\\Classes\\acmanager\\shell\\open\\command",
                "/ve", "/t", "REG_SZ", "/d", &new_cmd, "/f"
            ])
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
    }
    Ok(())
}

#[tauri::command]
fn dissociate_protocol() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let backup_path = get_backup_path();
        
        // 1. Si on a un backup, on le restaure
        if backup_path.exists() {
            if let Ok(content) = fs::read_to_string(&backup_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(old_cmd) = json["old_cmd"].as_str() {
                        let _ = std::process::Command::new("reg")
                            .args(&[
                                "add", "HKCU\\Software\\Classes\\acmanager\\shell\\open\\command",
                                "/ve", "/t", "REG_SZ", "/d", old_cmd, "/f"
                            ])
                            .output();
                    }
                }
            }
            let _ = fs::remove_file(backup_path);
        } else {
            // 2. Si pas de backup (QARL était le seul), on supprime la clé
            let _ = std::process::Command::new("reg")
                .args(&["delete", "HKCU\\Software\\Classes\\acmanager", "/f"])
                .output();
        }
    }
    Ok(())
}

#[tauri::command]
async fn download_csp_configs(
    _app_handle: tauri::AppHandle,
    ac_path: String,
    on_event: tauri::ipc::Channel<serde_json::Value>, // ⭐ Nouveau canal
) -> Result<String, String> {
    let url = "https://github.com/ac-custom-shaders-patch/acc-extension-config/archive/refs/heads/master.zip";
    
    // 1. Création d'un dossier temporaire
    let temp_dir = std::env::temp_dir().join("qarl_csp_config_temp");
    if temp_dir.exists() { let _ = std::fs::remove_dir_all(&temp_dir); }
    std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;
    let zip_path = temp_dir.join("csp_config.zip");

    // 2. Client HTTP avec timeout de 10 minutes, HTTP/1.1 forcé et User-Agent complet
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .http1_only() // ⭐ Force HTTP/1.1 pour éviter le 502 de codeload
        .connect_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(600)) // 10 minutes
        .build()
        .map_err(|e| e.to_string())?;

    println!("⏳ RUST : Début du téléchargement de l'archive CSP...");
    let _ = on_event.send(serde_json::json!({ "type": "info", "msg": "Téléchargement de l'archive (400 Mo)..." }));
    let resp = client.get(url)
        .header("Accept", "application/zip, application/octet-stream, */*")
        .send().await.map_err(|e| format!("Erreur réseau (timeout ?): {}", e))?;
    
    if !resp.status().is_success() {
        return Err(format!("Erreur HTTP: {}", resp.status()));
    }
    
    let bytes = resp.bytes().await.map_err(|e| format!("Erreur lecture: {}", e))?;
    println!("✅ RUST : Téléchargement terminé ({} Mo).", bytes.len() / 1024 / 1024);
    
    let mut file = std::fs::File::create(&zip_path).map_err(|e| e.to_string())?;
    use std::io::Write;
    file.write_all(&bytes).map_err(|e| e.to_string())?;

    // 3. Extraction
    println!("⏳ RUST : Extraction des fichiers...");
    let _ = on_event.send(serde_json::json!({ "type": "info", "msg": "Extraction des fichiers..." }));
    let ac_extension_dir = std::path::Path::new(&ac_path).join("extension");
    let mut copied = 0;

    let mut archive = unarc_rs::unified::ArchiveFormat::open_path(&zip_path)
        .map_err(|e| format!("Erreur ouverture ZIP: {}", e))?;

    while let Some(entry) = archive.next_entry().map_err(|e| e.to_string())? {
        let entry_name = entry.name().replace("\\", "/");
        
        let relative_path = if let Some(p) = entry_name.strip_prefix("acc-extension-config-master/config/cars/") {
            format!("config/cars/{}", p)
        } else if let Some(p) = entry_name.strip_prefix("acc-extension-config-master/config/tracks/") {
            format!("config/tracks/{}", p)
        } else if let Some(p) = entry_name.strip_prefix("acc-extension-config-master/backgrounds/") {
            format!("backgrounds/{}", p)
        } else {
            continue;
        };

        if relative_path.contains("..") { continue; }

        let out_path = ac_extension_dir.join(&relative_path);

        if entry.is_directory() {
            let _ = std::fs::create_dir_all(&out_path);
        } else {
            if let Some(parent) = out_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(mut out_file) = std::fs::File::create(&out_path) {
                let _ = archive.read_to(&entry, &mut out_file);
                copied += 1;

                // ⭐ NOUVEAU : Envoi de la progression toutes les 100 fichiers
                if copied % 100 == 0 {
                    let _ = on_event.send(serde_json::json!({
                        "type": "progress",
                        "msg": format!("{} fichiers extraits...", copied)
                    }));
                    tokio::task::yield_now().await;
                }
            }
        }
    }

    // 4. Nettoyage
    let _ = std::fs::remove_dir_all(&temp_dir);
    println!("✅ RUST : {} fichiers copiés dans extension/config/", copied);

    if copied == 0 {
        return Err("Aucun fichier de configuration trouvé dans l'archive.".into());
    }

    Ok(format!("✅ {} fichiers de configuration CSP installés !", copied))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(WatcherState::default())
        // ⭐ LOT 3 : Le Watchdog des périphériques
        .setup(|app| {
            // ⭐ On vérifie les arguments au démarrage (si QARL était fermé)
            let args: Vec<String> = std::env::args().collect();
            let initial_url = args.iter().find(|arg| arg.starts_with("acmanager://") || arg.starts_with("qarl://"));
            
            if let Some(url) = initial_url {
                let url = url.clone();
                let handle = app.handle().clone();
                // On attend un peu que le frontend soit prêt
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                    let _ = handle.emit("qarl-deep-link", url);
                });
            }

            // ⭐ Écoute des URLs entrantes (Deep Link)
            // L'événement est déclenché par le plugin single-instance (via la feature "deep-link")
            let deep_link_handle = app.handle().clone();
            app.listen("deep-link://new-url", move |event| {
                // Le payload contient l'URL sous forme de chaîne JSON
                let payload = event.payload();
                // On nettoie les guillemets éventuels autour de l'URL
                let clean_url = payload.trim_matches('"').to_string();
                
                println!("🔗 Deep Link reçu : {}", clean_url);
                
                // On émet un événement personnalisé pour le frontend Svelte
                let _ = deep_link_handle.emit("qarl-deep-link", clean_url);
            });

            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut last_iguids = Vec::new();
                
                // Snapshot initial
                if let Ok(devs) = get_directinput_devices() {
                    last_iguids = devs.iter().map(|d| d.iguid.clone()).collect();
                }
                
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    
                    if let Ok(devs) = get_directinput_devices() {
                        let current_iguids: Vec<String> = devs.iter().map(|d| d.iguid.clone()).collect();
                        
                        // On compare les deux listes (l'ordre a peu d'importance, mais l'ajout/suppression oui)
                        if current_iguids != last_iguids {
                            last_iguids = current_iguids;
                            let _ = handle.emit("controllers-changed", ());
                        }
                    }
                }
            });
            Ok(())
        })
        // ⭐ Le plugin single-instance DOIT être le premier enregistré
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            use tauri::Manager;
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
            
            // ⭐ On cherche l'URL dans les arguments et on l'envoie au frontend
            for arg in argv {
                if arg.starts_with("acmanager://") || arg.starts_with("qarl://") {
                    let _ = app.emit("qarl-deep-link", arg);
                    break;
                }
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_window_state::Builder::default().build()) // ⭐ Sauvegarde de la fenêtre
                .invoke_handler(tauri::generate_handler![
            get_ac_cars, 
            get_ac_tracks,
            get_ac_weathers,
            get_pure_plans,
            load_pure_plan,
            read_pure_ctrl,
            write_pure_ctrl,
            launch_race,
            launch_ac_only, // ⭐ Ajouté
            set_ac_language, // ⭐ NOUVEAU
            check_ac_capabilities,
            save_custom_pure_plan,
            read_race_history,
            clear_race_history,
            delete_history_entry,
            delete_pure_plan,
            install_mod_from_archive,
            get_mods_history,
            open_folder,
            get_online_servers,
            get_server_details,
            join_server,
            detect_steam_id,
            list_ac_controller_files,
            check_ac_initialized,
            detect_ac_path,
            detect_documents_ac_path,
            list_controller_presets,
            detect_setup_status,
            start_content_watcher,
            read_controls_ini,
            read_specific_ini,
            write_controls_ini,
            get_controller_profiles,
            get_controller_mapping,
            get_directinput_devices,
            get_directinput_states,
            parse_controllers_section,
            ensure_controller_registered,
            list_ppfilters,
            read_ini_file,
            write_ini_file,
            list_audio_devices,
            read_onboard_fov,
            write_onboard_fov,
            list_csp_templates,
            read_csp_module_data,
            set_csp_module_enabled,
            write_csp_module,
            update_ini_values,
            set_weather_fx_implementation,
            list_assists_presets,
            save_assists_preset,
            load_assists_preset,
            delete_assists_preset,
            list_csp_presets,
            save_csp_preset,
            load_csp_preset,
            reset_csp_to_defaults,
            import_csp_preset_from_path,
            list_cm_presets,
            reset_single_csp_module,
            list_schedule_presets,
            save_schedule_preset,
            load_schedule_preset,
            delete_schedule_preset,
            install_csp_archive,
            list_video_presets,
            save_video_preset,
            load_video_preset,
            delete_video_preset,
            cleanup_injected_track_configs,
            get_media_files,
            get_replay_info,
            launch_replay,
            delete_media_file,
            write_specific_ini,
            get_ini_hints,
            get_ini_hints_from_documents,
            list_python_apps,
            toggle_python_app,
            open_windows_default_apps,
            get_protocol_status,      // ⭐ Ajouté
            associate_protocol,       // ⭐ Ajouté
            dissociate_protocol,      // ⭐ Ajouté
            download_csp_configs      // ⭐ NOUVEAU
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                // ⭐ Nettoyage des fichiers ext_config.ini injectés à la fermeture de QARL
                cleanup_injected_track_configs();
            }
        });
}