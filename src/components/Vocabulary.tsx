const VOCAB: [string, string][] = [
  ["when", "start a route"],
  ["need", "require fields"],
  ["if", "check something"],
  ["not", "flip true/false"],
  ["get", "find a record"],
  ["all", "every record"],
  ["save", "store a record"],
  ["delete", "remove records"],
  ["answer", "reply and stop"],
  ["now", "current time"],
];

export default function Vocabulary() {
  return (
    <section className="section" id="language">
      <div className="section-tag">the language</div>
      <h2>Ten words. That's the whole vocabulary.</h2>
      <p>
        Settings at the top, routes below. Indent the steps of a route, like in Python.
        Data comes in with the request, you pass it through variables, save it, and send
        something back. Every line is one step, and the steps run in the order you wrote them.
      </p>
      <div className="vocab-grid">
        {VOCAB.map(([word, desc]) => (
          <div className="vocab-cell" key={word}>
            <code>{word}</code>
            <span>{desc}</span>
          </div>
        ))}
      </div>
    </section>
  );
}
