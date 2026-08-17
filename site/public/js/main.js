// Progressive enhancement for the waitlist form. Without this file the form
// still posts normally and the Worker answers with a redirect carrying the
// outcome in the query string; with it, the submit happens in place.

const form = document.getElementById("signup-form");
const status = document.getElementById("signup-status");
const input = document.getElementById("email");
const button = form?.querySelector(".signup-button");

/** Messages for the no-JS redirect path, keyed by the Worker's error codes. */
const CODE_MESSAGES = {
  invalid_email: "That doesn't look like an email address.",
  rate_limited: "Too many signups from here. Try again later.",
  storage: "Couldn't save that just now. Please try again.",
  cross_origin: "That request didn't come from this page.",
};

function show(state, message) {
  if (!status) return;
  status.dataset.state = state;
  status.textContent = message;
}

// Surface the outcome of a no-JS submit, then drop the query string so a
// reload doesn't repeat the message.
const params = new URLSearchParams(window.location.search);
if (params.has("joined")) {
  show("ok", "You're on the list. We'll email you when CoHDL opens up.");
} else if (params.has("error")) {
  show("error", CODE_MESSAGES[params.get("error")] ?? "Something went wrong. Please try again.");
}
if (params.has("joined") || params.has("error")) {
  params.delete("joined");
  params.delete("error");
  const query = params.toString();
  window.history.replaceState({}, "", window.location.pathname + (query ? `?${query}` : ""));
}

form?.addEventListener("submit", async (event) => {
  event.preventDefault();

  const email = (input?.value ?? "").trim();
  if (!email) {
    show("error", "Please enter an email address.");
    input?.focus();
    return;
  }

  if (button) button.disabled = true;
  show("", "Adding you…");

  try {
    const response = await fetch("/api/waitlist", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        email,
        // Carried through from ?ref=… so we can tell where signups came from.
        source: new URLSearchParams(window.location.search).get("ref") ?? "",
        company: form.querySelector("#company")?.value ?? "",
      }),
    });

    const body = await response.json().catch(() => ({}));

    if (response.ok && body.ok) {
      show("ok", body.message ?? "You're on the list.");
      form.reset();
    } else {
      show("error", body.message ?? "Something went wrong. Please try again.");
    }
  } catch {
    show("error", "Network trouble — check your connection and try again.");
  } finally {
    if (button) button.disabled = false;
  }
});
