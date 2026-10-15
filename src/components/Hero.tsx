const CODE_SAMPLE = `when someone posts /orders:
    need name, phone, item
    time = now
    order = save name, phone, item, time to orders
    answer "Thanks, " + name, order`;

function highlightCham(src: string): string {
  const keywords = new Set(["when", "need", "if", "not", "get", "all", "save", "delete", "answer", "now", "someone", "posts", "opens", "deletes", "to"]);
  return src
    .split("\n")
    .map(line => {
      // comments
      const commentIdx = findComment(line);
      let code = line;
      let comment = "";
      if (commentIdx >= 0) {
        code = line.slice(0, commentIdx);
        comment = line.slice(commentIdx);
      }

      // strings
      let html = "";
      let rest = code;
      while (true) {
        const q = rest.indexOf('"');
        if (q < 0) { html += tokenize(rest, keywords); break; }
        html += tokenize(rest.slice(0, q), keywords);
        rest = rest.slice(q);
        const end = rest.indexOf('"', 1);
        if (end < 0) { html += escape(rest); break; }
        html += `<span class="st">${escape(rest.slice(0, end + 1))}</span>`;
        rest = rest.slice(end + 1);
      }

      if (comment) html += `<span class="cm">${escape(comment)}</span>`;
      return html;
    })
    .join("\n");
}

function findComment(line: string): number {
  let quoted = false;
  for (let i = 0; i < line.length; i++) {
    if (line[i] === '"') quoted = !quoted;
    if (line[i] === "#" && !quoted) return i;
  }
  return -1;
}

function tokenize(s: string, keywords: Set<string>): string {
  if (!s) return "";
  return s.replace(/[a-zA-Z_][a-zA-Z0-9_.]*/g, word => {
    if (keywords.has(word)) return `<span class="kw">${word}</span>`;
    return word;
  });
}

function escape(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

export default function Hero() {
  return (
    <section className="hero">
      <div className="hero-left">
        <h1>A backend you can read in one breath.</h1>
        <p>
          Cham runs a small JSON backend from a text file. One binary, about 1 MB,
          ten words. No config, no migrations, no node_modules. Write routes as
          plain steps, and follow your data with a finger.
        </p>
        <div className="hero-buttons">
          <a className="btn-primary" href="#playground">Open the playground</a>
          <a className="btn-secondary" href="#install">How to install</a>
        </div>
      </div>
      <div className="hero-code">
        <div className="hero-code-head">
          <span className="dot" />
          app.cham — 5 lines, one endpoint
        </div>
        <pre dangerouslySetInnerHTML={{ __html: highlightCham(CODE_SAMPLE) }} />
      </div>
    </section>
  );
}
