use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WebviewWindowBuilder, WebviewUrl,
    Emitter,
};
use std::sync::atomic::{AtomicUsize, Ordering};

static MASK_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let new_i = MenuItem::with_id(app, "new_mask", "新建遮罩 (New Mask)", true, None::<&str>)?;
    let hide_all_i = MenuItem::with_id(app, "toggle_visibility", "显示/隐藏所有", true, None::<&str>)?;
    let unlock_all_i = MenuItem::with_id(app, "unlock_all", "🔓 解锁全部 (Unlock All)", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "退出 (Quit)", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[
        &new_i,
        &hide_all_i,
        &unlock_all_i,
        &quit_i,
    ])?;

    let tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "new_mask" => {
                let count = MASK_COUNTER.fetch_add(1, Ordering::SeqCst);
                let label = format!("mask_{}", count);
                
                let _ = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
                    .title("Useless Subtitle")
                    .inner_size(600.0, 120.0)
                    .min_inner_size(50.0, 20.0)
                    .decorations(false)
                    .transparent(true)
                    .always_on_top(true)
                    .skip_taskbar(true)
                    .resizable(true)
                    .build()
                    .unwrap();
            }
            "toggle_visibility" => {
                let windows = app.webview_windows();
                for (_, window) in windows.iter() {
                    if let Ok(visible) = window.is_visible() {
                        if visible {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                        }
                    }
                }
            }
            "unlock_all" => {
                let windows = app.webview_windows();
                for (_, window) in windows.iter() {
                    let _ = window.set_ignore_cursor_events(false);
                }
                // 通知前端 Vue 取消锁定状态，显示边框
                let _ = app.emit("unlock_all", ());
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                let app = tray.app_handle();
                let windows = app.webview_windows();
                for (_, window) in windows.iter() {
                    if let Ok(visible) = window.is_visible() {
                        if visible {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                        }
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}
