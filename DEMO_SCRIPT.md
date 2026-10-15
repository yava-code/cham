# Demo video: 4 minutes

## Before recording

```sh
cd cham
cargo build --release
rm -rf examples/*/data                 # start with empty data
./target/release/cham examples/shawarma     # terminal 1, :8080
./target/release/cham examples/lab          # terminal 2, :8081
```

Have open: an editor with `examples/shawarma/app.cham`, a browser with tabs on
`localhost:8080` and `localhost:8081`, a third terminal for `curl`.
Make the terminal font large. Close notifications.

## Plan

**0:00–0:30 · Problem.** Face or slide. "Ask a chatbot for a backend and you
get 400 lines and five config files. When it breaks, you paste the error
back. Studies show students who lean on AI like that learn less." Show the
README research section for two seconds.

**0:30–1:15 · The script.** Editor on `shawarma/app.cham`. Read the
`when someone posts /orders` route line by line: need, the price `if`, save, answer. "That's
the whole backend. No config, no migrations." Switch to terminal 1 and show
the startup line.

**1:15–2:00 · Shawarma app.** Browser :8080. Pick "Гостра", type a name and
phone, order. Point at the green message and the queue below. Submit again
with the phone empty: the red Ukrainian error appears. Point at terminal 1:
same message in the log.

**2:00–2:30 · Schema on the fly.** Terminal 3:
```sh
curl -H 'Content-Type: application/json' localhost:8080/orders \
  -d '{"name":"Ivan","phone":"500","item":"Фалафель","note":"без цибулі"}'
```
Terminal 1 prints that `orders` got a new field `note`. Open
`examples/shawarma/data/orders.json` to show the plain file.

**2:30–3:15 · Second domain.** Browser :8081, lab grid. Type a name, book
Robot arm 11:00. Change the name, click a free cell, then try to book Robot
arm 11:00 again through curl:
```sh
curl -H 'Content-Type: application/json' localhost:8081/bookings \
  -d '{"device":"Robot arm","slot":"11:00","who":"Piotr"}'
```
409 with a readable reason. Show `lab/app.cham`: about 15 lines, one new keyword
(`get` + `if`).

**3:15–3:45 · Three languages.** Stop terminal 2. Break line 3 of a copy of
the script (`sav` instead of `save`), run it three times:
```sh
printf 'when someone posts /x:\n    need a\n    sav a to things\n' > /tmp/bad.cham
for l in en uk ru; do CHAM_LANG=$l ./target/release/cham /tmp/bad.cham; done
```
Three "did you mean save?" messages with the line number.

**3:45–4:00 · Close.** "One binary, about 1,000 lines of Rust, two apps on the
same core. The student writes the logic and understands every step." Show
the repo URL.

## Kiroween cut (Skeleton Crew)
Spend more of 1:15–3:15 on "same skeleton, two apps": put both `app.cham`
files side by side for five seconds. Mention which parts of the work you did
in Kiro, once the rules say what they expect.
