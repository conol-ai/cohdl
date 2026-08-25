import React, { useEffect, useRef, useState } from "react";
import { Link } from "@tanstack/react-router";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  ApiError,
  isUnauthorized,
  post,
  put,
  useAdminComponentRequests,
  useConfig,
  useMe,
  type ComponentRequestResponse,
  type ComponentRequestSort,
  type ComponentRequestStatus,
} from "./api";
import { Icon, LoadingRows, StatePanel, formatDate } from "./components";
import { recaptchaToken } from "./recaptcha";

interface RequestValues {
  manufacturer: string;
  part_number: string;
  datasheet_url: string;
  description: string;
}

type RequestErrors = Partial<Record<keyof RequestValues | "form", string>>;

const EMPTY_VALUES: RequestValues = {
  manufacturer: "",
  part_number: "",
  datasheet_url: "",
  description: "",
};

function scalarLength(value: string): number {
  return [...value].length;
}

function validate(values: RequestValues): RequestErrors {
  const errors: RequestErrors = {};
  const manufacturer = values.manufacturer.trim();
  const partNumber = values.part_number.trim();
  if (!manufacturer || scalarLength(manufacturer) > 128) {
    errors.manufacturer = "Enter a manufacturer name of 128 characters or fewer.";
  }
  if (!partNumber || scalarLength(partNumber) > 128) {
    errors.part_number = "Enter a part number of 128 characters or fewer.";
  }
  try {
    const url = new URL(values.datasheet_url.trim());
    if (url.protocol !== "https:" || url.username || url.password || !url.hostname) {
      throw new Error("invalid URL");
    }
  } catch {
    errors.datasheet_url = "Enter a complete HTTPS datasheet or product-page URL.";
  }
  if (scalarLength(values.description.trim()) > 2000) {
    errors.description = "Keep the description to 2,000 characters or fewer.";
  }
  return errors;
}

function focusFirstError(errors: RequestErrors): void {
  for (const name of ["manufacturer", "part_number", "datasheet_url", "description"] as const) {
    if (errors[name]) {
      document.getElementById(`component-request-${name}`)?.focus();
      return;
    }
  }
}

function errorMessage(error: unknown): string {
  if (!(error instanceof ApiError)) {
    return "Could not submit request. Your details are still here. Please try again.";
  }
  if (error.status === 403) return "Could not verify this request. Please try submitting again.";
  if (error.status === 429) return "Too many requests. Please wait and try again.";
  if (error.status === 503) return "Component requests are temporarily unavailable.";
  if (error.status >= 500) {
    return "Could not submit request. Your details are still here. Please try again.";
  }
  return error.message;
}

export function ComponentRequestPage({ initialPart = "" }: { initialPart?: string }) {
  const [values, setValues] = useState<RequestValues>({
    ...EMPTY_VALUES,
    part_number: initialPart,
  });
  const [errors, setErrors] = useState<RequestErrors>({});
  const [result, setResult] = useState<
    (ComponentRequestResponse & { manufacturer: string; part_number: string }) | null
  >(null);
  const success = useRef<HTMLDivElement>(null);
  const config = useConfig();

  useEffect(() => {
    setValues((current) =>
      current.part_number || !initialPart ? current : { ...current, part_number: initialPart },
    );
  }, [initialPart]);

  const submission = useMutation({
    mutationFn: async (submitted: RequestValues) => {
      let recaptcha: string | undefined;
      try {
        recaptcha = await recaptchaToken(
          config.data?.recaptcha_site_key,
          "component_request",
        );
      } catch {
        throw new ApiError("could not load request verification", 403);
      }
      return post<ComponentRequestResponse>("/api/component-requests", {
        manufacturer: submitted.manufacturer,
        part_number: submitted.part_number,
        datasheet_url: submitted.datasheet_url,
        description: submitted.description || undefined,
        recaptcha,
      });
    },
    onSuccess: (response, submitted) => {
      setErrors({});
      setResult({
        ...response,
        manufacturer: submitted.manufacturer.trim(),
        part_number: submitted.part_number.trim(),
      });
      requestAnimationFrame(() => success.current?.focus());
    },
    onError: (error) => {
      if (error instanceof ApiError && error.fields) {
        const fieldErrors = error.fields as RequestErrors;
        setErrors(fieldErrors);
        requestAnimationFrame(() => focusFirstError(fieldErrors));
      }
    },
  });

  if (result) {
    return (
      <div className="request-page">
        <div className="page-heading">
          <p className="eyebrow">Registry coverage</p>
          <h1>Request a component</h1>
        </div>
        <div ref={success} tabIndex={-1} className="request-success-focus">
          <StatePanel
            tone="success"
            icon="check"
            title={result.duplicate ? "Already requested" : "Request received"}
            action={
              <div className="state-action-row">
                <button
                  type="button"
                  className="button button-primary"
                  onClick={() => {
                    setValues(EMPTY_VALUES);
                    setResult(null);
                    submission.reset();
                    requestAnimationFrame(() =>
                      document.getElementById("component-request-manufacturer")?.focus(),
                    );
                  }}
                >
                  Request another component
                </button>
                <Link className="button button-secondary" to="/packages">
                  Back to packages
                </Link>
              </div>
            }
          >
            {result.duplicate
              ? `We added your request to the existing queue for ${result.manufacturer} ${result.part_number}.`
              : `Thanks — our library team will review ${result.manufacturer} ${result.part_number} for registry coverage.`}
          </StatePanel>
        </div>
      </div>
    );
  }

  const unavailable = config.data && !config.data.component_requests_enabled;
  return (
    <div className="request-page">
      <div className="page-heading">
        <p className="eyebrow">Registry coverage</p>
        <h1>Request a component</h1>
        <p>
          Can’t find the part you need? Share its manufacturer, exact part number, and datasheet.
          The CoHDL library team will review it for registry coverage.
        </p>
      </div>

      {unavailable ? (
        <StatePanel tone="error" title="Component requests are temporarily unavailable">
          Please try again later. You can continue browsing the published package catalogue.
        </StatePanel>
      ) : (
        <section className="content-panel request-form-panel" aria-labelledby="request-form-title">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">Library request</p>
              <h2 id="request-form-title">Tell us what’s missing</h2>
            </div>
            <span className="request-required-note">Required unless marked optional</span>
          </div>
          <form
            className="component-request-form"
            aria-busy={submission.isPending}
            noValidate
            onSubmit={(event) => {
              event.preventDefault();
              const nextErrors = validate(values);
              setErrors(nextErrors);
              submission.reset();
              if (Object.keys(nextErrors).length > 0) {
                requestAnimationFrame(() => focusFirstError(nextErrors));
                return;
              }
              submission.mutate(values);
            }}
          >
            <div className="request-field-grid">
              <div className="request-field">
                <label htmlFor="component-request-manufacturer">Manufacturer</label>
                <input
                  id="component-request-manufacturer"
                  autoComplete="organization"
                  required
                  aria-invalid={Boolean(errors.manufacturer)}
                  aria-describedby="component-request-manufacturer-help"
                  placeholder="Texas Instruments"
                  value={values.manufacturer}
                  onChange={(event) =>
                    setValues((current) => ({ ...current, manufacturer: event.target.value }))
                  }
                />
                <p
                  id="component-request-manufacturer-help"
                  className={errors.manufacturer ? "field-error" : "field-help"}
                >
                  {errors.manufacturer ?? "Use the manufacturer name shown on the datasheet."}
                </p>
              </div>
              <div className="request-field">
                <label htmlFor="component-request-part_number">Part number</label>
                <input
                  id="component-request-part_number"
                  autoComplete="off"
                  spellCheck={false}
                  required
                  aria-invalid={Boolean(errors.part_number)}
                  aria-describedby="component-request-part-number-help"
                  placeholder="TPS63070RNMR"
                  value={values.part_number}
                  onChange={(event) =>
                    setValues((current) => ({ ...current, part_number: event.target.value }))
                  }
                />
                <p
                  id="component-request-part-number-help"
                  className={errors.part_number ? "field-error" : "field-help"}
                >
                  {errors.part_number ??
                    "Include the full ordering code when package or grade matters."}
                </p>
              </div>
            </div>

            <div className="request-field">
              <label htmlFor="component-request-datasheet_url">Datasheet URL</label>
              <input
                id="component-request-datasheet_url"
                type="url"
                inputMode="url"
                autoComplete="url"
                required
                aria-invalid={Boolean(errors.datasheet_url)}
                aria-describedby="component-request-datasheet-help"
                placeholder="https://www.example.com/component-datasheet.pdf"
                value={values.datasheet_url}
                onChange={(event) =>
                  setValues((current) => ({ ...current, datasheet_url: event.target.value }))
                }
              />
              <p
                id="component-request-datasheet-help"
                className={errors.datasheet_url ? "field-error" : "field-help"}
              >
                {errors.datasheet_url ??
                  "Link directly to the manufacturer’s datasheet or product page."}
              </p>
            </div>

            <div className="request-field">
              <div className="request-label-row">
                <label htmlFor="component-request-description">Description (optional)</label>
                <span className={scalarLength(values.description) > 2000 ? "counter-error" : ""}>
                  {scalarLength(values.description).toLocaleString()} / 2,000
                </span>
              </div>
              <textarea
                id="component-request-description"
                rows={5}
                aria-invalid={Boolean(errors.description)}
                aria-describedby="component-request-description-help"
                placeholder="Package, variant, or intended-use details that may help us review the request."
                value={values.description}
                onChange={(event) =>
                  setValues((current) => ({ ...current, description: event.target.value }))
                }
              />
              <p
                id="component-request-description-help"
                className={errors.description ? "field-error" : "field-help"}
              >
                {errors.description ?? "Add context that will help the library team review it."}
              </p>
            </div>

            {Object.keys(errors).length > 0 && (
              <StatePanel tone="error" title="Check the highlighted fields">
                Correct the fields above, then submit the request again.
              </StatePanel>
            )}
            {submission.isError && !(
              submission.error instanceof ApiError && submission.error.fields
            ) && (
              <StatePanel tone="error" title="Could not submit request">
                {errorMessage(submission.error)}
              </StatePanel>
            )}
            <div className="request-form-actions">
              <button
                type="submit"
                className="button button-primary"
                disabled={submission.isPending || config.isPending}
              >
                {submission.isPending ? "Submitting…" : "Submit request"}
              </button>
              <Link className="button button-ghost" to="/packages">
                Back to packages
              </Link>
            </div>
          </form>
        </section>
      )}
    </div>
  );
}

function datasheetHost(value: string): string {
  try {
    return new URL(value).hostname;
  } catch {
    return "Datasheet";
  }
}

function AdminRequestDashboard() {
  const qc = useQueryClient();
  const [status, setStatus] = useState<ComponentRequestStatus | "all">("open");
  const [sort, setSort] = useState<ComponentRequestSort>("requested");
  const [searchInput, setSearchInput] = useState("");
  const [search, setSearch] = useState("");
  const [notice, setNotice] = useState<string | null>(null);
  const requests = useAdminComponentRequests(status, sort, search);
  const update = useMutation({
    mutationFn: ({ id, status: next }: { id: number; status: ComponentRequestStatus }) =>
      put<{ changed: boolean }>(`/api/admin/component-requests/${id}`, { status: next }),
    onSuccess: async (result, variables) => {
      setNotice(
        result.changed
          ? variables.status === "resolved"
            ? "Component request resolved."
            : "Component request reopened."
          : "The request already had that status.",
      );
      await qc.invalidateQueries({ queryKey: ["admin", "component-requests"] });
    },
  });

  return (
    <div className="admin-page request-admin-page">
      <div className="page-heading page-heading-row">
        <div>
          <p className="eyebrow">Official operations</p>
          <h1>Component requests</h1>
          <p>Review missing parts, measure repeat demand, and close requests once covered.</p>
        </div>
        <span className="operator-badge">
          <Icon name="shield" size={17} />
          Official session
        </span>
      </div>

      <div className="admin-local-nav" aria-label="Registry administration sections">
        <Link to="/admin">Publishers</Link>
        <Link className="is-active" to="/admin/requests" aria-current="page">
          Component requests
        </Link>
      </div>

      <section className="content-panel request-admin-toolbar" aria-label="Request queue filters">
        <form
          className="admin-search"
          role="search"
          onSubmit={(event) => {
            event.preventDefault();
            setSearch(searchInput.trim());
          }}
        >
          <div className="admin-search-box">
            <Icon name="search" size={17} />
            <input
              aria-label="Search requests by manufacturer or part number"
              placeholder="Search manufacturer or part number"
              value={searchInput}
              onChange={(event) => setSearchInput(event.target.value)}
            />
            <button type="submit">Search</button>
          </div>
        </form>
        <div className="request-admin-controls">
          <div className="filter-pills" aria-label="Filter request status">
            {(["open", "resolved", "all"] as const).map((value) => (
              <button
                type="button"
                key={value}
                aria-pressed={status === value}
                onClick={() => {
                  setStatus(value);
                  setNotice(null);
                }}
              >
                {value === "all" ? "All" : value === "open" ? "Open" : "Resolved"}
              </button>
            ))}
          </div>
          <label className="sort-control">
            <span>Sort</span>
            <select
              value={sort}
              onChange={(event) => setSort(event.target.value as ComponentRequestSort)}
            >
              <option value="requested">Most requested</option>
              <option value="newest">Newest</option>
            </select>
          </label>
        </div>
      </section>

      {notice && (
        <div className="request-admin-notice">
          <StatePanel tone="success" icon="check" title={notice} />
        </div>
      )}
      {update.isError && (
        <div className="request-admin-notice">
          <StatePanel tone="error" title="Could not update the request">
            {(update.error as Error).message}
          </StatePanel>
        </div>
      )}

      {requests.isPending ? (
        <LoadingRows count={4} />
      ) : requests.isError ? (
        <StatePanel
          tone="error"
          title="Could not load component requests"
          action={
            <button className="button button-secondary" onClick={() => requests.refetch()}>
              Try again
            </button>
          }
        />
      ) : requests.data.requests.length === 0 ? (
        <StatePanel icon="check" title={status === "open" ? "No open component requests" : "No requests match these filters"}>
          {status === "open"
            ? "New requests will appear here after submission."
            : "Try a different status or search term."}
        </StatePanel>
      ) : (
        <div className="request-admin-list">
          {requests.data.requests.map((item) => (
            <article className="content-panel request-admin-card" key={item.id}>
              <div className="request-admin-card-main">
                <div className="request-admin-title-row">
                  <div>
                    <p className="eyebrow">{item.manufacturer}</p>
                    <h2><code>{item.part_number}</code></h2>
                  </div>
                  <span className={`request-status request-status-${item.status}`}>
                    {item.status}
                  </span>
                </div>
                {item.description && <p className="request-admin-description">{item.description}</p>}
                <div className="request-admin-meta">
                  <span>{item.request_count} {item.request_count === 1 ? "request" : "requests"}</span>
                  <span>Last requested {formatDate(item.last_requested_at)}</span>
                  <a
                    href={item.datasheet_url}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    Open datasheet at {datasheetHost(item.datasheet_url)}
                    <Icon name="external" size={13} />
                  </a>
                </div>
              </div>
              <button
                type="button"
                className={item.status === "open" ? "button button-primary" : "button button-secondary"}
                disabled={update.isPending}
                onClick={() => {
                  setNotice(null);
                  update.reset();
                  update.mutate({
                    id: item.id,
                    status: item.status === "open" ? "resolved" : "open",
                  });
                }}
              >
                {item.status === "open" ? "Resolve" : "Reopen"}
              </button>
            </article>
          ))}
          {requests.data.truncated && (
            <p className="results-note">Refine the filters to see beyond the first 100 requests.</p>
          )}
        </div>
      )}
    </div>
  );
}

export function AdminComponentRequestsPage() {
  const me = useMe();
  if (me.isPending) {
    return (
      <div className="admin-page">
        <div className="page-heading">
          <p className="eyebrow">Official operations</p>
          <h1>Component requests</h1>
        </div>
        <LoadingRows count={2} />
      </div>
    );
  }
  if (me.isError && !isUnauthorized(me.error)) {
    return (
      <div className="narrow-page">
        <StatePanel tone="error" icon="shield" title="Could not verify the official session" />
      </div>
    );
  }
  if (!me.data) {
    return (
      <div className="narrow-page">
        <StatePanel
          icon="shield"
          title="Sign in with an official account"
          action={<Link className="button button-primary" to="/account">Go to sign in</Link>}
        >
          Component requests are available only through a protected official web session.
        </StatePanel>
      </div>
    );
  }
  if (!me.data.official) {
    return (
      <div className="narrow-page">
        <StatePanel tone="error" icon="shield" title="Official account access is required" />
      </div>
    );
  }
  return <AdminRequestDashboard />;
}
