🧿 VIZR — Universal Terminal Visualizer Engine

«Real-time terminal signal intelligence engine built in Rust
Designed for mobile Linux environments (Pixel / Debian shells)»

---

⚡ Overview

VIZR is a terminal-native visualization engine that transforms raw system input streams into live, animated intelligence dashboards.

It ingests data from multiple sources such as:

- network activity (ping)
- system processes (build logs)
- stdin pipelines
- synthetic generators

…and converts them into:

- ⚡ Energy (activity intensity)
- 🟢 Stability (signal consistency)
- 🔴 Anomaly (noise / irregularity)
- 🟣 Sync Rate (coherence between signals)

All rendered in a cyberpunk-style TUI dashboard directly in your terminal.

---

🧠 Philosophy

VIZR is not a logger.
It is not a monitor.

It is a signal interpreter.

Instead of showing raw data, it translates system behavior into visual meaning.

---

🚀 Features

🔹 Multi-Stream Input Engine

- Supports multiple concurrent data sources
- Merges signals into unified system state

🔹 Real-Time Metrics

- Energy (throughput/activity)
- Stability (variance smoothing)
- Anomaly detection (noise spikes)
- Sync Rate (signal coherence)

🔹 Adaptive Rendering

- Dynamic matrix visual (auto-scaling)
- Signal wave graphs
- Pulse and distortion fields

🔹 Event Feed

- Timestamped (local system time)
- Source-aware tagging (PING / BUILD / STDIN)

🔹 Terminal UI (TUI)

- Built with "ratatui" + "crossterm"
- Fully animated in terminal
- Zero GUI required

---

🛠️ Requirements

- Rust (stable)
- Cargo
- Linux environment (Debian, Termux-style, Pixel Linux shell)

---

⚙️ Installation

git clone https://github.com/ShamelesAbyss/Vizr
cd vizr
cargo build --release

---

▶️ Running

Default (synthetic mode)

./target/release/vizr

---

Pipe real data into VIZR

Ping stream

ping google.com | ./target/release/vizr --stdin

Build logs

cargo build 2>&1 | ./target/release/vizr --stdin

Any command

your_command | ./target/release/vizr --stdin

---

🎛 Controls

Key| Action
"q"| Quit
"m"| Cycle modes

---

📡 Input Modes

Mode| Description
SYNTHETIC| Internal signal generator
STDIN| External pipe input
PING| Network signal
BUILD| System process logs

---

🧬 Metrics Explained

⚡ Energy

Represents total activity volume across all streams

🟢 Stability

Measures consistency of incoming signals

🔴 Anomaly

Detects irregular spikes or noise bursts

🟣 Sync Rate

Indicates how aligned multiple signal sources are

---

🧱 Architecture

Input Sources
   ↓
Adapters (Ping / Build / STDIN / Synthetic)
   ↓
Signal Ingestion Layer
   ↓
Fusion Engine
   ↓
State Model
   ↓
TUI Renderer (ratatui)

---

🔮 Roadmap

- Multi-stream weighted fusion engine
- Source tagging + filtering
- Export metrics (JSON / logs)
- Plugin adapters (custom inputs)
- Network capture (non-root safe methods)
- Android-native wrapper (future)

---

⚠️ Limitations

- Does NOT intercept system traffic (no root / VPN)
- Visual interpretation is heuristic-based
- Designed for observability, not enforcement

---

🧪 Example Use Cases

- Visualizing build systems
- Monitoring CLI tools in real-time
- Debugging noisy logs
- Creating terminal dashboards
- Mobile dev environment observability

---

💀 Why this exists

Because terminals don’t have to be boring.

---

👤 Author

Built by ShamelessAbyss
Powered by Rust + mobile Linux chaos

---

🧿 License

MIT
