use crate::{
    models::{
        AppConfig, MaskAppearance, MaskConfig, MaskGeometry, MaskViewState, RuntimeMaskState,
    },
    persistence,
};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
        Arc, Mutex,
    },
    time::Duration,
};
use tauri::{tray::TrayIcon, AppHandle};

enum SaveMessage {
    Schedule,
    Flush(Sender<Result<(), String>>),
}

struct SharedState {
    config: Mutex<AppConfig>,
    runtime: Mutex<HashMap<String, RuntimeMaskState>>,
    settings_target: Mutex<Option<String>>,
    config_path: PathBuf,
    tray: Mutex<Option<TrayIcon>>,
    tray_ready: AtomicBool,
}

#[derive(Clone)]
pub struct AppState {
    shared: Arc<SharedState>,
    save_sender: Sender<SaveMessage>,
}

impl AppState {
    pub fn load(app: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let loaded = persistence::load(app).map_err(std::io::Error::other)?;
        let shared = Arc::new(SharedState {
            config: Mutex::new(loaded.config),
            runtime: Mutex::new(HashMap::new()),
            settings_target: Mutex::new(None),
            config_path: loaded.path,
            tray: Mutex::new(None),
            tray_ready: AtomicBool::new(false),
        });
        let (save_sender, receiver) = mpsc::channel();
        spawn_save_worker(shared.clone(), receiver);

        let state = Self {
            shared,
            save_sender,
        };
        if loaded.needs_save {
            state.schedule_save();
        }
        Ok(state)
    }

    pub fn config(&self) -> AppConfig {
        self.shared.config.lock().unwrap().clone()
    }

    pub fn mark_initialized(&self) {
        self.shared.config.lock().unwrap().initialized = true;
        self.schedule_save();
    }

    pub fn add_mask(&self, mask: MaskConfig) {
        self.shared.config.lock().unwrap().masks.push(mask.clone());
        self.shared.runtime.lock().unwrap().insert(
            mask.id,
            RuntimeMaskState {
                locked: false,
                visible: true,
            },
        );
        self.schedule_save();
    }

    pub fn register_mask(&self, id: &str) {
        self.shared.runtime.lock().unwrap().insert(
            id.to_string(),
            RuntimeMaskState {
                locked: false,
                visible: true,
            },
        );
    }

    pub fn remove_mask(&self, id: &str) -> bool {
        let mut config = self.shared.config.lock().unwrap();
        let initial_length = config.masks.len();
        config.masks.retain(|mask| mask.id != id);
        let removed = config.masks.len() != initial_length;
        drop(config);
        self.shared.runtime.lock().unwrap().remove(id);
        if removed {
            self.schedule_save();
        }
        removed
    }

    pub fn mask(&self, id: &str) -> Option<MaskConfig> {
        self.shared
            .config
            .lock()
            .unwrap()
            .masks
            .iter()
            .find(|mask| mask.id == id)
            .cloned()
    }

    pub fn update_appearance(
        &self,
        id: &str,
        color: String,
        opacity: u8,
    ) -> Result<MaskAppearance, String> {
        let mut config = self.shared.config.lock().unwrap();
        let mask = config
            .masks
            .iter_mut()
            .find(|mask| mask.id == id)
            .ok_or_else(|| format!("unknown mask {id}"))?;
        mask.appearance = MaskAppearance { color, opacity };
        mask.appearance.validate();
        let appearance = mask.appearance.clone();
        drop(config);
        self.schedule_save();
        Ok(appearance)
    }

    pub fn update_geometry(&self, id: &str, mut geometry: MaskGeometry) -> Result<(), String> {
        geometry.validate();
        let mut config = self.shared.config.lock().unwrap();
        let mask = config
            .masks
            .iter_mut()
            .find(|mask| mask.id == id)
            .ok_or_else(|| format!("unknown mask {id}"))?;
        mask.geometry = geometry;
        drop(config);
        self.schedule_save();
        Ok(())
    }

    pub fn runtime(&self, id: &str) -> Option<RuntimeMaskState> {
        self.shared.runtime.lock().unwrap().get(id).copied()
    }

    pub fn set_locked(&self, id: &str, locked: bool) -> Result<(), String> {
        let mut runtime = self.shared.runtime.lock().unwrap();
        let state = runtime
            .get_mut(id)
            .ok_or_else(|| format!("unknown runtime mask {id}"))?;
        state.locked = locked;
        Ok(())
    }

    pub fn set_visible(&self, id: &str, visible: bool) -> Result<(), String> {
        let mut runtime = self.shared.runtime.lock().unwrap();
        let state = runtime
            .get_mut(id)
            .ok_or_else(|| format!("unknown runtime mask {id}"))?;
        state.visible = visible;
        Ok(())
    }

    pub fn view(&self, id: &str) -> Option<MaskViewState> {
        let mask = self.mask(id)?;
        let runtime = self.runtime(id).unwrap_or_default();
        Some(MaskViewState {
            id: mask.id,
            name: mask.name,
            color: mask.appearance.color,
            opacity: mask.appearance.opacity,
            locked: runtime.locked,
            visible: runtime.visible,
            tray_ready: self.tray_ready(),
        })
    }

    pub fn set_settings_target(&self, id: Option<String>) {
        *self.shared.settings_target.lock().unwrap() = id;
    }

    pub fn settings_target(&self) -> Option<String> {
        self.shared.settings_target.lock().unwrap().clone()
    }

    pub fn set_tray(&self, tray: TrayIcon) {
        *self.shared.tray.lock().unwrap() = Some(tray);
        self.shared.tray_ready.store(true, Ordering::SeqCst);
    }

    pub fn tray(&self) -> Option<TrayIcon> {
        self.shared.tray.lock().unwrap().clone()
    }

    pub fn tray_ready(&self) -> bool {
        self.shared.tray_ready.load(Ordering::SeqCst)
    }

    pub fn schedule_save(&self) {
        let _ = self.save_sender.send(SaveMessage::Schedule);
    }

    pub fn flush(&self) -> Result<(), String> {
        let (sender, receiver) = mpsc::channel();
        self.save_sender
            .send(SaveMessage::Flush(sender))
            .map_err(|error| error.to_string())?;
        receiver.recv().map_err(|error| error.to_string())?
    }
}

fn spawn_save_worker(shared: Arc<SharedState>, receiver: Receiver<SaveMessage>) {
    std::thread::spawn(move || loop {
        match receiver.recv() {
            Ok(SaveMessage::Schedule) => loop {
                match receiver.recv_timeout(Duration::from_millis(250)) {
                    Ok(SaveMessage::Schedule) => continue,
                    Ok(SaveMessage::Flush(reply)) => {
                        let result = save_snapshot(&shared);
                        let _ = reply.send(result);
                        break;
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        if let Err(error) = save_snapshot(&shared) {
                            eprintln!("failed to save mask state: {error}");
                        }
                        break;
                    }
                    Err(RecvTimeoutError::Disconnected) => return,
                }
            },
            Ok(SaveMessage::Flush(reply)) => {
                let _ = reply.send(save_snapshot(&shared));
            }
            Err(_) => return,
        }
    });
}

fn save_snapshot(shared: &SharedState) -> Result<(), String> {
    let snapshot = shared.config.lock().unwrap().clone();
    persistence::save(&shared.config_path, &snapshot)
}
