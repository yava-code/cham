import { useState, useRef, useCallback, useMemo } from "react";
import Editor from "@monaco-editor/react";
import {
  parse,
  handleRequest,
  Store,
  formatError,
  type ChamApp,
  type RequestResult,
} from "../cham/interpreter";
import SAMPLES from "../cham/samples";

type Method = "GET" | "POST" | "DELETE";

interface ConsoleLine {
  text: string;
  kind: "info" | "note" | "err";
}

export default function Playground() {
  const [sampleKey, setSampleKey] = useState<string>("shawarma");
  const [code, setCode] = useState<string>(SAMPLES["shawarma"]);
  const [method, setMethod] = useState<Method>("POST");
  const [path, setPath] = useState<string>("/orders");
  const [body, setBody] = useState<string>('{"name":"Олена","phone":"500123456","item":"Гостра","size":"велика"}');
  const [response, setResponse] = useState<RequestResult | null>(null);
  const [consoleLines, setConsoleLines] = useState<ConsoleLine[]>([]);
  const storeRef = useRef<Store>(new Store());
  const appRef = useRef<{ app: ChamApp } | { error: { line: number; text: string } } | null>(null);

  // Capture console.log for new-field notes

  const runRequest = useCallback(() => {
    const parsed = parse(code);
    appRef.current = parsed;

    if ("error" in parsed) {
      const msg = formatError(parsed.error, code);
      setResponse({ status: 500, body: { error: msg }, matchedRoute: null });
      setConsoleLines([{ text: msg, kind: "err" }]);
      return;
    }

    // Fresh store on each run for predictability
    storeRef.current = new Store();

    // Capture console notes
    const notes: ConsoleLine[] = [];
    const oldLog = console.log;
    console.log = (...args: unknown[]) => {
      notes.push({ text: args.join(" "), kind: "info" as const });
    };

    try {
      const result = handleRequest(parsed.app, method, path, body, storeRef.current);
      setResponse(result);
      setConsoleLines(notes.length ? notes : [{ text: `${method} ${path} -> ${result.status}`, kind: "info" }]);
    } catch (e) {
      setResponse({ status: 500, body: { error: (e as Error).message }, matchedRoute: null });
      setConsoleLines([{ text: (e as Error).message, kind: "err" }]);
    } finally {
      console.log = oldLog;
    }
  }, [code, method, path, body]);

  const loadSample = useCallback((key: string) => {
    setSampleKey(key);
    setCode(SAMPLES[key] ?? "");
    setResponse(null);
    setConsoleLines([]);
    storeRef.current = new Store();

    // Set sensible defaults per sample
    if (key === "shawarma") {
      setMethod("POST"); setPath("/orders");
      setBody('{"name":"Олена","phone":"500123456","item":"Гостра","size":"велика"}');
    } else if (key === "lab") {
      setMethod("POST"); setPath("/bookings");
      setBody('{"device":"Robot arm","slot":"11:00","who":"Piotr"}');
    } else if (key === "notes") {
      setMethod("POST"); setPath("/notes");
      setBody('{"name":"Marta","text":"Hello world"}');
    } else if (key === "broken") {
      setMethod("POST"); setPath("/x");
      setBody('{"a":"test"}');
    }
  }, []);

  const routes = useMemo(() => {
    const parsed = parse(code);
    if ("error" in parsed) return [];
    return parsed.app.routes.map(r => `${r.method} ${r.path}`);
  }, [code]);

  const responseBody = response ? JSON.stringify(response.body, null, 2) : "";

  return (
    <section className="playground" id="playground">
      <div className="playground-inner">
        <div className="playground-header">
          <div>
            <h2>Try it in your browser</h2>
            <p>Edit the script, send a request, see the response. No install needed.</p>
          </div>
          <div className="playground-controls">
            <select className="pg-select" value={sampleKey} onChange={e => loadSample(e.target.value)}>
              <option value="shawarma">Shawarma stand</option>
              <option value="lab">Lab bookings</option>
              <option value="notes">Guest book</option>
              <option value="broken">Broken script (error demo)</option>
            </select>
          </div>
        </div>

        <div className="playground-grid">
          <div className="pg-editor-wrap">
            <Editor
              height="100%"
              defaultLanguage="plaintext"
              theme="vs-dark"
              value={code}
              onChange={v => setCode(v ?? "")}
              options={{
                fontFamily: "ui-monospace, Menlo, Consolas, monospace",
                fontSize: 14,
                lineNumbers: "on",
                minimap: { enabled: false },
                scrollBeyondLastLine: false,
                padding: { top: 16 },
                tabSize: 4,
                insertSpaces: true,
                wordWrap: "on",
                renderLineHighlight: "line",
                smoothScrolling: true,
              }}
            />
          </div>

          <div className="pg-panel">
            <div className="pg-request-bar">
              <select className="pg-method" value={method} onChange={e => setMethod(e.target.value as Method)}>
                <option>GET</option>
                <option>POST</option>
                <option>DELETE</option>
              </select>
              <input
                className="pg-path"
                value={path}
                onChange={e => setPath(e.target.value)}
                placeholder="/orders"
              />
              <button className="pg-send" onClick={runRequest}>Send</button>
            </div>

            <textarea
              className="pg-path"
              style={{ minHeight: 60, resize: "vertical", fontFamily: "var(--mono)" }}
              value={body}
              onChange={e => setBody(e.target.value)}
              placeholder='{"name": "Olena"}'
            />

            {routes.length > 0 && (
              <div style={{ fontSize: 12, color: "var(--text-muted)", fontFamily: "var(--mono)" }}>
                Routes: {routes.join(", ")}
              </div>
            )}

            <div className="pg-response">
              {response ? (
                <>
                  <div className={`status ${response.status < 400 ? "ok" : "err"}`}>
                    {response.status} {response.matchedRoute ? response.matchedRoute : "no route"}
                  </div>
                  {responseBody}
                </>
              ) : (
                <span style={{ color: "var(--text-muted)" }}>
                  Press "Send" to run the request against the script.
                </span>
              )}
            </div>

            <div className="pg-console">
              {consoleLines.length === 0 ? (
                <span style={{ color: "var(--text-muted)" }}>Console output will appear here.</span>
              ) : (
                consoleLines.map((l, i) => (
                  <div key={i} className={`line ${l.kind === "err" ? "err" : l.kind === "note" ? "note" : ""}`}>
                    {l.text}
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
