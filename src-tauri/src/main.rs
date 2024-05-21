// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::borrow::Borrow;

use tauri::{AboutMetadata, CustomMenuItem, Manager, Menu, MenuItem, Submenu};
use tauri_plugin_theme_v1::{get_theme, set_theme, Theme, ThemePlugin};

mod udp;

#[tauri::command]
fn open_save_dialog(dir: &str, default_file_name: &str) -> Result<String, ()> {
    let fd = native_dialog::FileDialog::new()
        .set_location(dir)
        .set_filename(default_file_name)
        .show_save_single_file();
    if let Ok(Some(fd)) = fd {
        if let Some(p) = fd.to_str() {
            Ok(p.to_string())
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

fn main() {
    let mut ctx = tauri::generate_context!();
    let current_theme = get_theme(ctx.config());
    let app_name = "CDA";
    let mut menu = Menu::new();
    let mut theme_auto = CustomMenuItem::new("theme_auto".to_string(), "Auto");
    let mut theme_light = CustomMenuItem::new("theme_light".to_string(), "Light");
    let mut theme_dark = CustomMenuItem::new("theme_dark".to_string(), "Dark");
    match current_theme {
        Theme::Auto => {
            theme_auto.selected = true;
        }
        Theme::Light => {
            theme_light.selected = true;
        }
        Theme::Dark => {
            theme_dark.selected = true;
        }
    }
    #[cfg(target_os = "macos")]
    {
        menu = menu.add_submenu(Submenu::new(
            app_name,
            Menu::new()
                .add_native_item(MenuItem::About(
                    app_name.to_string(),
                    AboutMetadata::default(),
                ))
                .add_submenu(Submenu::new(
                    "Theme",
                    Menu::new()
                        .add_item(theme_auto)
                        .add_item(theme_light)
                        .add_item(theme_dark),
                ))
                .add_native_item(MenuItem::Separator)
                .add_native_item(MenuItem::Services)
                .add_native_item(MenuItem::Separator)
                .add_native_item(MenuItem::Hide)
                .add_native_item(MenuItem::HideOthers)
                .add_native_item(MenuItem::ShowAll)
                .add_native_item(MenuItem::Separator)
                .add_native_item(MenuItem::Quit),
        ));
    }

    let mut file_menu = Menu::new();
    let mut new_udp_window = CustomMenuItem::new("new_udp_window".to_string(), "New UDP Window");
    new_udp_window.keyboard_accelerator = Some("CmdOrCtrl+Shift+U".to_string());
    let mut new_websocket_window =
        CustomMenuItem::new("new_websocket_window".to_string(), "New Websocket Window");
    new_websocket_window.keyboard_accelerator = Some("CmdOrCtrl+Shift+W".to_string());
    file_menu = file_menu
        .add_item(new_udp_window)
        .add_item(new_websocket_window);
    #[cfg(target_os = "macos")]
    {
        file_menu = file_menu.add_native_item(MenuItem::Separator);
    }
    file_menu = file_menu.add_native_item(MenuItem::CloseWindow);
    #[cfg(not(target_os = "macos"))]
    {
        file_menu = file_menu.add_native_item(MenuItem::Quit);
    }
    menu = menu.add_submenu(Submenu::new("File", file_menu));

    #[cfg(not(target_os = "linux"))]
    let mut edit_menu = Menu::new();
    #[cfg(target_os = "macos")]
    {
        edit_menu = edit_menu.add_native_item(MenuItem::Undo);
        edit_menu = edit_menu.add_native_item(MenuItem::Redo);
        edit_menu = edit_menu.add_native_item(MenuItem::Separator);
    }
    #[cfg(not(target_os = "linux"))]
    {
        edit_menu = edit_menu.add_native_item(MenuItem::Cut);
        edit_menu = edit_menu.add_native_item(MenuItem::Copy);
        edit_menu = edit_menu.add_native_item(MenuItem::Paste);
    }
    #[cfg(target_os = "macos")]
    {
        edit_menu = edit_menu.add_native_item(MenuItem::SelectAll);
    }
    #[cfg(not(target_os = "linux"))]
    {
        menu = menu.add_submenu(Submenu::new("Edit", edit_menu));
    }
    #[cfg(target_os = "macos")]
    {
        menu = menu.add_submenu(Submenu::new(
            "View",
            Menu::new().add_native_item(MenuItem::EnterFullScreen),
        ));
    }

    let mut window_menu = Menu::new();
    window_menu = window_menu.add_native_item(MenuItem::Minimize);
    #[cfg(target_os = "macos")]
    {
        window_menu = window_menu.add_native_item(MenuItem::Zoom);
        window_menu = window_menu.add_native_item(MenuItem::Separator);
    }
    window_menu = window_menu.add_native_item(MenuItem::CloseWindow);
    menu = menu.add_submenu(Submenu::new("Window", window_menu));

    let theme = ThemePlugin::init(ctx.config_mut());

    // let mut theme_auto = rw_theme_auto.clone();
    tauri::Builder::default()
        .menu(menu)
        .on_menu_event(|event| match event.menu_item_id() {
            "new_udp_window" => {
                let label = "udp-".to_string() + &uuid::Uuid::new_v4().to_string();
                tauri::WindowBuilder::new(
                    event.window().app_handle().borrow(),
                    &label,
                    tauri::WindowUrl::App("index.html#udp".into()),
                )
                .build()
                .unwrap();
            }
            "new_websocket_window" => {
                let label: String = "ws-".to_string() + &uuid::Uuid::new_v4().to_string();
                tauri::WindowBuilder::new(
                    event.window().app_handle().borrow(),
                    &label,
                    tauri::WindowUrl::App("index.html#websocket".into()),
                )
                .build()
                .unwrap();
            }
            "theme_auto" => {
                let _ = set_theme(event.window().app_handle(), Theme::Auto);
                for w in event.window().app_handle().windows() {
                    let _ = w.1.menu_handle().get_item("theme_auto").set_selected(true);
                    let _ =
                        w.1.menu_handle()
                            .get_item("theme_light")
                            .set_selected(false);
                    let _ = w.1.menu_handle().get_item("theme_dark").set_selected(false);
                    let _ = w.1.emit("theme-change", "auto");
                }
            }
            "theme_light" => {
                let _ = set_theme(event.window().app_handle(), Theme::Light);
                for w in event.window().app_handle().windows() {
                    let _ = w.1.menu_handle().get_item("theme_auto").set_selected(false);
                    let _ = w.1.menu_handle().get_item("theme_light").set_selected(true);
                    let _ = w.1.menu_handle().get_item("theme_dark").set_selected(false);
                    let _ = w.1.emit("theme-change", "light");
                }
            }
            "theme_dark" => {
                let _ = set_theme(event.window().app_handle(), Theme::Dark);
                for w in event.window().app_handle().windows() {
                    let _ = w.1.menu_handle().get_item("theme_auto").set_selected(false);
                    let _ =
                        w.1.menu_handle()
                            .get_item("theme_light")
                            .set_selected(false);
                    let _ = w.1.menu_handle().get_item("theme_dark").set_selected(true);
                    let _ = w.1.emit("theme-change", "dark");
                }
            }
            _ => {}
        })
        .on_page_load(|w, _| {
            println!("on_page_load");
            let theme = get_theme(w.config().borrow());
            let _ = set_theme(w.app_handle(), theme);
        })
        .on_window_event(|e| match e.event() {
            tauri::WindowEvent::Destroyed => {
                let label = e.window().label().to_string();
                println!("{} destroyed", &label);
                tauri::async_runtime::spawn(async {
                    udp::unbind(label).await;
                });
            }
            _ => {}
        })
        .plugin(theme)
        .invoke_handler(tauri::generate_handler![
            open_save_dialog,
            udp::udp_bind,
            udp::udp_unbind,
            udp::udp_send
        ])
        .run(ctx)
        .expect("error while running tauri application");
}
