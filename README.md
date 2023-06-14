# auditor


## Installation for development
- Nodejs
- Yarn
- VScode development: https://code.visualstudio.com/api/get-started/your-first-extension
- VScode publishing: https://code.visualstudio.com/api/working-with-extensions/publishing-extension
- Install rustlang: https://www.rust-lang.org/tools/install
- Install Cargo watch for hot-reloading of the backend service: https://crates.io/crates/cargo-watch


## Installation for usage

### Install the extension

- `cd extension`
- `yarn install`
- `vsce package`
  - This will create a .vsix
- Go to the extension section of VSCode and install thee .vsix file


### Run the server

- `cd service`
- `REPO_PATH=<path-to-the-repo-you-want-to-audit> DB_PATH=<path-to-parent-directory-to-store-db> cargo run --bin auditor -- --port 3000`
  - Use `cargo watch -- cargo run ...` during development for hot reloading

### Run the web view

- `cd web`
- `trunk serve --open`
- Go to `http://localhost:8080`


## Usage

## Customize the extension colors

Modify the `settings.json` file and add the following:

```
  "workbench.colorCustomizations": {
    "auditor.reviewedBakcground":"#FFF00055",
    "auditor.modifiedBackground": "#FF000055",
    "auditor.ignoredBackground": "#D3D3D3F0"
  }
```

## Mark lines

Open the command pallet (cmd+shift+P) and search for the following. You can also assign your own shortcuts to these commands.
- Mark liens as modified
- Mark liens as reviewed
- Mark liens as cleared
- Mark liens as ignored

Click on the `+` sign that appears on the line number gutter to leave a comment


