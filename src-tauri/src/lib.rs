// Mobile entry point — delegates to the same app setup as main.rs.
// Required by Cargo for mobile targets (crate-type = ["staticlib", "cdylib", "rlib"]).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
