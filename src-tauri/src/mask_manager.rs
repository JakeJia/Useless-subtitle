use crate::{
    geometry,
    models::{MaskConfig, MaskViewState, MIN_HEIGHT, MIN_WIDTH},
    state::AppState,
    tray,
};
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Position, Size, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder, WindowEvent,
};

pub const MASK_LABEL_PREFIX: &str = "mask-";
pub const SETTINGS_LABEL: &str = "mask-settings";

pub fn restore_or_initialize(app: &AppHandle, state: &AppState) -> Result<(), String> {
    restore_or_initialize_with(
        state,
        || create_mask(app, state, None).map(|_| ()),
        |mask| build_mask_window(app, state, mask).map(|_| ()),
    )
}

fn restore_or_initialize_with(
    state: &AppState,
    create_default: impl FnOnce() -> Result<(), String>,
    mut restore_mask: impl FnMut(&MaskConfig) -> Result<(), String>,
) -> Result<(), String> {
    let config = state.config();
    if config.masks.is_empty() {
        create_default()?;
    } else {
        for mask in config.masks {
            if let Err(error) = restore_mask(&mask) {
                eprintln!("failed to restore {}: {error}", mask.name);
            }
        }
    }
    state.mark_initialized();
    Ok(())
}

pub fn create_mask(
    app: &AppHandle,
    state: &AppState,
    source: Option<&MaskConfig>,
) -> Result<String, String> {
    let config = state.config();
    let geometry = match source {
        Some(mask) => geometry::cascaded_geometry(app, &mask.geometry, config.masks.len())?,
        None => geometry::default_geometry(app, config.masks.len())?,
    };
    let mut mask = MaskConfig::new(config.next_name(), geometry);
    if let Some(source) = source {
        mask.appearance = source.appearance.clone();
    }
    let id = mask.id.clone();

    state.add_mask(mask.clone());
    if let Err(error) = build_mask_window(app, state, &mask) {
        state.remove_mask(&id);
        return Err(error);
    }
    tray::refresh(app, state).map_err(|error| error.to_string())?;
    Ok(id)
}

fn build_mask_window(
    app: &AppHandle,
    state: &AppState,
    mask: &MaskConfig,
) -> Result<WebviewWindow, String> {
    let label = mask.label();
    if app.get_webview_window(&label).is_some() {
        return Err(format!("window label already exists: {label}"));
    }
    let resolved = geometry::resolve_geometry(app, &mask.geometry)?;
    let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
        .title(&mask.name)
        .inner_size(mask.geometry.width_logical, mask.geometry.height_logical)
        .min_inner_size(MIN_WIDTH, MIN_HEIGHT)
        .decorations(false)
        // Native shadows add a 1px border on undecorated Windows windows.
        .shadow(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(true)
        .visible(false)
        .build()
        .map_err(|error| error.to_string())?;

    window
        .set_position(Position::Physical(resolved.position))
        .map_err(|error| error.to_string())?;
    window
        .set_size(Size::Physical(resolved.size))
        .map_err(|error| error.to_string())?;
    #[cfg(target_os = "macos")]
    window
        .set_visible_on_all_workspaces(true)
        .map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;

    state.register_mask(&mask.id);
    register_geometry_listener(window.clone(), state.clone(), mask.id.clone());
    Ok(window)
}

fn register_geometry_listener(window: WebviewWindow, state: AppState, id: String) {
    let observed_window = window.clone();
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Moved(_) | WindowEvent::Resized(_)) {
            match geometry::geometry_from_window(&observed_window) {
                Ok(geometry) => {
                    if let Err(error) = state.update_geometry(&id, geometry) {
                        eprintln!("failed to update geometry for {id}: {error}");
                    }
                }
                Err(error) => eprintln!("failed to read geometry for {id}: {error}"),
            }
        }
    });
}

pub fn id_from_label(label: &str) -> Result<&str, String> {
    label
        .strip_prefix(MASK_LABEL_PREFIX)
        .filter(|id| !id.is_empty())
        .ok_or_else(|| format!("window is not a mask: {label}"))
}

pub fn view_for_window(window: &WebviewWindow, state: &AppState) -> Result<MaskViewState, String> {
    let id = id_from_label(window.label())?;
    state.view(id).ok_or_else(|| format!("unknown mask {id}"))
}

pub fn emit_view(app: &AppHandle, state: &AppState, id: &str) -> Result<(), String> {
    let view = state.view(id).ok_or_else(|| format!("unknown mask {id}"))?;
    let label = format!("{MASK_LABEL_PREFIX}{id}");
    app.emit_to(&label, "mask-state-changed", &view)
        .map_err(|error| error.to_string())?;
    if state.settings_target().as_deref() == Some(id) {
        app.emit_to(SETTINGS_LABEL, "settings-target-changed", &view)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn lock_mask(app: &AppHandle, state: &AppState, id: &str, locked: bool) -> Result<(), String> {
    if locked && !state.tray_ready() {
        return Err("the system tray is unavailable; locking is disabled".to_string());
    }
    let window = mask_window(app, id)?;
    window
        .set_ignore_cursor_events(locked)
        .map_err(|error| error.to_string())?;
    state.set_locked(id, locked)?;
    emit_view(app, state, id)?;
    tray::refresh(app, state).map_err(|error| error.to_string())?;
    Ok(())
}

pub fn delete_mask(app: &AppHandle, state: &AppState, id: &str) -> Result<(), String> {
    let window = mask_window(app, id).ok();
    state.remove_mask(id);
    if state.settings_target().as_deref() == Some(id) {
        state.set_settings_target(None);
        if let Some(settings) = app.get_webview_window(SETTINGS_LABEL) {
            let _ = settings.hide();
        }
    }
    if let Some(window) = window {
        window.close().map_err(|error| error.to_string())?;
    }
    tray::refresh(app, state).map_err(|error| error.to_string())?;
    Ok(())
}

pub fn duplicate_mask(app: &AppHandle, state: &AppState, id: &str) -> Result<String, String> {
    let source = state.mask(id).ok_or_else(|| format!("unknown mask {id}"))?;
    create_mask(app, state, Some(&source))
}

pub fn set_mask_visible(
    app: &AppHandle,
    state: &AppState,
    id: &str,
    visible: bool,
) -> Result<(), String> {
    let window = mask_window(app, id)?;
    if visible {
        window.show().map_err(|error| error.to_string())?;
    } else {
        window.hide().map_err(|error| error.to_string())?;
    }
    state.set_visible(id, visible)?;
    emit_view(app, state, id)?;
    Ok(())
}

pub fn show_and_edit(app: &AppHandle, state: &AppState, id: &str) -> Result<(), String> {
    let window = mask_window(app, id)?;
    window
        .set_ignore_cursor_events(false)
        .map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;
    state.set_locked(id, false)?;
    state.set_visible(id, true)?;
    emit_view(app, state, id)?;
    tray::refresh(app, state).map_err(|error| error.to_string())?;
    Ok(())
}

pub fn set_all_visible(app: &AppHandle, state: &AppState, visible: bool) -> Result<(), String> {
    for mask in state.config().masks {
        if let Err(error) = set_mask_visible(app, state, &mask.id, visible) {
            eprintln!("failed to change visibility for {}: {error}", mask.id);
        }
    }
    tray::refresh(app, state).map_err(|error| error.to_string())?;
    Ok(())
}

pub fn unlock_all(app: &AppHandle, state: &AppState) -> Result<(), String> {
    for mask in state.config().masks {
        if state
            .runtime(&mask.id)
            .is_some_and(|runtime| runtime.locked)
        {
            if let Err(error) = lock_mask(app, state, &mask.id, false) {
                eprintln!("failed to unlock {}: {error}", mask.id);
            }
        }
    }
    tray::refresh(app, state).map_err(|error| error.to_string())?;
    Ok(())
}

pub fn reset_geometry(app: &AppHandle, state: &AppState, id: &str) -> Result<(), String> {
    let index = state
        .config()
        .masks
        .iter()
        .position(|mask| mask.id == id)
        .ok_or_else(|| format!("unknown mask {id}"))?;
    let geometry = geometry::default_geometry(app, index)?;
    let window = mask_window(app, id)?;
    geometry::recover_window_geometry(&window, &geometry)?;
    state.update_geometry(id, geometry)?;
    Ok(())
}

pub fn recover_all(app: &AppHandle, state: &AppState) -> Result<(), String> {
    for mask in state.config().masks {
        let window = match mask_window(app, &mask.id) {
            Ok(window) => window,
            Err(error) => {
                eprintln!("cannot recover {}: {error}", mask.id);
                continue;
            }
        };
        if let Ok(resolved) = geometry::resolve_geometry(app, &mask.geometry) {
            let _ = window.set_position(Position::Physical(resolved.position));
            let _ = window.set_size(Size::Physical(resolved.size));
            if let Ok(updated) = geometry::geometry_from_window(&window) {
                let _ = state.update_geometry(&mask.id, updated);
            }
        }
    }
    Ok(())
}

pub fn open_settings(app: &AppHandle, state: &AppState, id: &str) -> Result<(), String> {
    let view = state.view(id).ok_or_else(|| format!("unknown mask {id}"))?;
    state.set_settings_target(Some(id.to_string()));

    if let Some(window) = app.get_webview_window(SETTINGS_LABEL) {
        window.show().map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())?;
        app.emit_to(SETTINGS_LABEL, "settings-target-changed", view)
            .map_err(|error| error.to_string())?;
        return Ok(());
    }

    WebviewWindowBuilder::new(
        app,
        SETTINGS_LABEL,
        WebviewUrl::App("index.html?view=settings".into()),
    )
    .title("Mask Appearance")
    .inner_size(360.0, 250.0)
    .min_inner_size(320.0, 220.0)
    .resizable(false)
    .always_on_top(true)
    .build()
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn mask_window(app: &AppHandle, id: &str) -> Result<WebviewWindow, String> {
    app.get_webview_window(&format!("{MASK_LABEL_PREFIX}{id}"))
        .ok_or_else(|| format!("mask window does not exist: {id}"))
}

#[allow(dead_code)]
fn _physical_types_are_supported(position: PhysicalPosition<i32>, size: PhysicalSize<u32>) {
    let _ = (position, size);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{AppConfig, MaskGeometry},
        persistence,
    };
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    fn directory() -> PathBuf {
        std::env::temp_dir().join(format!("mask-startup-{}", uuid::Uuid::new_v4()))
    }

    fn load_state(directory: &Path) -> AppState {
        AppState::from_loaded(persistence::load_from_directory(directory).unwrap())
    }

    fn create_default(state: &AppState) -> Result<(), String> {
        state.add_mask(MaskConfig::new(
            state.config().next_name(),
            MaskGeometry::default(),
        ));
        Ok(())
    }

    #[test]
    fn empty_startup_persists_one_mask_independent_of_initialization() {
        for initialized in [false, true] {
            let directory = directory();
            let path = directory.join("mask-state.json");
            if initialized {
                persistence::save(
                    &path,
                    &AppConfig {
                        initialized,
                        ..AppConfig::default()
                    },
                )
                .unwrap();
            }
            let state = load_state(&directory);
            restore_or_initialize_with(
                &state,
                || create_default(&state),
                |_| panic!("empty startup must create a mask"),
            )
            .unwrap();
            state.flush().unwrap();
            let config = state.config();
            assert!(config.initialized);
            assert_eq!(config.masks.len(), 1);
            drop(state);

            let restarted = load_state(&directory);
            let mut restored = Vec::new();
            restore_or_initialize_with(
                &restarted,
                || panic!("saved mask must not be duplicated"),
                |mask| {
                    restored.push(mask.id.clone());
                    Ok(())
                },
            )
            .unwrap();
            restarted.flush().unwrap();
            assert_eq!(restored, vec![config.masks[0].id.clone()]);
            assert_eq!(restarted.config(), config);
            drop(restarted);
            fs::remove_dir_all(directory).unwrap();
        }
    }

    #[test]
    fn deleting_the_last_mask_remains_empty_until_next_startup() {
        let directory = directory();
        let state = load_state(&directory);
        restore_or_initialize_with(&state, || create_default(&state), |_| unreachable!()).unwrap();
        let old_id = state.config().masks[0].id.clone();
        assert!(state.remove_mask(&old_id));
        state.flush().unwrap();
        assert!(state.config().masks.is_empty());
        assert!(persistence::load_from_directory(&directory)
            .unwrap()
            .config
            .masks
            .is_empty());
        drop(state);

        let restarted = load_state(&directory);
        restore_or_initialize_with(
            &restarted,
            || create_default(&restarted),
            |_| unreachable!(),
        )
        .unwrap();
        restarted.flush().unwrap();
        let saved = persistence::load_from_directory(&directory).unwrap().config;
        assert_eq!(saved.masks.len(), 1);
        assert_ne!(saved.masks[0].id, old_id);
        drop(restarted);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn nonempty_startup_preserves_masks_and_isolates_restore_failures() {
        for initialized in [false, true] {
            let directory = directory();
            let mut config = AppConfig {
                initialized,
                masks: vec![
                    MaskConfig::new("First".into(), MaskGeometry::default()),
                    MaskConfig::new("Second".into(), MaskGeometry::default()),
                ],
                ..AppConfig::default()
            };
            config.masks[0].appearance.opacity = 90;
            config.masks[1].appearance.color = "#112233".into();
            config.masks[1].appearance.opacity = 40;
            persistence::save(&directory.join("mask-state.json"), &config).unwrap();
            let state = load_state(&directory);
            let mut restored = Vec::new();
            restore_or_initialize_with(
                &state,
                || panic!("nonempty startup must not create a default"),
                |mask| {
                    restored.push(mask.id.clone());
                    if mask.id == config.masks[0].id {
                        Err("window creation failed".into())
                    } else {
                        state.register_mask(&mask.id);
                        Ok(())
                    }
                },
            )
            .unwrap();
            state.flush().unwrap();
            assert_eq!(
                restored,
                config
                    .masks
                    .iter()
                    .map(|mask| mask.id.clone())
                    .collect::<Vec<_>>()
            );
            assert_eq!(state.config().masks, config.masks);
            assert_eq!(
                persistence::load_from_directory(&directory)
                    .unwrap()
                    .config
                    .masks,
                config.masks
            );
            assert!(state.config().initialized);
            assert!(state
                .view(&config.masks[1].id)
                .is_some_and(|view| view.visible && !view.locked));
            drop(state);
            fs::remove_dir_all(directory).unwrap();
        }
    }
}
