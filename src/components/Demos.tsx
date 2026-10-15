export default function Demos() {
  return (
    <section className="section" id="demos">
      <div className="section-tag">two apps, one engine</div>
      <h2>Two demo apps that share the same skeleton.</h2>
      <p>
        Both run on the same Cham binary. The shawarma stand's order page and kitchen queue
        is about 25 lines of backend. The lab booking grid that refuses double bookings is 15.
      </p>
      <div className="demos-grid">
        <div className="demo-card demo-shawarma">
          <div className="demo-preview">
            <div style={{ textAlign: "center" }}>
              <div style={{ fontSize: 24, fontWeight: "bold" }}>У Тараса</div>
              <div style={{ fontSize: 14, color: "#7a5a3a", marginTop: 4 }}>Гроховська 214, Варшава</div>
              <div style={{ marginTop: 16, display: "inline-flex", gap: 8 }}>
                <span style={{ border: "2px solid #2b1d12", padding: "4px 10px" }}>Класична 28 zł</span>
                <span style={{ border: "2px solid #2b1d12", padding: "4px 10px", background: "#c8451c", color: "#fffaf0" }}>Гостра 30 zł</span>
              </div>
            </div>
          </div>
          <h3>Shawarma stand</h3>
          <p>Order page for a shawarma shop on Grochowska street. Customers pick from the menu, enter name and phone; the kitchen sees the queue. Includes price check and delete.</p>
          <div className="demo-link">
            <a href="#playground" onClick={() => {}}>Try it in the playground</a>
          </div>
        </div>
        <div className="demo-card demo-lab">
          <div className="demo-preview">
            <div style={{ textAlign: "center", fontFamily: "var(--mono)" }}>
              <div style={{ fontSize: 16, color: "#fff", marginBottom: 12 }}>Lab 2.14 · equipment</div>
              <table style={{ borderCollapse: "collapse", fontSize: 12 }}>
                <tbody>
                  <tr><td style={{ border: "1px solid #263238", padding: "4px 8px", color: "#90a4ae" }}>Oscilloscope</td><td style={{ border: "1px solid #263238", padding: "4px 8px", color: "#4f6b75" }}>free</td><td style={{ border: "1px solid #263238", padding: "4px 8px", background: "#2e2a12", color: "#ffd54f" }}>Marta K.</td></tr>
                  <tr><td style={{ border: "1px solid #263238", padding: "4px 8px", color: "#90a4ae" }}>Robot arm</td><td style={{ border: "1px solid #263238", padding: "4px 8px", color: "#4f6b75" }}>free</td><td style={{ border: "1px solid #263238", padding: "4px 8px", color: "#4f6b75" }}>free</td></tr>
                </tbody>
              </table>
            </div>
          </div>
          <h3>Lab bookings</h3>
          <p>Equipment booking grid for a university robotics lab. Click a free cell to book. A second person trying the same slot gets a 409 with a readable reason — one get and one if.</p>
          <div className="demo-link">
            <a href="#playground" onClick={() => {}}>Try it in the playground</a>
          </div>
        </div>
      </div>
    </section>
  );
}
