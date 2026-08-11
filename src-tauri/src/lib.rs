mod commands;
mod geometry;
mod mask_manager;
mod models;
mod persistence;
mod state;
mod tray;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.handle()
                .set_activation_policy(tauri::ActivationPolicy::Accessory)?;

            let state = state::AppState::load(app.handle())?;
            app.manage(state.clone());

            tray::setup(app.handle(), &state)?;
            mask_manager::restore_or_initialize(app.handle(), &state)?;
            tray::refresh(app.handle(), &state)?;

            Ok(())
        })
        .on_menu_event(|app, event| {
            if let Err(error) = tray::handle_menu_event(app, event.id().as_ref()) {
                eprintln!("menu action failed: {error}");
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_current_mask,
            commands::get_settings_target,
            commands::update_mask_appearance,
            commands::lock_current_mask,
            commands::delete_current_mask,
            commands::show_current_mask_menu,
            commands::hide_settings_window,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                if let Some(state) = app.try_state::<state::AppState>() {
                    if let Err(error) = state.flush() {
                        eprintln!("failed to flush mask state during exit: {error}");
                    }
                }
            }
        });
}
