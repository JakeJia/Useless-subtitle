use crate::{mask_manager, state::AppState};
use tauri::{
    menu::{ContextMenu, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::TrayIconBuilder,
    AppHandle, Manager, WebviewWindow,
};

const TRAY_ID: &str = "useless-subtitle-tray";

pub fn setup(app: &AppHandle, state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_tray_menu(app, state)?;
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or("application icon is unavailable")?;
    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("Useless Subtitle")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .build(app)?;
    state.set_tray(tray);
    Ok(())
}

pub fn refresh(app: &AppHandle, state: &AppState) -> tauri::Result<()> {
    let Some(tray) = state.tray() else {
        return Ok(());
    };
    tray.set_menu(Some(build_tray_menu(app, state)?))
}

fn build_tray_menu(app: &AppHandle, state: &AppState) -> tauri::Result<Menu<tauri::Wry>> {
    let menu = Menu::new(app)?;
    let config = state.config();
    let any_visible = config.masks.iter().any(|mask| {
        state
            .runtime(&mask.id)
            .is_some_and(|runtime| runtime.visible)
    });
    let any_locked = config.masks.iter().any(|mask| {
        state
            .runtime(&mask.id)
            .is_some_and(|runtime| runtime.locked)
    });

    let new_mask = MenuItem::with_id(app, "new_mask", "New Mask", true, None::<&str>)?;
    let visibility = MenuItem::with_id(
        app,
        if any_visible { "hide_all" } else { "show_all" },
        if any_visible { "Hide All" } else { "Show All" },
        !config.masks.is_empty(),
        None::<&str>,
    )?;
    let unlock_all = MenuItem::with_id(app, "unlock_all", "Unlock All", any_locked, None::<&str>)?;
    menu.append_items(&[&new_mask, &visibility, &unlock_all])?;

    let masks_menu = Submenu::new(app, "Masks", !config.masks.is_empty())?;
    for mask in &config.masks {
        let runtime = state.runtime(&mask.id).unwrap_or_default();
        let status = match (runtime.visible, runtime.locked) {
            (false, _) => "Hidden",
            (true, true) => "Locked",
            (true, false) => "Editing",
        };
        let submenu = Submenu::new(app, format!("{} — {status}", mask.name), true)?;
        let show_edit = mask_item(app, &mask.id, "edit", "Show and Edit", true)?;
        let visibility = mask_item(
            app,
            &mask.id,
            "visibility",
            if runtime.visible { "Hide" } else { "Show" },
            true,
        )?;
        let lock = mask_item(
            app,
            &mask.id,
            "lock",
            if runtime.locked {
                "Unlock"
            } else {
                "Lock and Click Through"
            },
            state.tray_ready(),
        )?;
        let appearance = mask_item(app, &mask.id, "appearance", "Appearance…", true)?;
        let reset = mask_item(app, &mask.id, "reset", "Reset Position and Size", true)?;
        let duplicate = mask_item(app, &mask.id, "duplicate", "Duplicate", true)?;
        let delete = mask_item(app, &mask.id, "delete", "Delete", true)?;
        let separator = PredefinedMenuItem::separator(app)?;
        submenu.append_items(&[
            &show_edit,
            &visibility,
            &lock,
            &separator,
            &appearance,
            &reset,
            &duplicate,
            &delete,
        ])?;
        masks_menu.append(&submenu)?;
    }
    menu.append(&masks_menu)?;

    let separator = PredefinedMenuItem::separator(app)?;
    let recover = MenuItem::with_id(
        app,
        "recover_all",
        "Recover Off-Screen Masks",
        !config.masks.is_empty(),
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    menu.append_items(&[&separator, &recover, &quit])?;
    Ok(menu)
}

fn mask_item(
    app: &AppHandle,
    id: &str,
    action: &str,
    label: &str,
    enabled: bool,
) -> tauri::Result<MenuItem<tauri::Wry>> {
    MenuItem::with_id(
        app,
        format!("mask:{id}:{action}"),
        label,
        enabled,
        None::<&str>,
    )
}

pub fn show_mask_context_menu(
    app: &AppHandle,
    state: &AppState,
    window: &WebviewWindow,
    id: &str,
) -> Result<(), String> {
    let runtime = state
        .runtime(id)
        .ok_or_else(|| format!("unknown mask {id}"))?;
    if runtime.locked {
        return Ok(());
    }

    let menu = Menu::new(app).map_err(|error| error.to_string())?;
    let appearance =
        mask_item(app, id, "appearance", "Appearance…", true).map_err(|error| error.to_string())?;
    let lock = mask_item(
        app,
        id,
        "lock",
        "Lock and Click Through",
        state.tray_ready(),
    )
    .map_err(|error| error.to_string())?;
    let reset = mask_item(app, id, "reset", "Reset Position and Size", true)
        .map_err(|error| error.to_string())?;
    let duplicate =
        mask_item(app, id, "duplicate", "Duplicate", true).map_err(|error| error.to_string())?;
    let separator = PredefinedMenuItem::separator(app).map_err(|error| error.to_string())?;
    let delete =
        mask_item(app, id, "delete", "Delete Mask", true).map_err(|error| error.to_string())?;
    menu.append_items(&[&appearance, &lock, &reset, &duplicate, &separator, &delete])
        .map_err(|error| error.to_string())?;
    menu.popup(window.as_ref().window())
        .map_err(|error| error.to_string())
}

pub fn handle_menu_event(app: &AppHandle, event_id: &str) -> Result<(), String> {
    let state = app.state::<AppState>();
    match event_id {
        "new_mask" => {
            mask_manager::create_mask(app, &state, None)?;
        }
        "show_all" => mask_manager::set_all_visible(app, &state, true)?,
        "hide_all" => mask_manager::set_all_visible(app, &state, false)?,
        "unlock_all" => mask_manager::unlock_all(app, &state)?,
        "recover_all" => mask_manager::recover_all(app, &state)?,
        "quit" => {
            state.flush()?;
            app.exit(0);
        }
        _ => handle_mask_menu_event(app, &state, event_id)?,
    }
    Ok(())
}

fn handle_mask_menu_event(app: &AppHandle, state: &AppState, event_id: &str) -> Result<(), String> {
    let mut parts = event_id.split(':');
    if parts.next() != Some("mask") {
        return Ok(());
    }
    let id = parts
        .next()
        .ok_or_else(|| "mask menu ID has no mask identifier".to_string())?;
    let action = parts
        .next()
        .ok_or_else(|| "mask menu ID has no action".to_string())?;
    if parts.next().is_some() {
        return Err("mask menu ID contains unexpected components".to_string());
    }

    match action {
        "edit" => mask_manager::show_and_edit(app, state, id)?,
        "visibility" => {
            let visible = state
                .runtime(id)
                .map(|runtime| runtime.visible)
                .unwrap_or(false);
            mask_manager::set_mask_visible(app, state, id, !visible)?;
            refresh(app, state).map_err(|error| error.to_string())?;
        }
        "lock" => {
            let locked = state
                .runtime(id)
                .map(|runtime| runtime.locked)
                .unwrap_or(false);
            mask_manager::lock_mask(app, state, id, !locked)?;
        }
        "appearance" => mask_manager::open_settings(app, state, id)?,
        "reset" => mask_manager::reset_geometry(app, state, id)?,
        "duplicate" => {
            mask_manager::duplicate_mask(app, state, id)?;
        }
        "delete" => mask_manager::delete_mask(app, state, id)?,
        _ => return Err(format!("unknown mask menu action: {action}")),
    }
    Ok(())
}
