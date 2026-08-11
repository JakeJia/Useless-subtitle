// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod tray;
use tauri::{WebviewWindowBuilder, WebviewUrl};
use tauri_plugin_store::StoreExt;
use serde_json::Value;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            tray::setup_tray(app.handle())?;

            // Safely load historical windows
            let store = app.store("store.json");
            
            match store {
                Ok(store_impl) => {
                    let mut has_windows = false;
                    
                    if let Some(mask_list) = store_impl.get("mask_list") {
                        if let Some(arr) = mask_list.as_array() {
                            for item in arr {
                                if let Some(obj) = item.as_object() {
                                    if let (Some(Value::String(label)), Some(Value::Number(x)), Some(Value::Number(y)), Some(Value::Number(w)), Some(Value::Number(h))) = 
                                        (obj.get("label"), obj.get("x"), obj.get("y"), obj.get("width"), obj.get("height")) {
                                            
                                        let x_pos = x.as_f64().unwrap_or(100.0);
                                        let y_pos = y.as_f64().unwrap_or(100.0);
                                        let width = w.as_f64().unwrap_or(600.0);
                                        let height = h.as_f64().unwrap_or(120.0);

                                        has_windows = true;
                                        let _ = WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html".into()))
                                            .title("Useless Subtitle")
                                            .inner_size(width, height)
                                            .position(x_pos, y_pos)
                                            .min_inner_size(50.0, 20.0)
                                            .decorations(false)
                                            .always_on_top(true)
                                            .skip_taskbar(true)
                                            .resizable(true)
                                            .build();
                                    }
                                }
                            }
                        }
                    }
                    
                    if !has_windows {
                         let _ = WebviewWindowBuilder::new(app, "mask_0", WebviewUrl::App("index.html".into()))
                            .title("Useless Subtitle")
                            .inner_size(600.0, 120.0)
                            .min_inner_size(50.0, 20.0)
                            .decorations(false)
                            .always_on_top(true)
                            .skip_taskbar(true)
                            .resizable(true)
                            .build();
                    }
                }
                Err(_) => {
                    let _ = WebviewWindowBuilder::new(app, "mask_0", WebviewUrl::App("index.html".into()))
                        .title("Useless Subtitle")
                        .inner_size(600.0, 120.0)
                        .min_inner_size(50.0, 20.0)
                        .decorations(false)
                        .always_on_top(true)
                        .skip_taskbar(true)
                        .resizable(true)
                        .build();
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
