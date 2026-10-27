// Pas de console en release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    portabl_apps_lib::run()
}
