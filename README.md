<img src="assets/logo.png" alt="Cham logo" width="140">

# Cham

Cham runs a small JSON backend from a text file you can read top to bottom.
One binary, about 1 MB. No config, no migrations, no `node_modules`.

```
when someone posts /orders:
    need name, phone, item
    time = now
    order = save name, phone, item, time to orders
    answer "Thanks, " + name, order
```

Those five lines give you an endpoint that checks the input, timestamps it,
writes it to disk and answers. Every line is one step, and the steps run in
the order you wrote them. If you want to know what happens to a request, you
read the file.

When something goes wrong, Cham tells you in Ukrainian, Russian or English,
says which line or field broke, and how to fix it.

## Why

I study CS in Warsaw. The
pattern I keep seeing among classmates: someone asks a chatbot for "a backend for my shop", gets
400 lines of Express, Prisma, a `.env`, a Dockerfile, and can't explain where
the order goes after they press the button. When it breaks they paste the
stack trace back into the chat. The code works until the first error, and
then the student has nothing to hold on to.

Frameworks like Django or Spring are not the answer for that student either.
They are built for teams and years of maintenance, and the first hour is
spent on ceremony.

Cham sits between "write it all yourself" and "ask the AI". The language is
small enough to learn in ten minutes, so the student writes the logic and
keeps the whole picture in their head. The research below is why I think
that matters.

## Quickstart

```sh
cargo build --release
./target/release/cham new myapp     # app.cham + index.html, ready to run
./target/release/cham myapp         # open http://127.0.0.1:8080
```

On start Cham prints the routes it found and a three-line cheat sheet.
`PORT=8081` and `CHAM_LANG=uk|ru|en` override the settings in the file.

## The language

Settings go at the top, routes below. Indent the steps of a route, like in
Python. Data comes in with the request, you pass it through variables, save
it, and send something back.

```
port = 8080
lang = "uk"
price = 28

when someone posts /orders:
    need name, item, size                 # in: stop if a field is missing
    total = price                         # work: plain variables
    if size == "велика": total = price + 6
    order = save name, item, size, total to orders     # store
    answer 201 "До сплати " + total + " zł", order     # out

when someone opens /orders:
    answer all orders

when someone deletes /orders/:id:
    order = get orders id
    if not order:
        answer 404 "No order " + id
    delete orders id
    answer "Deleted " + order.item
```

The whole vocabulary:

| word | what it does |
|---|---|
| `when someone posts/opens/deletes /path:` | start a route; `:id` in the path becomes a variable |
| `need a, b` | answer 422 if any of these fields is missing or empty |
| `x = ...` | set a variable; request fields are variables already |
| `if ...:` | run the rest of the line, or the indented lines below, when true |
| `not` | flip true and false: `if not order:` |
| `save a, b to coll` | write those fields as a new record, gives it an `id`, returns it |
| `get coll a, b` | first record whose `a` and `b` equal your variables, or nothing |
| `delete coll a, b` | remove the matching records |
| `all coll` | every record in the collection |
| `answer [status] value [, data]` | reply and stop; text becomes `{"message"}`, or `{"error"}` for 400+ |
| `now` | current Unix time |

Operators: `+` adds numbers or joins text, `-`, `==`, `!=`, `>`, `<`, `>=`,
`<=`, and brackets. `order.item` reads a field of a record.

Files next to the script (or in `public/` if it exists) are served as the
website. CORS headers go out on every response.

### Storage

Each collection is a JSON array in `data/<name>.json` next to the script.
There is no schema. If a request brings a field the collection has never
seen, Cham stores it and prints a note:

```
  примітка: у "orders" з'явилось нове поле "note". Зберіг, міграція не потрібна.
```

You can open the file in any editor and see your data. Writes go through a
temp file and a rename, so a crash mid-write does not corrupt it. This is
built for hundreds or a few thousand records. Past that, use a real database.

## Two demos, two domains

**`examples/shawarma`**: order page for a shawarma stand on Grochowska. The
customer picks from the menu, leaves a name and phone; the kitchen sees the
queue under the form. Backend: 4 routes, about 25 lines, including a price check and delete.

**`examples/lab`**: equipment booking grid for a university robotics lab.
Click a free cell to book the oscilloscope for 11:00; the second person who
tries the same slot gets a 409 with a readable reason. Backend: 3 routes,
about 15 lines. Double booking is one `get` and one `if`.

```sh
./target/release/cham examples/shawarma     # :8080
./target/release/cham examples/lab          # :8081
```

## Errors people can read

A mistake in the script points at the line. The console prints it, and the
server answers every request with the same text, so you see it in the
browser too:

```
$ CHAM_LANG=en cham app.cham
app.cham, line 3: I don't know the word "sav". Did you mean "save"?
Fix the line above and start Cham again. Nothing was served.

$ CHAM_LANG=uk cham app.cham
app.cham, рядок 3: Я не знаю слова "sav". Можливо, ви мали на увазі "save"?

$ CHAM_LANG=ru cham app.cham
app.cham, строка 3: Я не знаю слова "sav". Возможно, вы имели в виду "save"?
```

A bad request gets a JSON error the frontend can show as is:

```
$ curl -d '{"name":"Ivan"}' localhost:8080/orders
{ "error": "Поле \"phone\" відсутнє або порожнє. Цей маршрут потребує: name, phone, item." }

$ curl -d '{name:Ivan' localhost:8080/orders
{ "error": "Тіло запиту не є коректним JSON (key must be a string at line 1 column 2). Надішліть щось на кшталт {\"name\": \"Олена\"} із Content-Type: application/json." }

$ curl -d '{"name":"A"}' localhost:8099/x      # script says: answer "Hi " + nam
{ "error": "Вы использовали \"nam\", но запрос его не прислал и ничто не задало его раньше. Отправьте \"nam\" с запросом или напишите nam = ... выше (строка 2)." }

$ curl localhost:8081/nothing
{ "error": "Nothing answers GET /nothing. Routes in this app: POST /bookings, GET /bookings" }
```

Every message lives in `src/msg.rs`. Adding Polish means adding one column.

## Research behind the design

**Novices run out of working memory before they run out of ideas.**
Sweller's cognitive load theory starts from the observation that problem
solving by means-ends analysis "requires a relatively large amount of
cognitive processing capacity which is consequently unavailable for schema
acquisition". A beginner juggling a router, an ORM, env files and a build tool
has little capacity left to learn what a request is.
Cham cuts the number of moving parts to one file and ten words.
([Sweller 1988, Cognitive Science 12(2)](https://onlinelibrary.wiley.com/doi/10.1207/s15516709cog1202_4))

**Unrestricted AI help raises practice scores and lowers learning.**
In a randomized trial with nearly 1,000 high school students, a plain
GPT-4 chat improved practice grades by 48%, but when access was removed those
students scored 17% worse than students who never had it. Students mostly
asked for answers and copied them, and they did not notice they had learned
less. A tutor that gave hints instead of answers removed most of the harm.
([Bastani et al. 2025, PNAS 122(26)](https://www.pnas.org/doi/10.1073/pnas.2422633122))

**Trusting the tool more means checking it less.** A Microsoft Research
and CMU survey of 319 knowledge workers (936 real tasks) found that "higher
confidence in GenAI is associated with less critical thinking, while higher
self-confidence is associated with more critical thinking."
([Lee et al., CHI 2025](https://www.microsoft.com/en-us/research/publication/the-impact-of-generative-ai-on-critical-thinking-self-reported-reductions-in-cognitive-effort-and-confidence-effects-from-a-survey-of-knowledge-workers/))

**People misjudge how much AI helps them.** In METR's randomized trial,
16 experienced open-source developers expected AI tools to cut task time by
24% and believed afterwards that they had saved 20%. Measured, they took 19%
longer. The authors warn against generalizing to every setting, but the gap
between felt and real progress matches what Bastani et al. saw in students.
([Becker et al. 2025, arXiv:2507.09089](https://arxiv.org/abs/2507.09089))

What Cham takes from this:

1. Keep the whole program small enough to hold in your head.
2. Make each step visible and in order, so you can predict the result before
   you run it.
3. When it breaks, explain the cause and the fix in the student's language.
   A stack trace sends them back to the chatbot. A sentence they understand
   lets them fix it themselves.

Cham does not ban AI. A student can still ask a model to write the `.cham`
file. The difference is that the result is a dozen lines they can check, not
four hundred they have to trust.

## Layout

```
src/main.rs     HTTP server, request pipeline, static files, CORS
src/script.rs   .cham parser: indentation, expressions, "did you mean" hints
templates/      what `cham new` writes
src/store.rs    JSON-file collections
src/msg.rs      every user-facing message in en / uk / ru
examples/       shawarma and lab demos
```

About 1,000 lines of Rust. Dependencies: `tiny_http` and `serde_json`.

## Limits

No auth, no update step yet, single-threaded, binds to 127.0.0.1.
It is a teaching tool and a prototype backend, not something to put in front
of the internet.

## License

MIT
