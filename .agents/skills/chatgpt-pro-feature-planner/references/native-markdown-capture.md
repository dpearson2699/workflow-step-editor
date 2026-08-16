# Native Markdown capture

Read this reference completely when a current typed
`generation_completed` result authorizes native Markdown capture. First apply
the planner entrypoint's automatic exact-conversation tab reacquisition when
the prior tab handle no longer exists. Then remain on that attempt's canonical
ChatGPT conversation. Do not reload, navigate away, or send. The supported
capture path observes ChatGPT's native Copy producer payload before the in-app
Browser's isolated clipboard boundary. Harness note: the page-realm evaluate
expression and clipboard wrap below run through the running harness browser's
JavaScript-evaluation capability; the forbidden-substitute list is unchanged
in both harnesses:

1. Take a fresh Browser DOM snapshot, confirm generation is terminal, and
   resolve exactly one accessible `Copy response` button for the completed
   assistant turn. A missing or ambiguous control is a capture failure.
2. Obtain the tab's supported `cdp` capability and create an unpredictable
   task-local state key. Use `cdp.send("Runtime.evaluate", ...)` once with
   `awaitPromise: true` and `returnByValue: true` to run the following
   self-contained page-realm expression. Inject the state key and a bounded
   timeout as JSON data; do not interpolate executable text into the expression.
   The page realm, not a later host command, owns installation, response-handler
   invocation, capture waiting, and cleanup:

~~~js
// NATIVE_CAPTURE_PAGE_FUNCTION_START
async ({ stateKey, timeoutMs }) => {
  const clipboard = navigator.clipboard;
  const names = ["write", "writeText"];
  const originals = Object.fromEntries(
    names.map((name) => [name, clipboard[name]])
  );
  const descriptors = Object.fromEntries(
    names.map((name) => [
      name,
      Object.getOwnPropertyDescriptor(clipboard, name)
    ])
  );
  const state = { calls: [], errors: [], pending: [] };
  const cleanup = () => {
    const errors = [];
    for (const name of names) {
      try {
        if (descriptors[name] === undefined) {
          delete clipboard[name];
        } else {
          Object.defineProperty(clipboard, name, descriptors[name]);
        }
      } catch (error) {
        errors.push(`${name}: ${String(error)}`);
      }
    }
    try {
      delete globalThis[stateKey];
    } catch (error) {
      errors.push(`state: ${String(error)}`);
    }
    for (const name of names) {
      if (clipboard[name] !== originals[name]) {
        errors.push(`${name}: original method was not restored`);
      }
    }
    if (Object.prototype.hasOwnProperty.call(globalThis, stateKey)) {
      errors.push("state: temporary state was not removed");
    }
    if (errors.length > 0) {
      throw new AggregateError(errors, "native capture cleanup failed");
    }
  };
  const record = (pending) => {
    const observed = Promise.resolve(pending).catch((error) => {
      state.errors.push(String(error));
    });
    state.pending.push(observed);
  };

  let extractionTimer;
  let result;
  try {
    Object.defineProperty(globalThis, stateKey, {
      configurable: true,
      value: state
    });
    Object.defineProperty(clipboard, "writeText", {
      configurable: true,
      value: function (text) {
        record(Promise.resolve().then(() => {
          if (typeof text !== "string") {
            throw new TypeError("writeText payload is not a string");
          }
          state.calls.push({ html: [], plain: [text] });
        }));
        return Reflect.apply(originals.writeText, this, [text]);
      },
      writable: true
    });
    Object.defineProperty(clipboard, "write", {
      configurable: true,
      value: function (items) {
        record((async () => {
          const representations = { html: [], plain: [] };
          for (const item of items) {
            for (const type of item.types) {
              if (type !== "text/plain" && type !== "text/html") {
                continue;
              }
              const text = await (await item.getType(type)).text();
              representations[
                type === "text/plain" ? "plain" : "html"
              ].push(text);
            }
          }
          state.calls.push(representations);
        })());
        return Reflect.apply(originals.write, this, [items]);
      },
      writable: true
    });

    const buttons = document.querySelectorAll(
      'button[data-testid="copy-turn-action-button"]' +
      '[aria-label="Copy response"]'
    );
    if (buttons.length !== 1) {
      throw new Error(`expected one Copy response button, got ${buttons.length}`);
    }
    buttons[0].click();

    const deadline = performance.now() + timeoutMs;
    while (state.pending.length === 0 && performance.now() < deadline) {
      await new Promise((resolve) => setTimeout(resolve, 0));
    }
    if (state.pending.length === 0) {
      throw new Error("native Copy handler made no clipboard call");
    }
    let observedPendingCount = 0;
    while (true) {
      if (state.pending.length > observedPendingCount) {
        const batch = state.pending.slice(observedPendingCount);
        observedPendingCount = state.pending.length;
        const remaining = Math.max(0, deadline - performance.now());
        await Promise.race([
          Promise.all(batch),
          new Promise((_, reject) => {
            extractionTimer = setTimeout(
              () => reject(new Error("clipboard extraction timed out")),
              remaining
            );
          })
        ]);
        clearTimeout(extractionTimer);
        extractionTimer = undefined;
        continue;
      }
      const now = performance.now();
      if (now >= deadline) {
        break;
      }
      await new Promise((resolve) => setTimeout(
        resolve,
        Math.min(1, deadline - now)
      ));
    }
    if (state.errors.length > 0) {
      throw new AggregateError(state.errors, "clipboard extraction failed");
    }
    if (state.calls.length !== 1) {
      throw new Error(`expected one clipboard call, got ${state.calls.length}`);
    }
    const [{ html, plain }] = state.calls;
    if (plain.length !== 1 || plain[0].length === 0) {
      throw new Error("expected one non-empty text/plain payload");
    }
    result = {
      callCount: 1,
      textHtml: html.length === 1 ? html[0] : null,
      textPlain: plain[0]
    };
  } finally {
    if (extractionTimer !== undefined) {
      clearTimeout(extractionTimer);
    }
    cleanup();
  }
  return { ...result, cleanupVerified: true };
}
// NATIVE_CAPTURE_PAGE_FUNCTION_END
~~~

Set `Runtime.evaluate.expression` to the exact invocation
`(${pageFunctionSource})(${captureArgumentsJSON})`, where
`pageFunctionSource` is the unchanged function between the markers above and
`captureArgumentsJSON` is produced only by
`JSON.stringify({ stateKey, timeoutMs })`. This invokes the async function in
the page realm; returning the function object without the JSON argument call is
not capture. Send that expression once with `awaitPromise: true` and
`returnByValue: true`.

3. The `writeText(text)` wrapper records the exact string as one call and
   invokes the original method with its original receiver. The `write(items)`
   wrapper enumerates every `ClipboardItem.types` entry and reads `text/plain`
   and optional `text/html` blobs directly with `getType(type)` and
   `Blob.text()`. Extraction starts without delaying the original clipboard
   invocation, and every extraction error remains part of the page-owned
   result. Do not transform either representation.
4. After the first native call, keep observation active through the full
   bounded capture window so a delayed clipboard call cannot escape.
   Require the single `Runtime.evaluate` result to report `cleanupVerified:
   true`, exactly one clipboard call, no extraction error, and exactly one
   non-empty `text/plain` string. Optional `text/html` is non-authoritative
   diagnostic evidence only. The expression re-resolves one
   `button[data-testid="copy-turn-action-button"][aria-label="Copy response"]`
   and invokes its existing `HTMLElement.click()` handler in the page realm.
   Do not use a Playwright, coordinate, or DOM CUA click: ChatGPT's response
   toolbar can remain `pointer-events: none` in background automation even when
   the accessible control exists.
5. Encode the exact `text/plain` string as UTF-8, write those bytes directly to
   the task-local `.md` file, and compute SHA-256 from those same bytes. Record
   the byte count and digest. In the same task, write one
   `pro-lifecycle-capture-result` containing the exact work, consultation,
   attempt, task, canonical conversation URL, committed binding digest, observed
   generation ID, captured response digest, and observation time. These fields
   correlate the native payload with the observed generation; they are not an
   authenticated Browser receipt. Preserve headings, tables, fenced code,
   links, and line endings as supplied; do not normalize or reconstruct Markdown.
6. The page-realm `finally` restores both original own-property descriptors, or
   deletes the temporary own properties when the methods were originally
   inherited, and removes all page-scoped capture state. It attempts every
   cleanup action before reporting any error and verifies both original
   functions and state removal. Keep a host-side `finally` only as a
   best-effort fallback. If the CDP result cannot verify cleanup, close or
   otherwise discard that tab before publishing the blocker or handing off; a
   live instrumented page realm must never survive the attempt.

Do not use `innerText`, `textContent`, copied HTML, HTML-to-Markdown conversion,
`tab.clipboard`, `pbpaste`, the macOS clipboard, Chrome, Computer Use, or a
human relay. If CDP is unavailable, instrumentation cannot be installed or
cleanup cannot be verified, terminal state or the Copy control is missing or
ambiguous, the captured call count is not exactly one, or exact non-empty
`text/plain` bytes are unavailable, publish no positive evidence and close the
attempt as `BLK-PRO-UNCERTAIN-SEND` through the existing capture-failure path.
Never resend, substitute a conversation, or substitute another representation.
Do not publish that blocker or hand off until cleanup is verified or the tab
has been discarded.
