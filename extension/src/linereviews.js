
const vscode = require("vscode");
const fetch = require("node-fetch");

function linereviewHandler(baseEndpoint) {
    const reviewEndpoint = baseEndpoint + 'reviews';
    const transformReviewEndpoint = baseEndpoint + 'transform';

    const ignoredLineDecorationType =
        vscode.window.createTextEditorDecorationType({
            backgroundColor: { id: "auditor.ignoredBackground" },
        });

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
            reviewEndpoint + "?" + new URLSearchParams({ file_name: fileName })
        );
        const review_state = await response.json();
        return review_state;
    };

    const updateReviewState = async (
        fileName,
        startLine,
        endLine,
        reviewState,
        totalLines,
    ) => {
        try {
            await fetch(reviewEndpoint, {
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
                    total_lines: totalLines,
                }),
            });
            const state = await getReviewState(fileName);
            showReviewState(state);
        } catch (error) {
            console.error("error updating review state:", error);
        }
    };

    const transformReviewState = async (fileName) => {
        try {
            await fetch(transformReviewEndpoint, {
                headers: {
                    Accept: "application/json",
                    "Content-Type": "application/json",
                },
                method: "POST",
                body: JSON.stringify({
                    file_name: fileName,
                }),
            });
            const state = await getReviewState(fileName);
            showReviewState(state);
        } catch (error) {
            console.error("error updating review state:", error);
        }
    };

    const showReviewState = ({ reviewed, modified, ignored }) => {
        let activeEditor = vscode.window.activeTextEditor;

        let _reviewed = new Set();
        let _modified = new Set();
        let _ignored = new Set();

        for (let i = 0; i < reviewed.length; i++) {
            const [s, e] = reviewed[i];
            for (let j = s; j <= e; j++) {
                _reviewed.add(j);
            }
        }
        for (let i = 0; i < modified.length; i++) {
            const [s, e] = modified[i];
            for (let j = s; j <= e; j++) {
                _modified.add(j);
            }
        }
        for (let i = 0; i < ignored.length; i++) {
            const [s, e] = ignored[i];
            for (let j = s; j <= e; j++) {
                _ignored.add(j);
            }
        }

        reviewed = _reviewed;
        modified = _modified;
        ignored = _ignored;

        if (activeEditor) {
            const reviewedLines = [];
            const modifiedLines = [];
            const ignoredLines = [];
            for (let i = 0; i < activeEditor.document.lineCount; i++) {
                const decoration = {
                    range: activeEditor.document.lineAt(i).range,
                    // hoverMessage: "Reviewed",
                    // hoverMessage: "Modified",
                    // hoverMessage: "Ignored",
                };
                if (reviewed.has(i)) {
                    reviewedLines.push(decoration);
                } else if (modified.has(i)) {
                    modifiedLines.push(decoration);
                } else if (ignored.has(i)) {
                    ignoredLines.push(decoration);
                }
            }

            activeEditor.setDecorations(reviewedLineDecorationType, reviewedLines);
            activeEditor.setDecorations(modifiedLineDecorationType, modifiedLines);
            activeEditor.setDecorations(ignoredLineDecorationType, ignoredLines);
        }
    };
    const updateStateCallback = (editor, state) => {
        let start = editor.selection.start.line;
        let end = editor.selection.end.line;
        if (end < start) {
            [start, end] = [end, start];
        }
        const fileName = editor.document.fileName;
        const totalLines = editor.document.lineCount;
        updateReviewState(fileName, start, end, state, totalLines);
    };

    vscode.commands.registerTextEditorCommand(
        "auditor.transform",
        (editor) => {
            const fileName = editor.document.fileName;
            transformReviewState(fileName);
        }
    );

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

    vscode.commands.registerTextEditorCommand(
        "auditor.markAsIgnored",
        (editor) => {
            updateStateCallback(editor, "Ignored");
        }
    );

    vscode.window.onDidChangeActiveTextEditor(async (event) => {
        if (event != undefined) {
            const fileName = event.document.fileName;
            if (fileName.endsWith("cpp") || fileName.endsWith("h") || fileName.endsWith("go")) {
                const state = await getReviewState(fileName);
                showReviewState(state);
            }
        }
    });

    // duplicate code: run the above on the first activation as well
    let activeEditor = vscode.window.activeTextEditor;
    if (activeEditor) {
        const fileName = activeEditor.document.fileName;
        if (fileName.endsWith("cpp") || fileName.endsWith("h") || fileName.endsWith("go")) {
            getReviewState(fileName).then((state) => {
                showReviewState(state);
            });
        }
    }
}

module.exports = linereviewHandler;