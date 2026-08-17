// Google Analytics (GA4) bootstrap for registry.cohdl.org.
//
// Kept out of index.html so the SPA's Content-Security-Policy can stay on
// script-src 'self' plus the tag manager's origin — no 'unsafe-inline', and no
// sha256 that has to be recomputed whenever the snippet changes.
//
// Route changes are not reported here: this is a client-routed SPA, so
// per-route page_views come from GA4's enhanced measurement ("History
// changes"), which is enabled by default on the data stream.
window.dataLayer = window.dataLayer || [];
function gtag() {
  dataLayer.push(arguments);
}
gtag("js", new Date());
gtag("config", "G-R73M37GXP7");
