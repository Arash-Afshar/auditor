const vscode = require("vscode");
const fetch = require("node-fetch");

const newNoteComment = (id, body, mode, author, parent, contextValue) => {
    return {
        id,
        body,
        mode,
        author,
        parent,
        contextValue
    }
}

async function commentHandler(context, endpoint) {
    const auditingFiletypes = vscode.workspace.getConfiguration().get('auditor.auditingFiletypes');

    endpoint += 'comments';
    const commentController = vscode.comments.createCommentController(
        "audit.comment-controller",
        "Comments/notes during code audits."
    );
    context.subscriptions.push(commentController);

    // A `CommentingRangeProvider` controls where gutter decorations that allow adding comments are shown
    commentController.commentingRangeProvider = {
        provideCommentingRanges: (document) => {
            if (auditingFiletypes.includes(document.languageId)) {
                return [new vscode.Range(0, 0, document.lineCount - 1, 0)];
            }
        },
    };

    // ------------------ Backend calls
    async function getCommentsFromBackend(fileName) {
        const response = await fetch(
            endpoint + "?" + new URLSearchParams({ file_name: fileName })
        );
        const review_state = await response.json();
        return review_state;
    }

    async function createCommentInBackend(fileName, lineNumber, body, author) {
        try {
            const response = await fetch(endpoint, {
                headers: {
                    Accept: "application/json",
                    "Content-Type": "application/json",
                },
                method: "POST",
                body: JSON.stringify({
                    file_name: fileName,
                    line_number: lineNumber,
                    body,
                    author
                }),
            });
            const comment_id = await response.json();
            return comment_id;
        } catch (error) {
            console.error("error updating review state:", error);
        }
    }

    async function deleteCommentInBackend(fileName, lineNumber, commentId) {
        try {
            await fetch(endpoint, {
                headers: {
                    Accept: "application/json",
                    "Content-Type": "application/json",
                },
                method: "DELETE",
                body: JSON.stringify({
                    file_name: fileName,
                    line_number: lineNumber,
                    comment_id: commentId,
                }),
            });
        } catch (error) {
            console.error("error updating review state:", error);
        }
    }

    // ------------------ Comment handlers

    function showComments(all_comments, uri) {
        for (const [line, comments] of Object.entries(all_comments)) {
            const lineNum = parseInt(line);
            let shownComments = [];
            const range = new vscode.Range(new vscode.Position(lineNum, 0), new vscode.Position(lineNum, 0));
            let thread = commentController.createCommentThread(uri, range, []);
            for (let i = 0; i < comments.length; i++) {
                const comment = comments[i];
                shownComments.push(newNoteComment(
                    comment.id,
                    comment.body,
                    vscode.CommentMode.Preview,
                    { name: comment.author },
                    thread,
                    "canDelete"
                ));
            }
            thread.comments = shownComments;
            thread.collapsibleState = vscode.CommentThreadCollapsibleState.Collapsed;
        }
    }

    async function replyNote(reply) {
        const thread = reply.thread;
        const author = "Arash";
        let fileName = thread.uri.path;
        let lineNumber = reply.thread.range.start.line;
        let comment_id = await createCommentInBackend(fileName, lineNumber, reply.text, author);
        const newComment = newNoteComment(
            comment_id,
            reply.text,
            vscode.CommentMode.Preview,
            { name: author },
            thread,
            "canDelete"
        );
        thread.comments = [...thread.comments, newComment];
    }

    // function editComment(comment) {
    //     if (!comment.parent) {
    //         return;
    //     }

    //     comment.parent.comments = comment.parent.comments.map((cmt) => {
    //         if (cmt.id === comment.id) {
    //             cmt.mode = vscode.CommentMode.Editing;
    //         }

    //         return cmt;
    //     });
    // }

    function saveComment(comment) {
        if (!comment.parent) {
            return;
        }

        comment.parent.comments = comment.parent.comments.map((cmt) => {
            if (cmt.id === comment.id) {
                cmt.body = cmt.body;
                cmt.mode = vscode.CommentMode.Preview;
            }

            return cmt;
        });
    }

    async function deleteComment(comment) {
        const thread = comment.parent;
        if (!thread) {
            return;
        }
        let fileName = thread.uri.path;
        let lineNumber = comment.parent.range.start.line;
        await deleteCommentInBackend(fileName, lineNumber, comment.id);
        thread.comments = thread.comments.filter((cmt) => cmt.id !== comment.id);

        if (thread.comments.length === 0) {
            thread.dispose();
        }
    }

    function cancelSaveComment(comment) {
        if (!comment.parent) {
            return;
        }

        comment.parent.comments = comment.parent.comments.map((cmt) => {
            if (cmt.id === comment.id) {
                cmt.body = cmt.body;
                cmt.mode = vscode.CommentMode.Preview;
            }

            return cmt;
        });
    }

    // --------------- Registering commands

    context.subscriptions.push(
        vscode.commands.registerCommand("auditor.createNote", (reply) => {
            replyNote(reply);
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand("auditor.replyNote", (reply) => {
            replyNote(reply);
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand("auditor.deleteNoteComment", deleteComment)
    );

    context.subscriptions.push(
        vscode.commands.registerCommand("auditor.deleteNote", (thread) => {
            thread.dispose();
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand("auditor.cancelsaveNote", cancelSaveComment)
    );

    context.subscriptions.push(
        vscode.commands.registerCommand("auditor.saveNote", saveComment)
    );

    // TODO: implement later
    // context.subscriptions.push(
    //     vscode.commands.registerCommand("auditor.editNote", editComment)
    // );

    context.subscriptions.push(
        vscode.commands.registerCommand("auditor.dispose", () => {
            commentController.dispose();
        })
    );

    // ----------------- Registering listeners
    let initialized = {};
    // duplicate code: run the above on the first activation as well
    let activeEditor = vscode.window.activeTextEditor;
    if (activeEditor) {
        const fileName = activeEditor.document.fileName;
        if (fileName.endsWith("cpp") || fileName.endsWith("h") || fileName.endsWith("go")) {
            if (!(fileName in initialized)) {
                initialized[fileName] = true;
                const comments = await getCommentsFromBackend(fileName);
                showComments(comments, activeEditor.document.uri);
            }
        }
    }

    vscode.window.onDidChangeActiveTextEditor(async (event) => {
        if (event != undefined) {
            const fileName = event.document.fileName;
            if (fileName.endsWith("cpp") || fileName.endsWith("h") || fileName.endsWith("go")) {
                if (!(fileName in initialized)) {
                    initialized[fileName] = true;
                    const comments = await getCommentsFromBackend(fileName);
                    showComments(comments, event.document.uri);
                }
            }
        }
    });
}

module.exports = commentHandler;