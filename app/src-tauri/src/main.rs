// Evita abrir um console extra em builds de release (relevante só fora do Linux).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    atlasoq_desktop_lib::run()
}
