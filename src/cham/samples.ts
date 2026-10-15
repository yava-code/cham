const SAMPLES: Record<string, string> = {
  shawarma: `# Shawarma "U Tarasa", Warsaw, ul. Grochowska.
# Customers order from the page, the cook watches the queue.

port = 8080
lang = "uk"
price = 28
large_extra = 6

when someone posts /orders:
    need name, phone, item, size
    total = price
    if size == "велика": total = price + large_extra
    time = now
    order = save name, phone, item, size, total, time to orders
    answer 201 "Дякуємо, " + name + "! До сплати " + total + " zł.", order

when someone opens /orders:
    answer all orders

when someone opens /orders/:id:
    order = get orders id
    if not order:
        answer 404 "Замовлення " + id + " немає"
    answer order

when someone deletes /orders/:id:
    order = get orders id
    if not order:
        answer 404 "Замовлення " + id + " немає"
    delete orders id
    answer "Видано: " + order.item + " для " + order.name
`,
  lab: `# Equipment booking for a university robotics lab.
# One device, one slot, one person.

port = 8081
lang = "en"

when someone posts /bookings:
    need device, slot, who
    taken = get bookings device, slot
    if taken:
        answer 409 device + " at " + slot + " is already booked by " + taken.who
    booking = save device, slot, who to bookings
    answer 201 device + " is yours at " + slot + ".", booking

when someone opens /bookings:
    answer all bookings

when someone deletes /bookings/:id:
    delete bookings id
    answer "Booking " + id + " cancelled."
`,
  notes: `# A simple guest book.

port = 8080
lang = "en"

when someone posts /notes:
    need name, text
    time = now
    note = save name, text, time to notes
    answer "Thanks, " + name, note

when someone opens /notes:
    answer all notes

when someone deletes /notes/:id:
    note = get notes id
    if not note:
        answer 404 "There is no note number " + id
    delete notes id
    answer "Deleted note " + id
`,
  broken: `# This script has a typo — try it to see the error.

when someone posts /x:
    need a
    sav a to things
    answer "ok"
`,
};

export default SAMPLES;
