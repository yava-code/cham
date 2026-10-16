# Demo video: 4 minutes

Spoken lines are in quotes. Overlays are files in `video/`: open them in a
browser, capture with OBS, press **R** to replay. Cut them in over the screen
recording when editing.

## Setup (before recording)

```sh
cd cham
cargo build --release
rm -rf examples/*/data
./target/release/cham examples/shawarma     # terminal 1, :8080
./target/release/cham examples/lab          # terminal 2, :8081
printf 'when someone posts /x:\n    need a\n    sav a to things\n' > /tmp/bad.cham
```

Screen layout: editor with `examples/shawarma/app.cham`, browser tabs on
`localhost:8080` and `localhost:8081`, a third terminal for curl. Large
terminal font, notifications off. Run `cham examples/shawarma` once more
before you record the console, so the route list and cheat sheet appear fresh.

## Timeline

**0:00 to 0:12 · Intro** · overlay `01-intro.html`
"Hi, I'm Varfolomiy, a CS student in Warsaw. This is Cham, a small backend you can read in one breath."

**0:12 to 0:45 · Problem** · overlay `02-problem.html`
"Ask a chatbot for a backend and you get hundreds of lines, a config file and a Dockerfile. It runs. Then it breaks, and you paste the error back into the chat. A study of almost a thousand students found that free GPT-4 help raised practice grades by 48 percent, and then lowered exam grades by 17 percent once the help was gone. METR measured experienced developers with AI tools: they were 19 percent slower, and still thought they were faster. I wanted something small enough that you keep the whole picture in your head."

**0:45 to 1:25 · The code** · editor on `examples/shawarma/app.cham`
"This is the whole backend for a shawarma stand. Settings at the top. Then routes. This one starts with 'when someone posts orders'."
Point at each line as you say it:
"Need: stop if name, phone, item or size is missing. A variable for the price. An if, for the large size. Save writes the order to a file. Answer sends the reply."

**1:25 to 1:50 · Data flow** · overlay `03-dataflow.html`
"Data comes in with the request, gets checked, gets stored, and goes back out. You can follow it with your finger."

**1:50 to 2:35 · Shawarma app** · browser `localhost:8080`
"Here's the page. I pick the spicy one, enter a name and a phone, and order."
Order a large one. Point at the green message and the total.
"The total is 34 złoty, because the size was large. The kitchen queue is below."
Submit again with the phone empty.
"Phone missing. The error comes in Ukrainian and says which fields the route needs."
Switch to terminal 1:
"Same message in the console."

**2:35 to 2:55 · Delete** · terminal 3
```sh
curl -X DELETE localhost:8080/orders/1
curl -X DELETE localhost:8080/orders/1
```
"Delete works. Delete it twice, and the second time the if-check answers 'no such order'."
Reload the page: the order is gone from the queue.

**2:55 to 3:25 · Second domain** · browser `localhost:8081`, then `examples/lab/app.cham`
"Same engine, different job. A booking grid for a robotics lab. I book the robot arm at 11."
Click the cell. Then:
```sh
curl -H 'Content-Type: application/json' localhost:8081/bookings \
  -d '{"device":"Robot arm","slot":"11:00","who":"Piotr"}'
```
"A second person tries the same slot. Refused, and the reason names who has it. That's one get and one if."

**3:25 to 3:50 · Errors in three languages** · overlay `04-errors.html`, then terminal
"A typo in the script. Cham gives the line, the word and a guess."
```sh
for l in en uk ru; do CHAM_LANG=$l ./target/release/cham /tmp/bad.cham; done
```
"English, Ukrainian, Russian. Adding a language is one more column in one file."

**3:50 to 4:00 · Vocabulary and outro** · overlays `05-vocabulary.html` (4 s), then `06-outro.html`
"That's all of it: ten words. The code is on GitHub. Build it, run `cham new myapp`, and edit the file."

## Notes for editing

- Total spoken text is about 450 words. At a calm pace it fits 4:00. Trim the problem section first if you run long.
- Use `07-lower-third.html?t=Varfolomiy%20Yasenoviy&s=CS%20student%2C%20Warsaw` over the first seconds of the editor shot. Press T for transparent, or load it with `#clear`.
- For Kiroween, spend more of 1:25 to 3:25 on "one engine, two apps": put both `app.cham` files side by side for five seconds.

## Voice cut (Bolna x Symphony buildathon)

Add 30 to 40 seconds after the second domain, or replace the lab part.
Start `./target/release/cham examples/voice-shawarma` and open
`localhost:8082` in Chrome. Allow the microphone once before recording.

1. Show the 14-line `examples/voice-shawarma/app.cham`. "Same language, same ten words."
2. Press the mic and answer out loud: name, "гостра", "велика", a phone number. Let the audience hear the questions.
3. Show the `POST /orders` line the page prints, then the spoken total.
4. Show an error read aloud: press the mic and skip a question three times. For a spoken Cham error, run
   `curl -H 'Content-Type: application/json' localhost:8082/orders -d '{"name":"A"}'` and show the Ukrainian JSON error. Say that the page reads this same text with speechSynthesis when a post fails.
5. Say what is true: the speech comes from the browser, Cham only supplies the route and the message text.
