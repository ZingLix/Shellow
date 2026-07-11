import { useEffect, useRef, useState } from "react";
import {
  AndroidLogo,
  AppleLogo,
  ArrowRight,
  BracketsCurly,
  Check,
  Code,
  Cpu,
  DeviceMobile,
  GitBranch,
  GithubLogo,
  HardDrives,
  Key,
  LockKey,
  Monitor,
  ShieldCheck,
  TerminalWindow,
  WifiHigh,
} from "@phosphor-icons/react";

import { AnimatedBeam } from "./components/AnimatedBeam.jsx";
import { BlurFade } from "./components/BlurFade.jsx";
import { BorderBeam } from "./components/BorderBeam.jsx";
import { Iphone } from "./components/Iphone.jsx";

const TESTFLIGHT = "https://testflight.apple.com/join/EFnQTH4T";
const PLAY = "https://play.google.com/apps/testing/xyz.zinglix.shellow";
const GITHUB = "https://github.com/ZingLix/Shellow";
const RELEASES = "https://github.com/ZingLix/Shellow/releases";

function ScrollProgress() {
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    const update = () => {
      const max = document.documentElement.scrollHeight - window.innerHeight;
      setProgress(max > 0 ? window.scrollY / max : 0);
    };
    update();
    window.addEventListener("scroll", update, { passive: true });
    window.addEventListener("resize", update);
    return () => {
      window.removeEventListener("scroll", update);
      window.removeEventListener("resize", update);
    };
  }, []);

  return <div className="scroll-progress" style={{ transform: `scaleX(${progress})` }} />;
}

function Brand({ compact = false }) {
  return (
    <span className="brand">
      <img src="/shellow-icon.png" alt="" className="brand-icon" />
      {!compact && <span>Shellow</span>}
    </span>
  );
}

function Header() {
  return (
    <header className="site-header">
      <nav className="nav-shell" aria-label="Primary navigation">
        <a className="brand-link" href="#top" aria-label="Shellow home">
          <Brand />
        </a>
        <div className="nav-links">
          <a href="#reliability">Reliability</a>
          <a href="#performance">Performance</a>
          <a href="#agents">Agents</a>
          <a href="#platforms">Platforms</a>
          <a href="#open-source">Open source</a>
        </div>
        <a className="nav-download" href="#download">
          Get the beta
        </a>
      </nav>
    </header>
  );
}

function StoreButtons({ compact = false }) {
  return (
    <div className={`store-buttons ${compact ? "store-buttons-compact" : ""}`}>
      <a className="button button-primary" href={TESTFLIGHT} target="_blank" rel="noreferrer">
        <AppleLogo weight="fill" />
        TestFlight
      </a>
      <a className="button button-secondary" href={PLAY} target="_blank" rel="noreferrer">
        <AndroidLogo weight="fill" />
        Android beta
      </a>
      {!compact && (
        <a className="button button-quiet" href={GITHUB} target="_blank" rel="noreferrer">
          <GithubLogo weight="fill" />
          GitHub
        </a>
      )}
    </div>
  );
}

function Hero() {
  return (
    <section id="top" className="hero section-dark">
      <img className="hero-icon-field" src="/shellow-icon.png" alt="" aria-hidden="true" />
      <div className="hero-rule" />
      <div className="container hero-grid">
        <BlurFade className="hero-copy">
          <div className="beta-line">
            <span className="status-square" />
            Public beta for iOS and Android
          </div>
          <h1>Your machine.<br />In your pocket.</h1>
          <p className="hero-lede">
            A reliable, GPU-native mobile terminal for remote shells and coding agents.
            Direct to your machine over SSH.
          </p>
          <StoreButtons />
          <div className="hero-proof" aria-label="Product highlights">
            <span>libghostty-vt</span>
            <span>russh</span>
            <span>wgpu</span>
            <span>SwiftUI + Compose</span>
          </div>
        </BlurFade>

        <BlurFade delay={0.12} direction="left" className="hero-devices">
          <div className="hero-device hero-device-terminal">
            <Iphone src="/screens/ios-terminal.jpg" />
          </div>
          <div className="hero-device hero-device-hosts">
            <Iphone src="/screens/ios-hosts.jpg" />
          </div>
        </BlurFade>
      </div>
    </section>
  );
}

function SectionHeading({ number, title, text, dark = false }) {
  return (
    <div className={`section-heading ${dark ? "section-heading-dark" : ""}`}>
      <div className="section-number">{number}</div>
      <div>
        <h2>{title}</h2>
        {text && <p>{text}</p>}
      </div>
    </div>
  );
}

function ArchitectureFlow() {
  const containerRef = useRef(null);
  const phoneRef = useRef(null);
  const sshRef = useRef(null);
  const vtRef = useRef(null);
  const renderRef = useRef(null);
  const surfaceRef = useRef(null);

  const nodes = [
    { ref: phoneRef, label: "Shellow", detail: "Native app", Icon: DeviceMobile },
    { ref: sshRef, label: "russh", detail: "SSH transport", Icon: WifiHigh },
    { ref: vtRef, label: "libghostty-vt", detail: "Terminal state", Icon: TerminalWindow },
    { ref: renderRef, label: "wgpu", detail: "GPU renderer", Icon: Cpu },
    { ref: surfaceRef, label: "Metal / Vulkan", detail: "Native surface", Icon: Monitor },
  ];

  return (
    <div className="architecture-flow" ref={containerRef}>
      <AnimatedBeam containerRef={containerRef} fromRef={phoneRef} toRef={sshRef} duration={4.1} />
      <AnimatedBeam containerRef={containerRef} fromRef={sshRef} toRef={vtRef} duration={4.3} delay={0.25} />
      <AnimatedBeam containerRef={containerRef} fromRef={vtRef} toRef={renderRef} duration={4.5} delay={0.5} />
      <AnimatedBeam containerRef={containerRef} fromRef={renderRef} toRef={surfaceRef} duration={4.7} delay={0.75} />
      {nodes.map(({ ref, label, detail, Icon }) => (
        <div className="architecture-node" ref={ref} key={label}>
          <Icon weight="duotone" />
          <strong>{label}</strong>
          <span>{detail}</span>
        </div>
      ))}
    </div>
  );
}

function Reliability() {
  const details = [
    ["Terminal semantics", "Persistent VT state, true color, scrollback, selection, search, mouse modes and OSC handling."],
    ["SSH you can trust", "Password and OpenSSH key authentication, host-key verification, pinning and keepalive support."],
    ["Sessions that last", "Attach to tmux, GNU screen or Zellij and return to work after the phone disconnects."],
  ];

  return (
    <section id="reliability" className="section section-light">
      <div className="container">
        <BlurFade>
          <SectionHeading
            number="01"
            title="Built from dependable parts."
            text="Shellow keeps terminal, transport and rendering responsibilities separate, then shares the same Rust behavior across both mobile platforms."
          />
        </BlurFade>
        <BlurFade delay={0.08}>
          <ArchitectureFlow />
        </BlurFade>
        <div className="reliability-list">
          {details.map(([title, body], index) => (
            <BlurFade delay={0.08 + index * 0.06} key={title} className="reliability-item">
              <span className="item-index">0{index + 1}</span>
              <h3>{title}</h3>
              <p>{body}</p>
            </BlurFade>
          ))}
        </div>
      </div>
    </section>
  );
}

function Performance() {
  const points = [
    "Native terminal surfaces — no embedded web terminal",
    "Metal on iOS and Vulkan on Android",
    "Glyph caching, text shaping and dirty-row updates",
    "A shared renderer built for sustained sessions",
  ];

  return (
    <section id="performance" className="section section-dark performance-section">
      <div className="container performance-grid">
        <BlurFade className="performance-copy">
          <SectionHeading
            dark
            number="02"
            title="Not a web terminal in a wrapper."
            text="Terminal frames are produced by the shared Rust core and presented directly through native GPU surfaces."
          />
          <ul className="check-list">
            {points.map((point) => (
              <li key={point}><Check weight="bold" />{point}</li>
            ))}
          </ul>
          <div className="performance-code">
            <code>keyboard → russh → libghostty-vt → wgpu → native surface</code>
          </div>
        </BlurFade>

        <BlurFade delay={0.12} direction="left" className="terminal-shot-shell">
          <img src="/screens/ios-terminal.jpg" alt="Shellow terminal running on iOS Simulator with sanitized demo data" />
          <BorderBeam colorFrom="#1c9f70" colorTo="#75dbab" duration={8} size={100} />
        </BlurFade>
      </div>
    </section>
  );
}

const agentContent = {
  codex: {
    name: "Codex",
    description: "Browse projects and threads, stream responses, inspect tool activity, handle approvals and resume remote work from your phone.",
    items: ["Projects and persistent threads", "Tool activity and command output", "Approvals, models and sandbox settings"],
  },
};

function Agents() {
  const [selected, setSelected] = useState("codex");
  const active = agentContent[selected];

  return (
    <section id="agents" className="section section-light agents-section">
      <div className="container">
        <BlurFade>
          <SectionHeading
            number="03"
            title="Your coding agents. First-class."
            text="Codex runs where your code lives. Shellow connects to that machine over SSH, without adding a Shellow relay or hosted backend."
          />
        </BlurFade>

        <div className="agents-grid">
          <BlurFade delay={0.08} className="agents-visual">
            <div className="agents-phone">
              <Iphone src="/screens/ios-hosts.jpg" frame="light" />
            </div>
            <div className="agent-path" aria-label="Direct SSH architecture">
              <span>Phone</span><ArrowRight /><span>SSH</span><ArrowRight /><span>Your machine</span>
            </div>
          </BlurFade>

          <BlurFade delay={0.14} className="agent-panel">
            <div className="agent-tabs" role="tablist" aria-label="Supported coding agents">
              {Object.entries(agentContent).map(([key, item]) => (
                <button
                  type="button"
                  role="tab"
                  aria-selected={selected === key}
                  className={selected === key ? "active" : ""}
                  onClick={() => setSelected(key)}
                  key={key}
                >
                  {item.name}
                </button>
              ))}
            </div>
            <div className="agent-panel-body" key={selected}>
              <div className="agent-logo-line">
                <BracketsCurly weight="duotone" />
                <h3>{active.name}</h3>
              </div>
              <p>{active.description}</p>
              <ul>
                {active.items.map((item) => <li key={item}><Check weight="bold" />{item}</li>)}
              </ul>
            </div>
          </BlurFade>
        </div>
      </div>
    </section>
  );
}

function Platforms() {
  return (
    <section id="platforms" className="section section-dark platforms-section">
      <div className="container">
        <BlurFade>
          <SectionHeading
            dark
            number="04"
            title="One core. Two native apps."
            text="SwiftUI and Jetpack Compose own the mobile experience. Rust owns SSH, terminal state, coding-agent sessions and GPU rendering."
          />
        </BlurFade>
        <div className="platform-table">
          <BlurFade delay={0.08} className="platform-row">
            <AppleLogo weight="fill" />
            <div><h3>iOS and iPadOS</h3><p>SwiftUI interface · Metal presentation · Keychain-backed secrets</p></div>
            <a href={TESTFLIGHT} target="_blank" rel="noreferrer">TestFlight <ArrowRight /></a>
          </BlurFade>
          <BlurFade delay={0.12} className="platform-row">
            <AndroidLogo weight="fill" />
            <div><h3>Android</h3><p>Jetpack Compose interface · Vulkan presentation · Keystore-backed secrets</p></div>
            <a href={PLAY} target="_blank" rel="noreferrer">Google Play <ArrowRight /></a>
          </BlurFade>
        </div>
        <BlurFade delay={0.18} className="shared-core-line">
          <HardDrives weight="duotone" />
          <span>Shared Rust core</span>
          <code>SSH · VT · renderer · agents</code>
        </BlurFade>
      </div>
    </section>
  );
}

function OpenSource() {
  const items = [
    { Icon: Code, title: "Read the implementation", text: "Inspect the native apps, shared Rust core, bridges and build scripts." },
    { Icon: GitBranch, title: "Build it yourself", text: "The repository documents local builds for iOS and Android." },
    { Icon: ShieldCheck, title: "Apache 2.0", text: "Use, study and contribute under a permissive open-source license." },
  ];

  return (
    <section id="open-source" className="section section-light open-source-section">
      <div className="container open-source-grid">
        <BlurFade className="open-source-copy">
          <SectionHeading
            number="05"
            title="Open by design."
            text="A terminal is part of your trust boundary. Shellow keeps the implementation visible, auditable and buildable."
          />
          <a className="source-link" href={GITHUB} target="_blank" rel="noreferrer">
            <GithubLogo weight="fill" />
            github.com/ZingLix/Shellow
            <ArrowRight />
          </a>
        </BlurFade>
        <div className="source-list">
          {items.map(({ Icon, title, text }, index) => (
            <BlurFade delay={0.08 + index * 0.06} className="source-item" key={title}>
              <Icon weight="duotone" />
              <div><h3>{title}</h3><p>{text}</p></div>
            </BlurFade>
          ))}
        </div>
      </div>
    </section>
  );
}

function Trust() {
  const points = [
    [LockKey, "No Shellow relay", "Your phone connects to your own machine over SSH."],
    [Key, "Secrets stay native", "Keychain on Apple platforms and Keystore-backed storage on Android."],
    [ShieldCheck, "Host identity", "Trust-on-first-use, verification and host-key pinning are built in."],
  ];

  return (
    <section className="trust-section">
      <div className="container trust-grid">
        {points.map(([Icon, title, text]) => (
          <div className="trust-item" key={title}>
            <Icon weight="duotone" />
            <h3>{title}</h3>
            <p>{text}</p>
          </div>
        ))}
      </div>
    </section>
  );
}

function FAQ() {
  const entries = [
    ["Is Shellow available now?", "Shellow is currently distributed as a beta through TestFlight, Google Play testing and GitHub Releases."],
    ["Does Shellow use a web terminal?", "No. The terminal uses libghostty-vt for state and a shared wgpu renderer attached to Metal or Vulkan native surfaces."],
    ["Where does Codex run?", "Codex runs on the machine you connect to. Shellow attaches over SSH and presents its sessions through a native mobile interface."],
    ["Can sessions continue after I disconnect?", "Persistent terminal workspaces and supported coding-agent sessions can remain on the remote machine and be resumed later."],
  ];

  return (
    <section className="faq-section section-light">
      <div className="container faq-grid">
        <h2>Questions</h2>
        <div className="faq-list">
          {entries.map(([question, answer], index) => (
            <details key={question} open={index === 0}>
              <summary>{question}</summary>
              <p>{answer}</p>
            </details>
          ))}
        </div>
      </div>
    </section>
  );
}

function Download() {
  return (
    <section id="download" className="download-section section-dark">
      <div className="container download-grid">
        <BlurFade>
          <Brand />
          <h2>Take your terminal with you.</h2>
          <p>Join the beta on iOS or Android, or download the latest build from GitHub.</p>
        </BlurFade>
        <BlurFade delay={0.1} className="download-actions">
          <StoreButtons compact />
          <a href={RELEASES} target="_blank" rel="noreferrer">GitHub Releases <ArrowRight /></a>
        </BlurFade>
      </div>
    </section>
  );
}

function Footer() {
  return (
    <footer className="footer">
      <div className="container footer-grid">
        <Brand />
        <p>Native mobile terminal and coding-agent client.</p>
        <div className="footer-links">
          <a href={GITHUB} target="_blank" rel="noreferrer">Source</a>
          <a href={`${GITHUB}/issues`} target="_blank" rel="noreferrer">Issues</a>
          <a href={`${GITHUB}/blob/main/LICENSE`} target="_blank" rel="noreferrer">License</a>
        </div>
      </div>
    </footer>
  );
}

export function App() {
  return (
    <div className="site-shell">
      <Header />
      <ScrollProgress />
      <main>
        <Hero />
        <Reliability />
        <Performance />
        <Agents />
        <Platforms />
        <OpenSource />
        <Trust />
        <FAQ />
        <Download />
      </main>
      <Footer />
    </div>
  );
}
