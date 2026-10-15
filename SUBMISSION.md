# Submission texts

Drafts only. Nothing here has been submitted. Check each hackathon's rules
before pasting (see notes at the top of each section).

---

## Kiroween 2026 · Skeleton Crew

> Rules note: submissions open Oct 30, 2026, close Dec 4, 2026 2:00 PM PT.
> The overview says only projects created during the submission period are
> eligible and asks people not to start building before Oct 30. Full rules,
> including how Kiro must be used, come out Oct 30. Re-read them before
> submitting. A Kiro user ID or AWS account ID is needed to register.

**Name:** Cham

**Tagline:** A backend skeleton you can read in one breath: routes as plain
steps, two apps on top of it.

**Inspiration**
Ask a chatbot for a shop backend and you get a framework, an ORM, an env file
and a Dockerfile. Our classmates ship that and can't say where an order goes.
We wanted the opposite: the smallest skeleton that still runs real apps, with
nothing hidden.

**What it does**
Cham reads a `.cham` file and serves it as a JSON API with static files and
CORS. A route is a list of steps: `need`, `default`, `stamp`, `unique`,
`save`, `list`, `find`, `reply`. Data goes into JSON files and picks up new
fields without migrations. Errors come back in Ukrainian, Russian or English
and say how to fix the problem.

Two apps on the same skeleton:
- a shawarma stand's order page and kitchen queue (12 lines of backend)
- a robotics lab equipment booking grid that refuses double bookings (10 lines)

**How we built it**
Rust, `tiny_http`, `serde_json`. The parser is a line-by-line match with a
Levenshtein "did you mean" for typos. Every message sits in one table in
`msg.rs`.

**Challenges**
Keeping the language small. Each keyword we added had to earn its place in
both demos or stay out. `unique` made it in because the lab app is useless
without it.

**What we learned**
A good error message does more teaching than a tutorial. Writing the same
error three times in three languages forced us to make it short and concrete.

**What's next**
`update` and `delete` steps, Polish messages, a `--watch` flag that reloads
the script on save.

**Built with:** rust, tiny_http, serde_json, html, css, javascript

---

## EurekaDev 2026 · Coding Track · CS + AI / Technology

> Rules note: deadline Oct 20, 2026 5:00 PM CDT (Oct 21, 00:00 Warsaw).
> Rules say the event is open to high school students (~13–19). Confirm
> eligibility before submitting. Participants must join the official Discord.
> Cross-posting is allowed. Judging: innovation, impact, execution,
> presentation, 25% each.

**Project name:** Cham: readable backends for people learning to code

**Elevator pitch**
Cham lets a beginner build a working backend in about ten lines they can read
and explain, with error messages in their own language.

**The problem**
Beginners now ask AI tools for code and get more than they can understand.
Research backs the worry. In a field experiment with about 1,000 high school
students, unrestricted GPT-4 help raised practice scores by 48% but lowered
later exam scores by 17% (Bastani et al., PNAS 2025). Sweller's cognitive
load theory explains part of it: a novice who spends working memory on
tooling has less left for learning the concept.

**Our solution**
Cham is a single Rust binary that runs a tiny route language:

```
post /orders
  need name, phone, item
  stamp
  save orders
  reply 201 "Thanks, {name}!"
```

Each line is one step, run in order. Storage adapts to new fields by itself.
When something breaks, Cham names the line or field and suggests a fix in
English, Ukrainian or Russian, instead of printing a stack trace.

**Demo apps**
1. Order page for a local shawarma stand with a live kitchen queue.
2. Equipment booking for a university lab that rejects double bookings.

**Where AI fits**
Cham does not fight AI tools. It gives them a smaller target. A model can
write a `.cham` file, and the student can still check every line of it.
The README cites the studies (Sweller 1988; Bastani et al. 2025; Lee et al.,
CHI 2025; Becker et al., METR 2025) with links.

**Tech:** Rust, tiny_http, serde_json, vanilla HTML/CSS/JS.

**Repo:** https://github.com/yava-code/cham

---

## Extra hackathons (unchecked rules: read before entering)

### Next Byte Hacks V5
https://next-byte-hacks-v5.devpost.com/ · online · deadline Oct 31, 2026 ·
$100 cash · ~196 registered · themes: beginner friendly, open ended.

**Tagline:** Your first backend, in ten lines you can read out loud.

**Description**
Cham is a tiny backend engine for people who just learned what an API is.
You write routes as plain steps (`need`, `save`, `reply`) and get a JSON API,
file storage and a static site from one binary. When you make a mistake, Cham
tells you which line and what to type instead, in English, Ukrainian or
Russian. We built two apps to test it: a shawarma stand's order page and a
lab booking grid.

### DSH Hacks V2
https://dsh-hacks-v2.devpost.com/ · online · deadline Nov 15, 2026 ·
non-cash prizes ("$100k+" in title) · ~495 registered · open ended.
STEMise runs it; check the age limits first.

**Tagline:** A backend language small enough to learn in one lesson.

**Description**
Teachers who want students to build a real web app hit the same wall: the
backend part takes weeks of setup. Cham removes the setup. Students write a
short `.cham` file, run one program, and have a working API with saved data.
Errors are explained in plain words and in the student's language. The repo
includes a lesson-sized example for a food stand and one for a school lab.

### Build With AI: Basics
https://learn-ai-basics.devpost.com/ · online · deadline Oct 26, 2026 ·
$2,500 · ~3,500 registered · themes: AI/ML, education.
Check whether the rules require an AI feature inside the project; Cham has
none, so this one only fits if "AI-assisted building" counts.

**Tagline:** Learn the basics with AI without handing it the steering wheel.

**Description**
Studies show students who copy AI answers learn less (Bastani et al., PNAS
2025). Cham keeps AI help but shrinks what it produces: a model can write a
ten-line `.cham` file and the learner can check each line, run it and read
the errors in their own language. We tested it on two small apps.
