// Shared reCAPTCHA v3 verification for browser-originated writes.

export interface RecaptchaEnv {
  RECAPTCHA_SECRET_KEY?: string;
}

export async function recaptchaOk(
  env: RecaptchaEnv,
  request: Request,
  token: string | undefined,
  action: string,
  expectedHostname?: string,
): Promise<boolean> {
  // Existing account flows deliberately remain usable in local development
  // before dashboard variables exist. Public anonymous writes apply their own
  // fail-closed configuration check before calling this helper.
  if (!env.RECAPTCHA_SECRET_KEY) return true;
  if (!token || token.length > 4096) return false;

  const form = new URLSearchParams({
    secret: env.RECAPTCHA_SECRET_KEY,
    response: token,
    remoteip: request.headers.get("CF-Connecting-IP") ?? "",
  });
  try {
    const resp = await fetch("https://www.google.com/recaptcha/api/siteverify", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: form.toString(),
      signal: AbortSignal.timeout(5000),
    });
    if (!resp.ok) return false;
    const data = (await resp.json()) as {
      success?: boolean;
      score?: number;
      action?: string;
      hostname?: string;
    };
    return (
      data.success === true &&
      data.action === action &&
      (data.score ?? 0) >= 0.5 &&
      (!expectedHostname || data.hostname === expectedHostname)
    );
  } catch {
    return false;
  }
}
