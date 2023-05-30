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
- `REPO_PATH=<path-to-the-repo-you-want-to-audit> DB_PATH=<path-to-a-json-file-to-store-reviews> cargo run --bin service -- --port 3000`
  - Use `cargo watch -- cargo run ...` during development for hot reloading


## Usage

- TODO