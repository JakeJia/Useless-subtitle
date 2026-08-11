use crate::models::{MaskGeometry, DEFAULT_HEIGHT, DEFAULT_WIDTH, MIN_HEIGHT, MIN_WIDTH};
use tauri::{AppHandle, Manager, Monitor, PhysicalPosition, PhysicalSize, WebviewWindow};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicalRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct ResolvedGeometry {
    pub position: PhysicalPosition<i32>,
    pub size: PhysicalSize<u32>,
}

pub fn default_geometry(app: &AppHandle, cascade_index: usize) -> Result<MaskGeometry, String> {
    let monitor = app
        .primary_monitor()
        .map_err(|error| error.to_string())?
        .or_else(|| app.available_monitors().ok()?.into_iter().next())
        .ok_or_else(|| "no connected display is available".to_string())?;

    let scale = monitor.scale_factor();
    let display_width = monitor.size().width as f64 / scale;
    let display_height = monitor.size().height as f64 / scale;
    let cascade = (cascade_index % 8) as f64 * 24.0;

    Ok(MaskGeometry {
        monitor_key: monitor_key(&monitor),
        offset_x_logical: ((display_width - DEFAULT_WIDTH) / 2.0 + cascade).max(0.0),
        offset_y_logical: ((display_height - DEFAULT_HEIGHT) / 2.0 + cascade).max(0.0),
        width_logical: DEFAULT_WIDTH.min(display_width),
        height_logical: DEFAULT_HEIGHT.min(display_height),
        saved_scale_factor: scale,
    })
}

pub fn cascaded_geometry(
    app: &AppHandle,
    source: &MaskGeometry,
    cascade_index: usize,
) -> Result<MaskGeometry, String> {
    let mut geometry = source.clone();
    let cascade = ((cascade_index % 8) + 1) as f64 * 24.0;
    geometry.offset_x_logical += cascade;
    geometry.offset_y_logical += cascade;
    let resolved = resolve_geometry(app, &geometry)?;
    geometry_from_physical(app, resolved.position, resolved.size)
}

pub fn resolve_geometry(
    app: &AppHandle,
    geometry: &MaskGeometry,
) -> Result<ResolvedGeometry, String> {
    let monitors = app
        .available_monitors()
        .map_err(|error| error.to_string())?;
    let monitor = choose_monitor(app, &monitors, geometry.monitor_key.as_deref())?;
    let scale = monitor.scale_factor();

    let desired = PhysicalRect {
        x: monitor.position().x + (geometry.offset_x_logical * scale).round() as i32,
        y: monitor.position().y + (geometry.offset_y_logical * scale).round() as i32,
        width: (geometry.width_logical.max(MIN_WIDTH) * scale).round() as u32,
        height: (geometry.height_logical.max(MIN_HEIGHT) * scale).round() as u32,
    };

    let bounds = PhysicalRect {
        x: monitor.position().x,
        y: monitor.position().y,
        width: monitor.size().width,
        height: monitor.size().height,
    };
    let clamped = clamp_physical_rect(desired, bounds, scale);

    Ok(ResolvedGeometry {
        position: PhysicalPosition::new(clamped.x, clamped.y),
        size: PhysicalSize::new(clamped.width, clamped.height),
    })
}

pub fn geometry_from_window(window: &WebviewWindow) -> Result<MaskGeometry, String> {
    let position = window.outer_position().map_err(|error| error.to_string())?;
    let size = window.outer_size().map_err(|error| error.to_string())?;
    geometry_from_physical(window.app_handle(), position, size)
}

fn geometry_from_physical(
    app: &AppHandle,
    position: PhysicalPosition<i32>,
    size: PhysicalSize<u32>,
) -> Result<MaskGeometry, String> {
    let monitor = app
        .monitor_from_point(position.x as f64, position.y as f64)
        .map_err(|error| error.to_string())?
        .or_else(|| app.primary_monitor().ok().flatten())
        .ok_or_else(|| "no connected display is available".to_string())?;
    let scale = monitor.scale_factor();

    Ok(MaskGeometry {
        monitor_key: monitor_key(&monitor),
        offset_x_logical: (position.x - monitor.position().x) as f64 / scale,
        offset_y_logical: (position.y - monitor.position().y) as f64 / scale,
        width_logical: size.width as f64 / scale,
        height_logical: size.height as f64 / scale,
        saved_scale_factor: scale,
    })
}

pub fn recover_window_geometry(
    window: &WebviewWindow,
    geometry: &MaskGeometry,
) -> Result<(), String> {
    let resolved = resolve_geometry(window.app_handle(), geometry)?;
    window
        .set_position(resolved.position)
        .map_err(|error| error.to_string())?;
    window
        .set_size(resolved.size)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn choose_monitor(
    app: &AppHandle,
    monitors: &[Monitor],
    requested_key: Option<&str>,
) -> Result<Monitor, String> {
    if let Some(key) = requested_key {
        if let Some(found) = monitors
            .iter()
            .find(|monitor| monitor_key(monitor).as_deref() == Some(key))
        {
            return Ok(found.clone());
        }
    }

    app.primary_monitor()
        .map_err(|error| error.to_string())?
        .or_else(|| monitors.first().cloned())
        .ok_or_else(|| "no connected display is available".to_string())
}

fn monitor_key(monitor: &Monitor) -> Option<String> {
    monitor.name().cloned().or_else(|| {
        Some(format!(
            "{}:{}:{}x{}",
            monitor.position().x,
            monitor.position().y,
            monitor.size().width,
            monitor.size().height
        ))
    })
}

pub fn clamp_physical_rect(
    desired: PhysicalRect,
    bounds: PhysicalRect,
    scale_factor: f64,
) -> PhysicalRect {
    let scale = if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    };
    let min_width = (MIN_WIDTH * scale).round() as u32;
    let min_height = (MIN_HEIGHT * scale).round() as u32;
    let width = desired.width.max(min_width).min(bounds.width.max(1));
    let height = desired.height.max(min_height).min(bounds.height.max(1));
    let max_x = bounds
        .x
        .saturating_add(bounds.width.saturating_sub(width) as i32);
    let max_y = bounds
        .y
        .saturating_add(bounds.height.saturating_sub(height) as i32);

    PhysicalRect {
        x: desired.x.clamp(bounds.x, max_x),
        y: desired.y.clamp(bounds.y, max_y),
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_off_screen_rectangles() {
        let bounds = PhysicalRect {
            x: -1920,
            y: 0,
            width: 1920,
            height: 1080,
        };
        let desired = PhysicalRect {
            x: 5000,
            y: -500,
            width: 600,
            height: 120,
        };
        assert_eq!(
            clamp_physical_rect(desired, bounds, 1.0),
            PhysicalRect {
                x: -600,
                y: 0,
                width: 600,
                height: 120
            }
        );
    }

    #[test]
    fn applies_dpi_aware_minimum_size() {
        let bounds = PhysicalRect {
            x: 0,
            y: 0,
            width: 3840,
            height: 2160,
        };
        let desired = PhysicalRect {
            x: 10,
            y: 10,
            width: 10,
            height: 10,
        };
        let result = clamp_physical_rect(desired, bounds, 2.0);
        assert_eq!(result.width, 192);
        assert_eq!(result.height, 64);
    }

    #[test]
    fn keeps_large_windows_inside_display_bounds() {
        let bounds = PhysicalRect {
            x: 0,
            y: 0,
            width: 1280,
            height: 720,
        };
        let desired = PhysicalRect {
            x: -10,
            y: 40,
            width: 2000,
            height: 900,
        };
        assert_eq!(clamp_physical_rect(desired, bounds, 1.0), bounds);
    }
}
