use crate::models::{AppConfig, MaskConfig, MaskGeometry, CONFIG_SCHEMA_VERSION};
use serde_json::Value;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager};

const CONFIG_FILE_NAME: &str = "mask-state.json";
const LEGACY_FILE_NAME: &str = "store.json";

pub struct LoadedConfig {
    pub config: AppConfig,
    pub path: PathBuf,
    pub needs_save: bool,
}

pub fn load(app: &AppHandle) -> Result<LoadedConfig, String> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    let path = directory.join(CONFIG_FILE_NAME);
    let backup_path = backup_path(&path);

    if path.exists() {
        match read_config(&path) {
            Ok(config) => {
                return Ok(LoadedConfig {
                    config: config.validate(),
                    path,
                    needs_save: false,
                });
            }
            Err(primary_error) => {
                if backup_path.exists() {
                    if let Ok(config) = read_config(&backup_path) {
                        preserve_corrupt_file(&path);
                        return Ok(LoadedConfig {
                            config: config.validate(),
                            path,
                            needs_save: true,
                        });
                    }
                }
                preserve_corrupt_file(&path);
                eprintln!("configuration was invalid and has been isolated: {primary_error}");
                return Ok(LoadedConfig {
                    config: AppConfig::default(),
                    path,
                    needs_save: true,
                });
            }
        }
    }

    let legacy_path = directory.join(LEGACY_FILE_NAME);
    if legacy_path.exists() {
        match migrate_legacy(&legacy_path) {
            Ok(config) => {
                return Ok(LoadedConfig {
                    config,
                    path,
                    needs_save: true,
                })
            }
            Err(error) => eprintln!("legacy configuration could not be migrated: {error}"),
        }
    }

    Ok(LoadedConfig {
        config: AppConfig::default(),
        path,
        needs_save: true,
    })
}

pub fn save(path: &Path, config: &AppConfig) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "configuration path has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;

    let bytes = serde_json::to_vec_pretty(config).map_err(|error| error.to_string())?;
    let temporary_path = temporary_path(path);
    let backup_path = backup_path(path);

    {
        let mut file = fs::File::create(&temporary_path).map_err(|error| error.to_string())?;
        file.write_all(&bytes).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
    }

    if path.exists() {
        if backup_path.exists() {
            fs::remove_file(&backup_path).map_err(|error| error.to_string())?;
        }
        fs::rename(path, &backup_path).map_err(|error| error.to_string())?;
    }

    if let Err(error) = fs::rename(&temporary_path, path) {
        if backup_path.exists() && !path.exists() {
            let _ = fs::rename(&backup_path, path);
        }
        return Err(error.to_string());
    }

    Ok(())
}

fn read_config(path: &Path) -> Result<AppConfig, String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    let config: AppConfig = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if config.schema_version > CONFIG_SCHEMA_VERSION {
        return Err(format!(
            "configuration schema {} is newer than supported schema {}",
            config.schema_version, CONFIG_SCHEMA_VERSION
        ));
    }
    Ok(config)
}

fn migrate_legacy(path: &Path) -> Result<AppConfig, String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    let root: Value = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    let list = root
        .get("mask_list")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut config = AppConfig {
        schema_version: CONFIG_SCHEMA_VERSION,
        initialized: true,
        masks: Vec::new(),
    };

    for (index, item) in list.into_iter().enumerate() {
        let Some(object) = item.as_object() else {
            continue;
        };
        let geometry = MaskGeometry {
            monitor_key: None,
            offset_x_logical: number(object.get("x"), 100.0),
            offset_y_logical: number(object.get("y"), 100.0),
            width_logical: number(object.get("width"), 600.0),
            height_logical: number(object.get("height"), 120.0),
            saved_scale_factor: 1.0,
        };
        let mut mask = MaskConfig::new(format!("Mask {}", index + 1), geometry);
        if let Some(color) = object.get("color").and_then(Value::as_str) {
            mask.appearance.color = color.to_string();
        }
        mask.appearance.opacity = object
            .get("opacity")
            .and_then(|value| value.as_u64().or_else(|| value.as_str()?.parse().ok()))
            .unwrap_or(90)
            .clamp(10, 100) as u8;
        mask.validate();
        config.masks.push(mask);
    }

    Ok(config.validate())
}

fn number(value: Option<&Value>, default: f64) -> f64 {
    value
        .and_then(|value| value.as_f64().or_else(|| value.as_str()?.parse().ok()))
        .filter(|value| value.is_finite())
        .unwrap_or(default)
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_extension("json.bak")
}

fn temporary_path(path: &Path) -> PathBuf {
    path.with_extension("json.tmp")
}

fn preserve_corrupt_file(path: &Path) {
    if !path.exists() {
        return;
    }
    let corrupt_path = path.with_extension("corrupt.json");
    if corrupt_path.exists() {
        let _ = fs::remove_file(&corrupt_path);
    }
    let _ = fs::rename(path, corrupt_path);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MaskGeometry;

    #[test]
    fn round_trips_atomic_configuration() {
        let directory =
            std::env::temp_dir().join(format!("useless-subtitle-test-{}", uuid::Uuid::new_v4()));
        let path = directory.join(CONFIG_FILE_NAME);
        let mut config = AppConfig {
            initialized: true,
            ..AppConfig::default()
        };
        config
            .masks
            .push(MaskConfig::new("Mask 1".into(), MaskGeometry::default()));

        save(&path, &config).unwrap();
        assert_eq!(read_config(&path).unwrap(), config);

        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn migrates_legacy_mask_list() {
        let directory =
            std::env::temp_dir().join(format!("useless-subtitle-legacy-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join(LEGACY_FILE_NAME);
        fs::write(
            &path,
            r##"{"mask_list":[{"label":"mask_1","color":"#112233","opacity":"80","x":10,"y":20,"width":300,"height":60}]}"##,
        )
        .unwrap();

        let config = migrate_legacy(&path).unwrap();
        assert!(config.initialized);
        assert_eq!(config.masks.len(), 1);
        assert_eq!(config.masks[0].appearance.color, "#112233");
        assert_eq!(config.masks[0].appearance.opacity, 80);

        let _ = fs::remove_dir_all(directory);
    }
}
