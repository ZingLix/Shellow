<p align="center">
  <img src="apps/ios/Shellow/Assets.xcassets/AppIcon.appiconset/Icon-ios-marketing-1024.png" width="160" alt="Shellow app icon" />
</p>

<h1 align="center">Shellow</h1>

<p align="center">
  A high-performance mobile terminal and native Codex client,<br />
  connecting directly to your machines over SSH.
</p>

<p align="center">
  <a href="https://testflight.apple.com/join/EFnQTH4T">
    <img src="https://img.shields.io/badge/TestFlight-0D96F6?logo=app-store&logoColor=white&style=for-the-badge" alt="Download on TestFlight" />
  </a>
  <a href="https://play.google.com/apps/testing/xyz.zinglix.shellow">
    <img src="https://img.shields.io/badge/Google_Play_Beta-414141?logo=google-play&logoColor=white&style=for-the-badge" alt="Get the Google Play beta" />
  </a>
  <a href="https://github.com/ZingLix/Shellow/releases">
    <img src="https://img.shields.io/badge/GitHub_Releases-000000?logo=github&logoColor=white&style=for-the-badge" alt="Download from GitHub Releases" />
  </a>
</p>

> [!IMPORTANT]
> Shellow is currently in beta. Features and stored data formats may change, and
> you may encounter incomplete behavior. Please avoid relying on it as your only
> way to access a critical server.

## Install

### iOS

Install the latest beta through [TestFlight](https://testflight.apple.com/join/EFnQTH4T).
Apple may report that the beta is full or unavailable when all tester slots are
occupied.

### Android

Google Play access is limited to members of the Shellow test group:

1. Join the [Shellow test group](https://groups.google.com/g/shellow-test) with
   the Google account you use on Google Play.
2. Open the [Google Play testing page](https://play.google.com/apps/testing/xyz.zinglix.shellow)
   with the same account and opt in.
3. Install Shellow from Google Play. Access may take a few minutes to become
   available after joining the group.

   **Note**: Google Play may show a price or payment warning. This is part of the
   test setup—you will not be charged and do not need to make a real payment.

Alternatively, download an Android build directly from
[GitHub Releases](https://github.com/ZingLix/Shellow/releases). Installing an APK
outside Google Play may require enabling installation from your browser or file
manager.

## Why Shellow

### High-performance terminal

- **Ghostty terminal engine**: `libghostty-vt` provides standards-aware terminal
  parsing and state management.
- **GPU-native rendering**: A shared Rust `wgpu` renderer draws through Metal on
  iOS and Vulkan on Android.
- **Built for sustained sessions**: Text shaping, glyph caching, and dirty-row
  updates keep terminal interaction responsive.
- **Native surface end to end**: Terminal frames render directly into platform
  surfaces instead of an embedded web terminal.

### Native Codex client

- **First-class Codex UI**: Browse projects and threads, send messages, review
  tool activity, handle approvals, and adjust settings from mobile.
- **SSH only**: Shellow connects directly to Codex on your own machine, without a
  Shellow relay, hosted backend, browser session, or extra agent.
- **Persistent remote tasks**: Codex work continues on the remote host when the
  phone disconnects, and can be resumed later.
- **Setup stays in the app**: Shellow explains and runs the one-time remote setup
  only after explicit confirmation.

### Native on both platforms

- **Platform-native UI**: SwiftUI on iOS and Jetpack Compose on Android.
- **Shared Rust core**: Both apps use the same SSH, terminal, Codex, and rendering
  implementation without sharing a web view.

For implementation details, see [Architecture](docs/ARCHITECTURE.md) and the
[terminal capability checklist](docs/TERMINAL_CAPABILITIES.md).

## Build from source

Shellow combines native iOS and Android apps with a shared Rust core. See the
[building guide](docs/BUILDING.md) for setup, builds, tests, and releases.

## Project status

Shellow is under active development and currently distributed as a beta.
Platform behavior and feature parity may change between releases.

## More from me

I also make [Sillage](https://github.com/ZingLix/Sillage/),
an app that turns your trips into a personal timeline by automatically bringing
together visited places, flights, trains, hotels, and photos. It can also connect
to Immich for photo management. If you love traveling, take a look.

<p align="center">
  <a href="https://github.com/ZingLix/Sillage/">
    <img src="https://github.com/ZingLix/Sillage/blob/main/imgs/hero.png?raw=true" alt="Sillage travel timeline" width="640" />
  </a>
</p>

## License

Shellow is licensed under the [Apache License 2.0](LICENSE). Third-party
dependencies and bundled assets retain their respective licenses.
