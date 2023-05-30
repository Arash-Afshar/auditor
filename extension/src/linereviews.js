
const vscode = require("vscode");
const fetch = require("node-fetch");

function linereviewHandler(endpoint) {
    const reviewedLineDecorationType =
        vscode.window.createTextEditorDecorationType({
            backgroundColor: { id: "auditor.reviewedBackground" },
        });

    const modifiedLineDecorationType =
        vscode.window.createTextEditorDecorationType({
            backgroundColor: { id: "auditor.modifiedBackground" },
        });

    const getReviewState = async (fileName) => {
        const response = await fetch(
            endpoint + "?" + new URLSearchParams({ file_name: fileName })
        );
        const review_state = await response.json();
        console.log("querying", fileName, "result:", review_state);
        return review_state;
    };

    const updateReviewState = async (
        fileName,
        startLine,
        endLine,
        reviewState
    ) => {
        try {
            await fetch(endpoint, {
                headers: {
                    Accept: "application/json",
                    "Content-Type": "application/json",
                },
                method: "POST",
                body: JSON.stringify({
                    file_name: fileName,
                    start_line: startLine,
                    end_line: endLine,
                    review_state: reviewState,
                }),
            });
            const state = await getReviewState(fileName);
            showReviewState(state);
        } catch (error) {
            console.error("error updating review state:", error);
        }
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
        const fileName = editor.document.fileName;
        updateReviewState(fileName, start, end, state);
    };

    vscode.commands.registerTextEditorCommand(
        "auditor.markAsReviewed",
        (editor) => {
            updateStateCallback(editor, "Reviewed");
        }
    );

    vscode.commands.registerTextEditorCommand(
        "auditor.markAsModified",
        (editor) => {
            updateStateCallback(editor, "Modified");
        }
    );

    vscode.commands.registerTextEditorCommand(
        "auditor.clearReviews",
        (editor) => {
            updateStateCallback(editor, "Cleared");
        }
    );
    vscode.window.onDidChangeActiveTextEditor(async (event) => {
        if (event != undefined) {
            const fileName = event.document.fileName;
            const state = await getReviewState(fileName);
            showReviewState(state);
        }
    });

    // duplicate code: run the above on the first activation as well
    let activeEditor = vscode.window.activeTextEditor;
    if (activeEditor) {
        const fileName = activeEditor.document.fileName;
        getReviewState(fileName).then((state) => {
            showReviewState(state);
        });
    }
}

module.exports = linereviewHandler;