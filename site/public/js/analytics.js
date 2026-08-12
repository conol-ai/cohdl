// Google Analytics (GA4) bootstrap for cohdl.org.
//
// The standard snippet inlines this block, which forces the CSP to carry
// either 'unsafe-inline' or a sha256 of the exact bytes — and that hash then
// has to be recomputed by hand every time the snippet changes. Keeping it in
// its own file lets script-src stay 'self' plus the tag manager's own origin.
window.dataLayer = window.dataLayer || [];
function gtag() {
  dataLayer.push(arguments);
}
gtag("js", new Date());
gtag("config", "G-R73M37GXP7");
