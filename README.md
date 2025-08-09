## rust-top

An htop-like terminal process viewer written in Rust using ratatui + crossterm + sysinfo.

### Features
- CPU and memory overview
- Sortable process list (by CPU, memory, or PID)
- Incremental name filter
- Vim-style navigation (j/k) and arrow keys

### Install
- From source:
  - Prereqs: Rust toolchain (use `rustup`)
  - Build and run:
    ```bash
    cargo run
    ```
  - Build release binary:
    ```bash
    cargo build --release
    ./target/release/rust-top
    ```

### Usage
- Keys:
  - q / Esc or Ctrl-C: quit
  - j / k or ArrowDown / ArrowUp: move selection
  - c / m / p: sort by CPU / Memory / PID
  - /: start typing to filter by process name, Enter/Esc to finish

### Notes
- Tested on Linux (including WSL2). macOS should work; Windows support depends on terminal capabilities.
- Process listing is capped to 200 rows for responsiveness; this can be adjusted in code.

### Roadmap
- Per-core CPU bars and load averages
- Process actions: kill/terminate, renice
- Additional columns: user, state, threads, command, time, I/O
- Paging and stable selection across filters
- Configurable refresh rate and help popup

### License
TBD by the repository owner.


