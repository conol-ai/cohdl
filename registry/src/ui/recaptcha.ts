// reCAPTCHA v3 is loaded only when a form that needs it is submitted.

declare global {
  interface Window {
    grecaptcha?: {
      ready(cb: () => void): void;
      execute(siteKey: string, opts: { action: string }): Promise<string>;
    };
  }
}

let loaded: Promise<void> | null = null;

function loadRecaptcha(siteKey: string): Promise<void> {
  if (!loaded) {
    loaded = new Promise((resolve, reject) => {
      const script = document.createElement("script");
      script.src = `https://www.google.com/recaptcha/api.js?render=${encodeURIComponent(siteKey)}`;
      script.onload = () => window.grecaptcha!.ready(resolve);
      script.onerror = () => {
        script.remove();
        reject(new Error("could not load reCAPTCHA"));
      };
      document.head.appendChild(script);
    });
    loaded.catch(() => {
      loaded = null;
    });
  }
  return loaded;
}

export async function recaptchaToken(
  siteKey: string | null | undefined,
  action: string,
): Promise<string | undefined> {
  if (!siteKey) return undefined;
  await loadRecaptcha(siteKey);
  return window.grecaptcha!.execute(siteKey, { action });
}
