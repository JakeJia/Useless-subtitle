use crate::{mask_manager, models::MaskViewState, state::AppState, tray};
use tauri::{AppHandle, State, WebviewWindow};

#[tauri::command]
pub fn get_current_mask(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<MaskViewState, String> {
    mask_manager::view_for_window(&window, &state)
}

#[tauri::command]
pub fn get_settings_target(state: State<'_, AppState>) -> Result<Option<MaskViewState>, String> {
    Ok(state.settings_target().and_then(|id| state.view(&id)))
}

#[tauri::command]
pub fn update_mask_appearance(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    color: String,
    opacity: u8,
) -> Result<MaskViewState, String> {
    state.update_appearance(&id, color, opacity)?;
    mask_manager::emit_view(&app, &state, &id)?;
    state.view(&id).ok_or_else(|| format!("unknown mask {id}"))
}

#[tauri::command]
pub fn lock_current_mask(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let id = mask_manager::id_from_label(window.label())?.to_string();
    mask_manager::lock_mask(&app, &state, &id, true)
}

#[tauri::command]
pub fn delete_current_mask(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let id = mask_manager::id_from_label(window.label())?.to_string();
    mask_manager::delete_mask(&app, &state, &id)
}

#[tauri::command]
pub fn show_current_mask_menu(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let id = mask_manager::id_from_label(window.label())?.to_string();
    tray::show_mask_context_menu(&app, &state, &window, &id)
}

#[tauri::command]
pub fn hide_settings_window(window: WebviewWindow) -> Result<(), String> {
    if window.label() != mask_manager::SETTINGS_LABEL {
        return Err("only the settings window may invoke this command".to_string());
    }
    window.hide().map_err(|error| error.to_string())
}
