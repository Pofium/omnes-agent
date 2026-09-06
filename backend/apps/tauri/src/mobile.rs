//! Mobile entry point for OmnesAgent Desktop (iOS/Android).

#[tauri::mobile_entry_point]
fn main() {
    omnesagent_desktop::run();
}
