const test = require("node:test");
const assert = require("node:assert/strict");

const {
  DESKTOP_ONLY_MESSAGE,
  createDesktopBridge,
  fillPrivateKeyPath,
  setEditorError,
} = require("../app.js");

function fakeMessageNode() {
  return {
    textContent: "",
    hidden: true,
    classList: {
      toggle(_className, force) {
        this.owner.hidden = force;
      },
      owner: null,
    },
  };
}

test("createDesktopBridge tolerates missing tauri globals and returns desktop-only errors", async () => {
  const bridge = createDesktopBridge({});

  assert.equal(bridge.isDesktop, false);
  await assert.rejects(() => bridge.invoke("load_state"), new Error(DESKTOP_ONLY_MESSAGE));
  await assert.rejects(() => bridge.pickPrivateKeyPath(), new Error(DESKTOP_ONLY_MESSAGE));
});

test("createDesktopBridge uses tauri dialog open and returns the selected key path", async () => {
  const bridge = createDesktopBridge({
    __TAURI__: {
      core: {
        invoke: async () => ({ ok: true }),
      },
      dialog: {
        open: async () => "/home/top/.ssh/id_ed25519",
      },
    },
  });

  assert.equal(await bridge.pickPrivateKeyPath(), "/home/top/.ssh/id_ed25519");
});

test("setEditorError renders drawer errors inline", () => {
  const editorError = fakeMessageNode();
  editorError.classList.owner = editorError;

  setEditorError({ editorError }, "保存失败：private_key_path is required");

  assert.equal(editorError.textContent, "保存失败：private_key_path is required");
  assert.equal(editorError.hidden, false);
});

test("fillPrivateKeyPath writes the selected path back into the input", () => {
  const input = { value: "" };

  fillPrivateKeyPath(input, "/home/top/.ssh/id_ed25519");

  assert.equal(input.value, "/home/top/.ssh/id_ed25519");
});
