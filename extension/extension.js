const vscode = require("vscode");
const fetch = require("node-fetch");

let commentId = 1;

class NoteComment {
  id; //: number;
  label; //: string | undefined;
  savedBody; //: string | vscode.MarkdownString; // for the Cancel button

  constructor(body, mode, author, parent, contextValue) {
    this.id = ++commentId;
    this.savedBody = body;
  }
}

/**
 * @param {vscode.ExtensionContext} context
 */
function activate(context) {
  // Backend endpoint
  const endpoint = "http://localhost:3000/review";

  // -------------------------- Comment handling
  const commentController = vscode.comments.createCommentController(
    "audit.comment-controller",
    "Comments/notes during code audits."
  );
  context.subscriptions.push(commentController);

  // A `CommentingRangeProvider` controls where gutter decorations that allow adding comments are shown
  commentController.commentingRangeProvider = {
    provideCommentingRanges: (document, token) => {
      const lineCount = document.lineCount;
      return [new vscode.Range(0, 0, lineCount - 1, 0)];
    },
  };

  context.subscriptions.push(
    vscode.commands.registerCommand("auditor.createNote", (reply) => {
      console.log("----> createNote");
      replyNote(reply);
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("auditor.replyNote", (reply) => {
      console.log("----> replyNote");
      replyNote(reply);
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("auditor.startDraft", (reply) => {
      console.log("----> startDraft");
      const thread = reply.thread;
      thread.contextValue = "draft";
      const newComment = new NoteComment(
        reply.text,
        vscode.CommentMode.Preview,
        { name: "vscode" },
        thread
      );
      newComment.label = "pending";
      thread.comments = [...thread.comments, newComment];
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("auditor.finishDraft", (reply) => {
      console.log("----> finishDraft");
      const thread = reply.thread;

      if (!thread) {
        return;
      }

      thread.contextValue = undefined;
      thread.collapsibleState = vscode.CommentThreadCollapsibleState.Collapsed;
      if (reply.text) {
        const newComment = new NoteComment(
          reply.text,
          vscode.CommentMode.Preview,
          { name: "vscode" },
          thread
        );
        thread.comments = [...thread.comments, newComment].map((comment) => {
          comment.label = undefined;
          return comment;
        });
      }
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("auditor.deleteNoteComment", (comment) => {
      console.log("----> deleteNoteComment");
      const thread = comment.parent;
      if (!thread) {
        return;
      }

      thread.comments = thread.comments.filter((cmt) => cmt.id !== comment.id);

      if (thread.comments.length === 0) {
        thread.dispose();
      }
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("auditor.deleteNote", (thread) => {
      console.log("----> deleteComment");
      thread.dispose();
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("auditor.cancelsaveNote", (comment) => {
      console.log("----> cancelsaveNote");
      if (!comment.parent) {
        return;
      }

      comment.parent.comments = comment.parent.comments.map((cmt) => {
        if (cmt.id === comment.id) {
          cmt.body = cmt.savedBody;
          cmt.mode = vscode.CommentMode.Preview;
        }

        return cmt;
      });
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("auditor.saveNote", (comment) => {
      console.log("----> saveNote");
      if (!comment.parent) {
        return;
      }

      comment.parent.comments = comment.parent.comments.map((cmt) => {
        if (cmt.id === comment.id) {
          cmt.savedBody = cmt.body;
          cmt.mode = vscode.CommentMode.Preview;
        }

        return cmt;
      });
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("auditor.editNote", (comment) => {
      console.log("----> editNote");
      if (!comment.parent) {
        return;
      }

      comment.parent.comments = comment.parent.comments.map((cmt) => {
        if (cmt.id === comment.id) {
          cmt.mode = vscode.CommentMode.Editing;
        }

        return cmt;
      });
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("auditor.dispose", () => {
      console.log("----> disposeNote");
      commentController.dispose();
    })
  );

  function replyNote(reply) {
    const thread = reply.thread;
    const newComment = new NoteComment(
      reply.text,
      vscode.CommentMode.Preview,
      { name: "vscode" },
      thread,
      thread.comments.length ? "canDelete" : undefined
    );
    if (thread.contextValue === "draft") {
      newComment.label = "pending";
    }

    thread.comments = [...thread.comments, newComment];
  }

  // -------------------------- Line review handling
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
  // vscode.window.onDidChangeActiveTextEditor(async (event) => {
  //   if (event != undefined) {
  //     const fileName = event.document.fileName;
  //     const state = await getReviewState(fileName);
  //     showReviewState(state);
  //   }
  // });

  // // duplicate code: run the above on the first activation as well
  // let activeEditor = vscode.window.activeTextEditor;
  // if (activeEditor) {
  //   const fileName = activeEditor.document.fileName;
  //   getReviewState(fileName).then((state) => {
  //     showReviewState(state);
  //   });
  // }
}

// This method is called when your extension is deactivated
function deactivate() {}

module.exports = {
  activate,
  deactivate,
};
