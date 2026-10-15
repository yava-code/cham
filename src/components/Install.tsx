export default function Install() {
  return (
    <section className="section" id="install">
      <div className="section-tag">get started</div>
      <h2>Three commands to a running backend.</h2>
      <p>
        Cham is a single Rust binary. If you have Rust installed, you already have everything
        you need. If not, install it from rustup.org first.
      </p>
      <div className="install-steps">
        <div className="install-step">
          <div className="install-num">1</div>
          <div className="install-body">
            <h3>Build the binary</h3>
            <p>Clone the repo and compile. The binary lands in target/release/.</p>
            <pre className="install-code"><code>{`git clone https://github.com/yava-code/cham
cd cham
cargo build --release`}</code></pre>
          </div>
        </div>
        <div className="install-step">
          <div className="install-num">2</div>
          <div className="install-body">
            <h3>Scaffold a new app</h3>
            <p>Creates a folder with app.cham and index.html, ready to run.</p>
            <pre className="install-code"><code>{`./target/release/cham new myapp`}</code></pre>
          </div>
        </div>
        <div className="install-step">
          <div className="install-num">3</div>
          <div className="install-body">
            <h3>Run it</h3>
            <p>Open http://127.0.0.1:8080 in your browser. Edit app.cham and restart to change the API.</p>
            <pre className="install-code"><code>{`./target/release/cham myapp`}</code></pre>
          </div>
        </div>
      </div>
    </section>
  );
}
