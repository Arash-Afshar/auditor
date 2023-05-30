const commentHandler = require("./comments");
const linereviewHandler = require("./linereviews");

/**
 * @param {vscode.ExtensionContext} context
 */
function activate(context) {
  // Backend endpoint
  const endpoint = "http://localhost:3000/";

  commentHandler(context, endpoint);
  linereviewHandler(endpoint);
}

// This method is called when your extension is deactivated
function deactivate() { }

module.exports = {
  activate,
  deactivate,
};
