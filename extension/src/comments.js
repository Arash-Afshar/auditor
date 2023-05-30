const vscode = require("vscode");

let commentId = 1;

const newNoteComment = (body, mode, author, parent, contextValue) => {
    return {
        id: commentId++,
        body,
        mode,
        author,
        parent,
        contextValue
    }
}

function commentHandler(context, endpoint) {
    const commentController = vscode.comments.createCommentController(
        "audit.comment-controller",
        "Comments/notes during code audits."
    );
    context.subscriptions.push(commentController);

    // A `CommentingRangeProvider` controls where gutter decorations that allow adding comments are shown
    commentController.commentingRangeProvider = {
        provideCommentingRanges: (document) => {
            const lineCount = document.lineCount;
            return [new vscode.Range(0, 0, lineCount - 1, 0)];
        },
    };

    // ------------------ Comment handlers

    function editComment(comment) {
        if (!comment.parent) {
            return;
        }

        comment.parent.comments = comment.parent.comments.map((cmt) => {
            if (cmt.id === comment.id) {
                cmt.mode = vscode.CommentMode.Editing;
            }

            return cmt;
        });
    }

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

    function deleteComment(comment) {
        const thread = comment.parent;
        if (!thread) {
            return;
        }

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

    function replyNote(reply) {
        const thread = reply.thread;
        const newComment = newNoteComment(
            reply.text,
            vscode.CommentMode.Preview,
            { name: "Arash" },
            thread,
            "canDelete"
        );

        thread.comments = [...thread.comments, newComment];
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

    context.subscriptions.push(
        vscode.commands.registerCommand("auditor.editNote", editComment)
    );

    context.subscriptions.push(
        vscode.commands.registerCommand("auditor.dispose", () => {
            commentController.dispose();
        })
    );
}

module.exports = commentHandler;