# termscp

<p align="center">
  <img src="/assets/images/termscp.svg" width="256" height="256" />
</p>

<p align="center">~ A feature rich terminal file transfer ~</p>
<p align="center">
  <a href="https://veeso.github.io/termscp/" target="_blank">Website</a>
  ·
  <a href="https://veeso.github.io/termscp/#get-started" target="_blank">Installation</a>
  ·
  <a href="https://veeso.github.io/termscp/#user-manual" target="_blank">User manual</a>
</p>

<p align="center">Developed by <a href="https://veeso.github.io/">@veeso</a></p>
<p align="center">Current version: 0.6.1 (31/08/2021)</p>

[![License: MIT](https://img.shields.io/badge/License-MIT-teal.svg)](https://opensource.org/licenses/MIT) [![Stars](https://img.shields.io/github/stars/veeso/termscp.svg)](https://github.com/veeso/termscp) [![Downloads](https://img.shields.io/crates/d/termscp.svg)](https://crates.io/crates/termscp) [![Crates.io](https://img.shields.io/badge/crates.io-v0.6.1-orange.svg)](https://crates.io/crates/termscp) [![Docs](https://docs.rs/termscp/badge.svg)](https://docs.rs/termscp)  

[![Linux](https://github.com/veeso/termscp/workflows/Linux/badge.svg)](https://github.com/veeso/termscp/actions) [![MacOs](https://github.com/veeso/termscp/workflows/MacOS/badge.svg)](https://github.com/veeso/termscp/actions) [![Windows](https://github.com/veeso/termscp/workflows/Windows/badge.svg)](https://github.com/veeso/termscp/actions) [![FreeBSD](https://github.com/veeso/termscp/workflows/FreeBSD/badge.svg)](https://github.com/veeso/termscp/actions) [![Coverage Status](https://coveralls.io/repos/github/veeso/termscp/badge.svg)](https://coveralls.io/github/veeso/termscp)

---

## About termscp 🖥

Termscp is a feature rich terminal file transfer and explorer, with support for SCP/SFTP/FTP. So basically is a terminal utility with an TUI to connect to a remote server to retrieve and upload files and to interact with the local file system. It is **Linux**, **MacOS**, **BSD** and **Windows** compatible and supports SFTP, SCP, FTP and FTPS.

![Explorer](assets/images/explorer.gif)

---

## Features 🎁

- 📁  Different communication protocols
  - SFTP
  - SCP
  - FTP and FTPS
- 🖥  Explore and operate on the remote and on the local machine file system with a handy UI
  - Create, remove, rename, search, view and edit files
- ⭐  Connect to your favourite hosts through built-in bookmarks and recent connections
- 📝  View and edit text files with your favourite text editor
- 💁  SFTP/SCP authentication through SSH keys and username/password
- 🐧  Compatible with Windows, Linux, BSD and MacOS
- ✏  Customizable
  - Themes
  - Custom file explorer format
  - Customizable text editor
  - Customizable file sorting
- 🔐  Save your password in your operating system key vault
- 🦀  Rust-powered
- 🤝  Easy to extend with new file transfers protocols
- 👀  Developed keeping an eye on performance
- 🦄  Frequent awesome updates

---

## Get started 🚀

If you're considering to install termscp I want to thank you 💜 ! I hope you will enjoy termscp!  
If you want to contribute to this project, don't forget to check out our contribute guide. [Read More](CONTRIBUTING.md)

If you are a Linux, a FreeBSD or a MacOS user this simple shell script will install termscp on your system with a single command:

```sh
curl --proto '=https' --tlsv1.2 -sSLf "https://git.io/JBhDb" | sh
```

while if you're a Windows user, you can install termscp with [Chocolatey](https://chocolatey.org/).

For more information or other platforms, please visit [veeso.github.io](https://veeso.github.io/termscp/#get-started) to view all installation methods.

### Requirements ❗

- **Linux** users:
  - libssh
  - libdbus-1
- **BSD** users:
  - libssh

### Optional Requirements ✔️

These requirements are not forcely required to run termscp, but to enjoy all of its features

- **Linux/BSD** users:
  - To **open** files via `V` (at least one of these)
    - *xdg-open*
    - *gio*
    - *gnome-open*
    - *kde-open*
- **Linux** users:
  - A keyring manager: read more in the [User manual](docs/man.md#linux-keyring)
- **WSL** users
  - To **open** files via `V` (at least one of these)
    - [wslu](https://github.com/wslutilities/wslu)

---

## Buy me a coffee ☕

If you like termscp and you'd love to see the project to grow, please consider a little donation 🥳

[![Buy-me-a-coffee](https://img.buymeacoffee.com/button-api/?text=Buy%20me%20a%20coffee&emoji=&slug=veeso&button_colour=404040&font_colour=ffffff&font_family=Comic&outline_colour=ffffff&coffee_colour=FFDD00)](https://www.buymeacoffee.com/veeso)

---

## Documentation 📚

The developer documentation can be found on Rust Docs at <https://docs.rs/termscp>

---

## Known issues 🧻

- `NoSuchFileOrDirectory` on connect (WSL1): I know about this issue and it's a glitch of WSL I guess. Don't worry about it, just move the termscp executable into another PATH location, such as `/usr/bin`, or install it through the appropriate package format (e.g. deb).

---

## Upcoming Features 🧪

Major termscp releases will now be seasonal, so expect 4 major updates during the year.

Planned for *🍁 Autumn update 🍇*:

- **Self-update ⬇️**: In order to increase users updating termscp, I want to provide the possibility to  update termscp directly from application, when a new update is available.
- **AWS S3 support 🪣**: I'll use `rust-s3` library to implement this. This is really big **Maybe** for the autumn update and might be moved to the Winter update.
- **Prompt before replacing files ☢️**: Possibility to configure whether a prompt should be displayed before replacing files.

Planned for *❄️ Winter update ⛄*:

- **SMB Support 🎉**: This will require a long time to be implemented, since I'm currently working on a Rust native SMB library, since I don't want to add new C-bindings. ~~Fear the 🦚~~
- **Configuration profile for bookmarks 📚**: Basically this feature adds the possibility to have a specific setup for a certain host, instead of having only one global configuration.

Along to new features, termscp developments is now focused on UX and performance improvements, so if you have any suggestion, feel free to open an issue.

---

## Contributing and issues 🤝🏻

Contributions, bug reports, new features and questions are welcome! 😉
If you have any question or concern, or you want to suggest a new feature, or you want just want to improve termscp, feel free to open an issue or a PR.

Please follow [our contributing guidelines](CONTRIBUTING.md)

---

## Changelog ⏳

View termscp's changelog [HERE](CHANGELOG.md)

---

## Powered by 💪

termscp is powered by these aweseome projects:

- [bytesize](https://github.com/hyunsik/bytesize)
- [crossterm](https://github.com/crossterm-rs/crossterm)
- [edit](https://github.com/milkey-mouse/edit)
- [keyring-rs](https://github.com/hwchen/keyring-rs)
- [open-rs](https://github.com/Byron/open-rs)
- [rpassword](https://github.com/conradkleinespel/rpassword)
- [ssh2-rs](https://github.com/alexcrichton/ssh2-rs)
- [suppaftp](https://github.com/veeso/suppaftp)
- [textwrap](https://github.com/mgeisler/textwrap)
- [tui-rs](https://github.com/fdehau/tui-rs)
- [tui-realm](https://github.com/veeso/tui-realm)
- [whoami](https://github.com/libcala/whoami)
- [wildmatch](https://github.com/becheran/wildmatch)

---

## Gallery 🎬

> Termscp Home

![Auth](assets/images/auth.gif)

> Bookmarks

![Bookmarks](assets/images/bookmarks.gif)

> Setup

![Setup](assets/images/config.gif)

> Text editor

![TextEditor](assets/images/text-editor.gif)

---

## License 📃

termscp is licensed under the MIT license.

You can read the entire license [HERE](LICENSE)
