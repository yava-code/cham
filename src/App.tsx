import Hero from "./components/Hero";
import Vocabulary from "./components/Vocabulary";
import Install from "./components/Install";
import Demos from "./components/Demos";
import Research from "./components/Research";
import Playground from "./components/Playground";

export default function App() {
  return (
    <>
      <nav className="nav">
        <div className="nav-inner">
          <a href="#" className="nav-brand">
            <img src="/assets/logo.png" alt="Cham" />
            <span>Cham</span>
          </a>
          <div className="nav-links">
            <a href="#language">Language</a>
            <a href="#demos">Demos</a>
            <a href="#install">Install</a>
            <a href="#research">Research</a>
            <a href="https://github.com/yava-code/cham" target="_blank" rel="noreferrer">GitHub</a>
            <a href="#playground" className="nav-cta">Playground</a>
          </div>
        </div>
      </nav>

      <Hero />
      <Playground />
      <Vocabulary />
      <Demos />
      <Install />
      <Research />

      <footer className="footer">
        <div className="footer-inner">
          <div className="footer-links">
            <a href="https://github.com/yava-code/cham" target="_blank" rel="noreferrer">GitHub</a>
            <a href="#playground">Playground</a>
            <a href="#install">Install</a>
            <a href="#language">Language</a>
          </div>
          <div className="footer-copy">
            MIT License · Built by Varfolomiy Yasenoviy
          </div>
        </div>
      </footer>
    </>
  );
}
