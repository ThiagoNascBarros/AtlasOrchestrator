//! Integração Tauri do AtlasOQ: comandos, canais de saída e ciclo de vida do app.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("falha ao iniciar o AtlasOQ");
}
