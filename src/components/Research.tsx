const RESEARCH = [
  {
    title: "Unrestricted AI help raises practice scores and lowers learning.",
    body: "In a randomized trial with nearly 1,000 high school students, plain GPT-4 chat improved practice grades by 48%, but when access was removed those students scored 17% worse than students who never had it. A tutor that gave hints instead of answers removed most of the harm.",
    cite: "Bastani et al. 2025, PNAS 122(26)",
  },
  {
    title: "Novices run out of working memory before they run out of ideas.",
    body: "Sweller's cognitive load theory: a beginner juggling a router, an ORM, env files and a build tool has little capacity left to learn what a request is. Cham cuts the number of moving parts to one file and ten words.",
    cite: "Sweller 1988, Cognitive Science 12(2)",
  },
  {
    title: "Trusting the tool more means checking it less.",
    body: "A Microsoft Research and CMU survey of 319 knowledge workers found that higher confidence in GenAI is associated with less critical thinking, while higher self-confidence is associated with more.",
    cite: "Lee et al., CHI 2025",
  },
  {
    title: "People misjudge how much AI helps them.",
    body: "In METR's randomized trial, 16 experienced developers expected AI tools to cut task time by 24% and believed afterwards they had saved 20%. Measured, they took 19% longer.",
    cite: "Becker et al. 2025, arXiv:2507.09089",
  },
];

export default function Research() {
  return (
    <section className="section" id="research">
      <div className="section-tag">why it matters</div>
      <h2>The research behind the design.</h2>
      <p>
        Cham does not ban AI. A student can still ask a model to write the .cham file.
        The difference is that the result is a dozen lines they can check, not four hundred
        they have to trust.
      </p>
      {RESEARCH.map(r => (
        <div className="research-item" key={r.cite}>
          <h4>{r.title}</h4>
          <p>{r.body}</p>
          <div className="cite">{r.cite}</div>
        </div>
      ))}
    </section>
  );
}
