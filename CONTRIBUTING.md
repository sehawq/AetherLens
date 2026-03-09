# Contributing to AetherLens

First off, thanks for taking the time to contribute! 🎉

AetherLens is a high-performance network analysis tool built with Rust, .NET 8, and Next.js. We welcome contributions from the community to make it even better.

## Project Structure

- **core-engine/**: Rust-based packet sniffer and analysis engine.
- **api-server/**: .NET 8 Web API for data aggregation and SignalR broadcasting.
- **web-dashboard/**: Next.js 14 frontend for real-time visualization.
- **shared/proto/**: Protocol Buffers definitions shared between services.

## Prerequisites

Before you start, ensure you have the following installed:

1.  **Rust Toolchain**: `rustup update stable`
2.  **.NET 8 SDK**: [Download here](https://dotnet.microsoft.com/download/dotnet/8.0)
3.  **Node.js 18+**: [Download here](https://nodejs.org/)
4.  **Npcap SDK** (Windows): Required for building the core engine. Extract to `npcap-sdk/` in the root.

## Development Workflow

1.  **Fork the repository** and clone it locally.
2.  **Create a branch** for your feature or fix: `git checkout -b feature/amazing-feature`.
3.  **Run the lab** in demo mode to verify your setup:
    ```cmd
    start_lab.bat
    ```
    (Select option 2 for Demo Mode if you don't want to install Npcap drivers yet).

### Working on Core Engine (Rust)

- Follow standard Rust formatting:
  ```bash
  cd core-engine
  cargo fmt
  cargo clippy
  ```
- Run tests:
  ```bash
  cargo test --features packet-capture
  ```

### Working on Backend (.NET)

- Ensure code style compliance:
  ```bash
  cd api-server
  dotnet format
  ```
- Run unit tests:
  ```bash
  cd AetherLens.Tests
  dotnet test
  ```

### Working on Dashboard (Next.js)

- We use Tailwind CSS and TypeScript.
- Lint your code:
  ```bash
  cd web-dashboard
  npm run lint
  ```

## Protocol Changes

If you modify `shared/proto/aether.proto`:
1.  Rebuild the Rust core (`cargo build`).
2.  Rebuild the .NET backend (it will auto-generate C# classes).
3.  You may need to update the frontend types manually or via a generator if added.

## Pull Request Process

1.  Update the `README.md` with details of changes to the interface, if applicable.
2.  Update the `ARCHITECTURE.md` if you change the data flow or system design.
3.  Ensure your code passes all tests and linters.
4.  Open a PR with a clear description of the problem and solution.

## License

By contributing, you agree that your contributions will be licensed under its MIT License.
