//! Every message a person can see, in three languages.
//! Keys are short on purpose; `say` fills {0}, {1}, ... from args.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Lang { En, Uk, Ru }

impl Lang {
    pub fn parse(s: &str) -> Option<Lang> {
        match s.trim().to_lowercase().as_str() {
            "en" | "english" => Some(Lang::En),
            "uk" | "ua" | "ukrainian" => Some(Lang::Uk),
            "ru" | "russian" => Some(Lang::Ru),
            _ => None,
        }
    }
}

fn table(lang: Lang, key: &str) -> &'static str {
    use Lang::*;
    match (key, lang) {
        ("listening", En) => "Cham is running at http://{0}  (script: {1}, {2} routes)",
        ("listening", Uk) => "Cham працює на http://{0}  (скрипт: {1}, маршрутів: {2})",
        ("listening", Ru) => "Cham работает на http://{0}  (скрипт: {1}, маршрутов: {2})",

        ("no_script", En) => "I can't open {0}. Create it next to the binary or pass the path: cham path/to/app.cham",
        ("no_script", Uk) => "Не можу відкрити {0}. Створіть файл поруч із програмою або передайте шлях: cham шлях/до/app.cham",
        ("no_script", Ru) => "Не могу открыть {0}. Создайте файл рядом с программой или передайте путь: cham путь/к/app.cham",

        ("line", En) => "{0}, line {1}: {2}",
        ("line", Uk) => "{0}, рядок {1}: {2}",
        ("line", Ru) => "{0}, строка {1}: {2}",

        ("unknown_word", En) => "I don't know the word \"{0}\".{1}",
        ("unknown_word", Uk) => "Я не знаю слова \"{0}\".{1}",
        ("unknown_word", Ru) => "Я не знаю слова \"{0}\".{1}",

        ("did_you_mean", En) => " Did you mean \"{0}\"?",
        ("did_you_mean", Uk) => " Можливо, ви мали на увазі \"{0}\"?",
        ("did_you_mean", Ru) => " Возможно, вы имели в виду \"{0}\"?",

        ("needs_arg", En) => "\"{0}\" needs more after it. Example: {1}",
        ("needs_arg", Uk) => "Після \"{0}\" чогось бракує. Приклад: {1}",
        ("needs_arg", Ru) => "После \"{0}\" чего-то не хватает. Пример: {1}",

        ("step_outside", En) => "\"{0}\" must sit under a route. Start a route first, e.g. post /orders",
        ("step_outside", Uk) => "\"{0}\" має стояти під маршрутом. Спочатку напишіть маршрут, наприклад post /orders",
        ("step_outside", Ru) => "\"{0}\" должен стоять под маршрутом. Сначала напишите маршрут, например post /orders",

        ("bad_status", En) => "\"{0}\" is not an HTTP status. Use a number like 200 or 201.",
        ("bad_status", Uk) => "\"{0}\" не є HTTP-статусом. Напишіть число, наприклад 200 або 201.",
        ("bad_status", Ru) => "\"{0}\" не HTTP-статус. Напишите число, например 200 или 201.",

        ("unclosed_quote", En) => "a quote opens here but never closes. Add \" at the end of the text.",
        ("unclosed_quote", Uk) => "лапки відкриваються, але не закриваються. Додайте \" в кінці тексту.",
        ("unclosed_quote", Ru) => "кавычка открыта, но не закрыта. Добавьте \" в конце текста.",

        ("script_failed", En) => "Fix the line above and start Cham again. Nothing was served.",
        ("script_failed", Uk) => "Виправте рядок вище й запустіть Cham ще раз. Сервер не стартував.",
        ("script_failed", Ru) => "Исправьте строку выше и запустите Cham снова. Сервер не стартовал.",

        ("bad_json", En) => "The request body is not valid JSON ({0}). Send something like {\"name\": \"Olena\"} with Content-Type: application/json.",
        ("bad_json", Uk) => "Тіло запиту не є коректним JSON ({0}). Надішліть щось на кшталт {\"name\": \"Олена\"} із Content-Type: application/json.",
        ("bad_json", Ru) => "Тело запроса не корректный JSON ({0}). Отправьте что-то вроде {\"name\": \"Олена\"} с Content-Type: application/json.",

        ("not_object", En) => "Expected a JSON object in curly braces, got {0}.",
        ("not_object", Uk) => "Очікував JSON-об'єкт у фігурних дужках, а прийшло: {0}.",
        ("not_object", Ru) => "Ожидал JSON-объект в фигурных скобках, а пришло: {0}.",

        ("missing", En) => "Field \"{0}\" is missing or empty. This route needs: {1}.",
        ("missing", Uk) => "Поле \"{0}\" відсутнє або порожнє. Цей маршрут потребує: {1}.",
        ("missing", Ru) => "Поле \"{0}\" отсутствует или пустое. Этому маршруту нужны: {1}.",

        ("taken", En) => "There is already a record in \"{0}\" with the same {1}. Pick another value.",
        ("taken", Uk) => "У \"{0}\" вже є запис із таким самим {1}. Оберіть інше значення.",
        ("taken", Ru) => "В \"{0}\" уже есть запись с таким же {1}. Выберите другое значение.",

        ("no_route", En) => "Nothing answers {0} {1}. Routes in this app: {2}",
        ("no_route", Uk) => "Ніщо не відповідає на {0} {1}. Маршрути цього застосунку: {2}",
        ("no_route", Ru) => "Ничто не отвечает на {0} {1}. Маршруты этого приложения: {2}",

        ("not_found", En) => "No record in \"{0}\" where {1} = {2}.",
        ("not_found", Uk) => "У \"{0}\" немає запису, де {1} = {2}.",
        ("not_found", Ru) => "В \"{0}\" нет записи, где {1} = {2}.",

        ("new_field", En) => "  note: \"{0}\" got a new field \"{1}\". Stored it, no migration needed.",
        ("new_field", Uk) => "  примітка: у \"{0}\" з'явилось нове поле \"{1}\". Зберіг, міграція не потрібна.",
        ("new_field", Ru) => "  заметка: в \"{0}\" появилось новое поле \"{1}\". Сохранил, миграция не нужна.",

        ("disk", En) => "Could not write data/{0}.json: {1}. Check that the folder is writable.",
        ("disk", Uk) => "Не вдалося записати data/{0}.json: {1}. Перевірте, чи можна писати в папку.",
        ("disk", Ru) => "Не удалось записать data/{0}.json: {1}. Проверьте, можно ли писать в папку.",

        ("port_busy", En) => "Port {0} is busy. Stop the other program or run with PORT=8081.",
        ("port_busy", Uk) => "Порт {0} зайнятий. Зупиніть іншу програму або запустіть з PORT=8081.",
        ("port_busy", Ru) => "Порт {0} занят. Остановите другую программу или запустите с PORT=8081.",

        (_, _) => "{0}",
    }
}

pub fn say(lang: Lang, key: &str, args: &[&str]) -> String {
    let mut out = table(lang, key).to_string();
    for (i, a) in args.iter().enumerate() {
        out = out.replace(&format!("{{{i}}}"), a);
    }
    out
}
