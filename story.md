## The problem
Beginners now ask AI tools for code and get more than they can understand.
Research backs the worry. In a field experiment with about 1,000 high school
students, unrestricted GPT-4 help raised practice scores by 48% but lowered
later exam scores by 17% (Bastani et al., PNAS 2025). Sweller's cognitive
load theory explains part of it: a novice who spends working memory on
tooling has less left for learning the concept.

## Our solution
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

## Demo apps
1. Order page for a local shawarma stand with a live kitchen queue.
2. Equipment booking for a university lab that rejects double bookings.

## Where AI fits
Cham does not fight AI tools. It gives them a smaller target. A model can
write a `.cham` file, and the student can still check every line of it.
The README cites the studies (Sweller 1988; Bastani et al. 2025; Lee et al.,
CHI 2025; Becker et al., METR 2025) with links.


## Built with
Rust, tiny_http, serde_json, vanilla HTML/CSS/JS.

Repo: https://github.com/yava-code/cham
