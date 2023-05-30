const vscode = require("vscode");

/**
 * @param {vscode.ExtensionContext} context
 */
function activate(context) {
  const prefix = "/home/aafshar/repos/vscode-extensions";

  const reviewedLineDecorationType =
    vscode.window.createTextEditorDecorationType({
      backgroundColor: { id: "auditor.reviewedBackground" },
    });

  const modifiedLineDecorationType =
    vscode.window.createTextEditorDecorationType({
      backgroundColor: { id: "auditor.modifiedBackground" },
    });
  const getReviewState = async (fileName) => {
    console.log("Querying the state for ", fileName);
    return {
      reviewed: [0, 1],
      modified: [2],
    };
  };

  const updateReviewState = async (
    fileName,
    startLine,
    endLine,
    reviewState
  ) => {
    console.log("Update", fileName, startLine, endLine, reviewState);
  };

  const showReviewState = ({ reviewed, modified }) => {
    let activeEditor = vscode.window.activeTextEditor;
    reviewed = new Set(reviewed);
    modified = new Set(modified);
    if (activeEditor) {
      const reviewedLines = [];
      const modifiedLines = [];
      for (let i = 0; i < activeEditor.document.lineCount; i++) {
        const decoration = {
          range: activeEditor.document.lineAt(i).range,
          // hoverMessage: "Reviewed",
          // hoverMessage: "Modified",
        };
        if (reviewed.has(i)) {
          reviewedLines.push(decoration);
        } else if (modified.has(i)) {
          modifiedLines.push(decoration);
        }
      }

      activeEditor.setDecorations(reviewedLineDecorationType, reviewedLines);
      activeEditor.setDecorations(modifiedLineDecorationType, modifiedLines);
    }
  };
  const updateStateCallback = (editor, state) => {
    let start = editor.selection.start.line;
    let end = editor.selection.end.line;
    if (end < start) {
      [start, end] = [end, start];
    }
    const fileName = editor.document.fileName.replace(prefix, "");
    updateReviewState(fileName, start, end, state);
  };

  vscode.commands.registerTextEditorCommand(
    "auditor.markAsReviewed",
    (editor) => {
      updateStateCallback(editor, "reviewed");
    }
  );

  vscode.commands.registerTextEditorCommand(
    "auditor.clearReviews",
    (editor) => {
      updateStateCallback(editor, "cleared");
    }
  );
  vscode.window.onDidChangeActiveTextEditor(async (event) => {
    if (event != undefined) {
      const fileName = event.document.fileName.replace(prefix, "");
      const state = await getReviewState(fileName);
      showReviewState(state);
    }
  });

  // duplicate code: run the above on the first activation as well
  let activeEditor = vscode.window.activeTextEditor;
  if (activeEditor) {
    const fileName = activeEditor.document.fileName.replace(prefix, "");
    getReviewState(fileName).then((state) => {
      showReviewState(state);
    });
  }
}

// This method is called when your extension is deactivated
function deactivate() { }

module.exports = {
  activate,
  deactivate,
};
