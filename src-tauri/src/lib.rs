pub mod commands;
pub mod core;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .manage(AppState {
            place_db: crate::core::geo::PlaceDb::embedded(),
            cancel: std::sync::Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::pick_folder,
            commands::list_photos,
            commands::read_photo_meta,
            commands::make_thumb,
            commands::make_proxy,
            commands::render_preview,
            commands::build_watermark_svg,
            commands::cancel,
            commands::export_batch,
        ])
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
