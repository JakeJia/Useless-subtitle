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
    let config = state.config();
    if !config.initialized && config.masks.is_empty() {
        create_mask(app, state, None)?;
        state.mark_initialized();
        return Ok(());
    }

    for mask in config.masks {
        if let Err(error) = build_mask_window(app, state, &mask) {
            eprintln!("failed to restore {}: {error}", mask.name);
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
