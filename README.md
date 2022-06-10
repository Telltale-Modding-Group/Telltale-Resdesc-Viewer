# Telltale Resdesc Viewer
An easy way to view `_resdesc_` files.

![Screenshot](/assets/screenshot.png?raw=true)

## Getting Started
[Download the latest released .zip archive](https://github.com/Telltale-Modding-Group/Telltale-Resdesc-Viewer/releases/)
and extract it into a directory of your choice. **Make sure `telltale-resdesc-viewer.exe` and `ttarchext.exe` are 
present in the same directory!**

To open a `_resdesc_` file, open the application and go to file > open, then select the desired `_resdesc_` file.

Alternatively, right click on an existing `_resdesc_` file, Open with > Choose another app > More apps >
(Scroll to bottom) Look for another app on this PC, then choose `telltale-resdesc-viewer.exe`. This will automatically
open up the application with the contents of the chosen `_resdesc_` file.

Another method of opening `_resdesc_` files is to drag the file onto the `telltale-resdesc-viewer.exe` file directly,
which will open up the viewer with the contents of the dragged file.

## Building from Source
Before building, Rust, the Tauri framework and NodeJS must be installed:
 - [Tauri (which includes Rust)](https://tauri.studio/v1/guides/getting-started/prerequisites)
 - [NodeJS](https://nodejs.org/en/)

To get started, clone the repository (`git clone https://github.com/Telltale-Modding-Group/Telltale-Resdesc-Viewer`)
and navigate to the cloned directory in a terminal.

Next, run `npm i` to install JavaScript dependencies. Once this completes, run `npm run tauri:build`, which will generate
a binary in `./src-tauri/target/release`.

## Development
Follow the previous steps for building from source to install dependencies and clone the repository. Rather than
running `npm run tauri:build`, instead run `npm run tauri:dev`, which will automatically open a window with hot-reload
functionality built-in (i.e. making any code changes in Rust / JavaScript will refresh the application).

### Overview

The majority of the logic lives in `src-tauri/src/main.rs`, which initialises the application, loads any initial args, 
and handles decryption of `_resdesc_` files via `ttarchext.exe`.

The frontend code is native JavaScript without any UI frameworks (doing so would probably be overkill!). On load, 
`main.js` will call out to the `get_initial_contents` command in Rust, which will display the contents of the initial
path given to the application if present.

The menubar > file > open logic is handled in Rust, which will prompt the user with a filepicker to choose a .lua file.
If the chosen file is decrypted successfully, Rust will emit the contents of the file to the 'content' channel, which 
`main.js` listens to. When this happens, the frontend will be updated with the new contents.

## Contributions
Contributions are welcome! If you see a typo or see a better approach to the shoddy Rust, please make a pull request.