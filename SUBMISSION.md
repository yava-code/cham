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
CORS. A route is a few lines of real code built from ten words: `when`,
`need`, `if`, `not`, `get`, `all`, `save`, `delete`, `answer`, `now`, plus
plain variables. Data goes into JSON files and picks up new
fields without migrations. Errors come back in Ukrainian, Russian or English
and say how to fix the problem.

Two apps on the same skeleton:
- a shawarma stand's order page and kitchen queue (about 25 lines of backend)
- a robotics lab equipment booking grid that refuses double bookings (about 15 lines)

**How we built it**
Rust, `tiny_http`, `serde_json`. The parser is a line-by-line match with a
Levenshtein "did you mean" for typos. Every message sits in one table in
`msg.rs`.

**Challenges**
Keeping the language small. Each keyword we added had to earn its place in
both demos or stay out. We cut a `unique` keyword once `get` plus `if` did the same job.

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
when someone posts /orders:
    need name, phone, item
    time = now
    order = save name, phone, item, time to orders
    answer "Thanks, " + name, order
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

## More hackathons

Rules checked on Oct 15, 2026. Times are Warsaw time. Cham's first commit is
from Oct 15, 2026, which falls inside every submission period below, so the
"new projects only" rules are met.

### Hyperbloom October: UI/UX and Web Design
https://hyperbloom-october.devpost.com/ · deadline Oct 19, 23:00 ·
$9,117 · ~505 registered · adult students only · new projects only, one
project per person. Judged on design, so the demo pages carry this one.

**Tagline:** Two small web apps whose backend fits on one screen.

**Description**
We designed two interfaces for people who are not tech companies: an order
page for a shawarma stand on Grochowska street in Warsaw, and a booking grid
for a university robotics lab. The shawarma page looks like a paper menu,
because that is what the regulars expect. The lab grid shows who has which
device at what hour, and a click books a free cell. Errors show up in plain
Ukrainian or English next to the button you pressed, never as a code. Both
run on Cham, a tiny backend we wrote so the logic behind each page stays
around ten readable lines.

### Build With AI: Basics
https://learn-ai-basics.devpost.com/ · deadline Oct 26, 22:00 · $2,500
($1,250 / $750 / $500) · ~3,500 registered · age of majority, open worldwide
· new projects only · needs a 1 to 3 minute video and a public repo.
Reviewers won't run the code, so the video must show it working.

**Tagline:** Ask the AI for ten lines you can check, not four hundred you
have to trust.

**Description**
Cham is a backend for learners. You describe a route in plain steps (`need`,
`save`, `answer`) and one Rust binary serves it with storage and a static
site. We built it after reading Bastani et al. (PNAS 2025): students with
unrestricted GPT-4 help did 17% worse on a later exam. With Cham you can
still ask a model to write the script, but the result is short enough to read
line by line, and when it breaks the error tells you what to change in
English, Ukrainian or Russian. For: CS freshmen and self-taught beginners.
What I learned: writing one error message in three languages makes you cut it
to the one fact that matters.

### Next Byte Hacks V5
https://next-byte-hacks-v5.devpost.com/ · deadline Oct 31, 22:00 · $100
cash · ~196 registered · 13+, worldwide, teams 1 to 4 · new projects only
· needs a description, tech list, screenshots, 2 to 5 minute video, repo.

**Tagline:** Your first backend, in ten lines you can read out loud.

**Description**
Cham is a tiny backend engine for people who just learned what an API is.
You write routes as plain steps and get a JSON API, file storage and a
static site from one binary. Make a typo and Cham tells you the line and the
word you probably meant. We tested it on a shawarma stand's order page and a
lab booking grid.

### DSH Hacks V2
https://dsh-hacks-v2.devpost.com/ · rules say submissions close Nov 8; the
header says Nov 14, 23:45 PST (Nov 15, 08:45). Aim for Nov 8 · non-cash
prizes · ~495 registered · 13+ and enrolled at an educational institution
(university counts) · AI use allowed.

**Tagline:** A backend language small enough to learn in one lesson.

**Description**
Teachers who want students to build a real web app hit the same wall: the
backend takes weeks of setup. Cham removes the setup. Students write a short
`.cham` file, run one program and get a working API with saved data. Errors
come in plain words and in the student's language. The repo has two
lesson-sized examples: a food stand and a school lab.

### Code for Humanity
https://code-for-humanity.devpost.com/ · deadline Jan 15, 2027, 13:00 ·
prizes not listed · ~585 registered · students only · project must address a
social, educational, accessibility or similar problem.

**Tagline:** Programming errors explained in the language you think in.

**Description**
Most programming tools speak English only, and their errors assume you
already know the jargon. For a Ukrainian student in Poland that means two
barriers before the first working app. Cham is a small backend tool whose
every message, from a typo in your script to a missing form field, comes in
Ukrainian, Russian or English and says what to change. The language has
fourteen keywords, so a beginner can read the whole program. It is open
source under MIT, and adding a language means filling one table.
