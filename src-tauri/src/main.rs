#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde::Serialize;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use tauri::api::dialog::{message, FileDialogBuilder};
use tauri::api::process::{Command, CommandEvent};
use tauri::State;
use tauri::{CustomMenuItem, Manager, Menu, MenuItem, Submenu};
use tempdir::TempDir;

/// Represents a decrypted resdesc file that has gone through ttarchext
#[derive(Serialize)]
struct Content {
    /// The name of the input file
    filename: String,
    /// The decrypted contents of the input file
    content: String,
}

/// The global state used by the application
struct AppState {
    /// The initial path provided to the application via the first arg on the CLI
    /// NOTE: This is mainly used to allow [get_initial_contents] to retrieve the initial path.
    initial_path: Mutex<Option<String>>,
}

/// Given a path to a resdesc file, this attempts to decrypt it using ttarchext.exe. This will
/// return either the decrypted contents as [Content] if successful, or a [String] error message.
fn decrypt_resdesc_file(path: &str) -> Result<Content, String> {
    // Create a new temporary directory. This will be used to store the decrypted resdesc file.
    let temp_path = TempDir::new("resdesc").map_err(|_| "Unable to create temporary directory!")?;
    let temp_path_name = temp_path.path().to_str().ok_or("Invalid temp path")?;

    // Spawns the ttarchext.exe process via Tauri's "sidecar" concept. The configuration for this
    // lives in tauri.conf.json
    let (mut rx, _) = Command::new_sidecar("ttarchext")
        .map_err(|_| "Unable to create ttarchext sidecar!")?
        .args(["67", path, temp_path_name])
        .spawn()
        .map_err(|_| "Unable to spawn ttarchext process!")?;

    // Listen to the output of ttarchext.exe. If something goes wrong, immediately return an Err.
    while let Some(event) = rx.blocking_recv() {
        let mut error = "".to_string();
        match event {
            CommandEvent::Stderr(err) => error = err,
            CommandEvent::Error(err) => error = err,
            _ => {}
        }

        if !error.is_empty() {
            return Err(error);
        }
    }

    // ttarchext.exe completed without any errors. From here, read the contents of the file and
    // place them within a [Content] struct.
    let source_path = Path::new(&path);
    let filename = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Unable to get filename from source path!")?;
    let decrypted_file_path = Path::new(filename);

    let content = fs::read_to_string(temp_path.path().join(decrypted_file_path))
        .map_err(|_| "Unable to read file!")?;

    Ok(Content {
        filename: filename.to_string(),
        content,
    })
}

/// Retrieves the contents of the lua file passed in as a CLI arg, if present.
#[tauri::command]
fn get_initial_contents(state: State<AppState>) -> Option<Content> {
    state
        .initial_path
        // Lots of tomfoolery just to get access to initial_path, gah...
        .lock()
        .unwrap()
        .clone()
        // Attempt to retrieve the contents of the initial path. If something goes wrong,
        // hide the error.
        .and_then(|path| decrypt_resdesc_file(&path).ok())
}

fn main() {
    // Build up the menu for the window.
    // NOTE: God DAMN this is a lot of boilerplate for a single file dropdown with a grand total of
    // TWO buttons...
    let file_submenu = Submenu::new(
        "File",
        Menu::new()
            .add_item(CustomMenuItem::new("open", "Open"))
            .add_native_item(MenuItem::Quit),
    );
    let menu = Menu::new().add_submenu(file_submenu);

    tauri::Builder::default()
        // Register get_initial_contents as a command, VERY important!
        .invoke_handler(tauri::generate_handler![get_initial_contents])
        .manage(AppState {
            initial_path: Mutex::new(None),
        })
        .setup(|app| {
            let global_state: State<AppState> = app.state();

            // Go through and see if an arg was given to the application. If so, use that arg
            // as the content of initial_path in the global state.
            if let Ok(matches) = app.get_cli_matches() {
                if let Some(arg) = matches.args.get("source") {
                    // Get a lock on the global state, and assign the arg to the initial_path.
                    *global_state.initial_path.lock().unwrap() =
                        arg.value.as_str().map(|str| str.to_string());
                }
            }

            Ok(())
        })
        .menu(menu)
        .on_menu_event(|event| match event.menu_item_id() {
            // When file > open is clicked, an open file dialog should be displayed, limited to
            // only .lua files
            "open" => FileDialogBuilder::new()
                .add_filter("resdesc", &["lua"])
                .pick_file(move |path_opt| {
                    if let Some(path) = path_opt {
                        let path_str = path.to_str().expect("Path was invalid");

                        // Attempt to decrypt the selected file. If successful, display its contents
                        // to the user, otherwise show an error message.
                        match decrypt_resdesc_file(path_str) {
                            Ok(content) => event
                                .window()
                                .emit("content", &content)
                                .expect("Unable to emit decrypted file"),
                            Err(error) => {
                                message(
                                    Some(event.window()),
                                    "Error",
                                    format!("There was an error opening the file:\n{}", error),
                                );
                            }
                        }
                    }
                }),
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
