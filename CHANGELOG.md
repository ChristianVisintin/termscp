# Changelog

- [Changelog](#changelog)
  - [0.6.1](#061)
  - [0.6.0](#060)
  - [0.5.1](#051)
  - [0.5.0](#050)
  - [0.4.2](#042)
  - [0.4.1](#041)
  - [0.4.0](#040)
  - [0.3.3](#033)
  - [0.3.2](#032)
  - [0.3.1](#031)
  - [0.3.0](#030)
  - [0.2.0](#020)
  - [0.1.3](#013)
  - [0.1.2](#012)
  - [0.1.1](#011)
  - [0.1.0](#010)

---

## 0.6.1

Released on 31/08/2021

- Enhancements:
  - Now that tui-rs supports title alignment, UI has been improved
  - Added new `Directory already exists` variant for file transfer errors
- Bugfix:
  - Fixed [Issue 58](https://github.com/veeso/termscp/issues/58):When uploading a directory, create directory only if it doesn't exist
  - Fixed [Issue 59](https://github.com/veeso/termscp/issues/59): When copying files with tricky copy, the upper progress bar shows no text
- Dependencies:
  - Updated `bitflags` to `1.3.2`
  - Updated `bytesize` to `1.1.0`
  - Updated `crossterm` to `0.20`
  - Updated `open` to `2.0.1`
  - Added `tui-realm-stdlib 0.6.0`
  - Replaced `ftp4` with `suppaftp 4.1.2`
  - Updated `tui-realm` to `0.6.0`

## 0.6.0

Released on 23/07/2021

> 🍹 Summer update 2021 🍨

- **Open any file** in explorer:
  - Open file with default program for file type with `<V>`
  - Open file with a specific program with `<W>`
- **Themes**:
  - You can now set colors for 26 elements in the application
  - Colors can be any RGB, also **CSS colors** syntax is supported (e.g. `aquamarine`)
  - Configure theme from settings or import from CLI using the `-t <theme file>` argument
  - You can find several themes in the `themes/` directory
- **Keyring support for Linux**
  - From now on keyring will be available for Linux only
  - Read the manual to find out if your system supports the keyring and how you can enable it
  - libdbus is now a dependency
  - added `with-keyring` feature
  - **❗ BREAKING CHANGE ❗**: if you start using keyring on Linux, all the saved password will be lost
- **In-app release notes**
  - Possibility to see the release note of the new available release whenever a new version is available
  - Just press `<CTRL+R>` when a new version is available from the auth activity to read the release notes
- **Installation script**:
  - From now on, in case cargo is used to install termscp, all the cargo dependencies will be installed
- **Start termscp from configuration**: Start termscp with `-c` or `--config` to start termscp from configuration page
- Enhancements:
  - Show a "wait" message when deleting, copying and moving files and when executing commands
  - Replaced all `...` with `…` in texts
  - Check if remote host is valid in authentication form
  - Check if port number is valid in authentication form
  - From now on, if you try to leave setup without making any change, you won't be prompted whether to save configuration or not
- Bugfix:
  - Fixed broken input cursor when typing UTF8 characters (tui-realm 0.3.2)
  - Fixed save bookmark dialog: you could switch out from dialog with `<TAB>`
  - Fixed transfer interruption: it was not possible to abort a transfer if the size of the file was less than 65k
  - Changed `Remote address` to `Remote host` in authentication form
- Dependencies:
  - Added `argh 0.1.5`
  - Added `open 1.7.0`
  - Removed `getopts`
  - Updated `rand` to `0.8.4`
  - Updated `textwrap` to `0.14.2`
  - Updated `tui-realm` to `0.4.3`

## 0.5.1

Released on 21/06/2021

- Enhancements:
  - **CI now uses containers to test file transfers (SSH/FTP)**
    - Improved coverage
    - Found many bugs which has now been fixed
    - Build in CI won't fail due to test servers not responding
    - We're now able to test all the functionalities of the file transfers
  - **Status bar improvements**
    - "Show hidden files" in status bar
    - Status bar has now been splitted into two, one for each explorer tab
  - **Error message if terminal window is too small**
    - If the terminal window has less than 24 lines, then an error message is displayed in the auth activity
    - Changed auth layout to absolute sizes
- Bugfix:
  - Fixed UI not showing connection errors
  - Fixed termscp on Windows dying whenever opening a file with text editor
  - Fixed broken input cursor when typing UTF8 characters (tui-realm 0.3.2)
  - Fixed [Issue 44](https://github.com/veeso/termscp/issues/44): Could not move files to other paths in FTP
  - Fixed [Issue 43](https://github.com/veeso/termscp/issues/43): Could not remove non-empty directories in FTP
  - Fixed [Issue 39](https://github.com/veeso/termscp/issues/39): Help panels as `ScrollTable` to allow displaying entire content on small screens
  - Fixed [Issue 38](https://github.com/veeso/termscp/issues/38): Transfer size was wrong when transferring "selected" files (with mark)
  - Fixed [Issue 37](https://github.com/veeso/termscp/issues/37): progress bar not visible when editing remote files
- Dependencies:
  - Updated `textwrap` to `0.14.0`
  - Updated `tui-realm` to `0.4.2`

## 0.5.0

Released on 23/05/2021

> 🌸 Spring Update 2021 🌷

- **Synchronized browsing**:
  - Added the possibility to enabled the synchronized brower navigation
    - when you enter a directory, the same directory will be entered on the other tab
    - Enable sync browser with `<Y>`
    - Read more on manual: [Synchronized browsing](docs/man.md#Synchronized-browsing-)
- **Remote and Local hosts file formatter**:
  - Added the possibility to set different formatters for local and remote hosts
- **Work on multiple files**:
  - Added the possibility to work on **multiple files simultaneously**
  - Select a file with `<M>`, the file when selected will have a `*` prepended to its name
  - Select all files in the current directory with `<CTRL+A>`
  - Read more on manual: [Work on multiple files](docs/man.md#Work-on-multiple-files-)
- **Logging**:
  - termscp now writes a log file, useful to debug and to contribute to fix issues.
  - Read more on [manual](docs/man.md)
- **File transfer changes**
  - *SFTP*
    - Added **COPY** command to SFTP (Please note that Copy command is not supported by SFTP natively, so here it just uses the `cp` shell command as it does in SCP).
  - *FTP*
    - Added support for file copy (achieved through *tricky-copy*: the file is first downloaded, then uploaded with a different file name)
- **Double progress bar**:
  - From now one two progress bar will be displayed:
    - the first, on top, displays the full transfer state (e.g. when downloading a directory of 10 files, the progress of the entire transfer)
    - the second, on bottom, displays the transfer of the individual file being written (as happened for the old versions)
    - changed the progress bar colour from `LightGreen` to `Green`
- Enhancements
  - Added a status bar in the file explorer showing whether the sync browser is enabled and which file sorting mode is selected
  - Removed the goold old figlet title
  - Protocol input as first field in UI
  - Port is now updated to standard for selected protocol
    - when you change the protocol in the authentication form and the current port is standard (`< 1024`), the port will be automatically changed to default value for the selected protocol (e.g. current port: `123`, protocol changed to `FTP`, port becomes `21`)
- Bugfix:
  - Fixed wrong text wrap in log box
  - Fixed empty bookmark name causing termscp to crash
  - Fixed error message not being shown after an upload failure
  - Fixed default protocol not being loaded from config
  - [Issue 23](https://github.com/veeso/termscp/issues/23): Remove created file if transfer failed or was abrupted
- Dependencies:
  - Added `tui-realm 0.3.0`
  - Removed `tui` (as direct dependency)
  - Updated `regex` to `1.5.4`

## 0.4.2

Released on 13/04/2021

- Enhancements:
  - Use highlight symbol for logbox of `tui-rs` instead of adding a `Span`
- Bugfix:
  - removed `eprintln!` in ftp transfer causing UI to break in Windows

## 0.4.1

Released on 07/04/2021

- Enhancements:
  - SCP file transfer:
    - Added possibility to stat directories.
- Bugfix:
  - [Issue 18](https://github.com/veeso/termscp/issues/18): Set file transfer type to `Binary` for FTP
  - [Issue 17](https://github.com/veeso/termscp/issues/17)
    - SCP: fixed symlink not properly detected
    - FTP: added symlink support for Linux targets
  - [Issue 10](https://github.com/veeso/termscp/issues/10): Fixed port not being loaded from bookmarks into gui
  - [Issue 9](https://github.com/veeso/termscp/issues/9): Fixed issues related to paths on remote when using Windows
- Dependencies:
  - Added `path-slash 0.1.4` (Windows only)
  - Added `thiserror 1.0.24`
  - Updated `edit` to `0.1.3`
  - Updated `magic-crypt` to `3.1.7`
  - Updated `rand` to `0.8.3`
  - Updated `regex` to `1.4.5`
  - Updated `textwrap` to `0.13.4`
  - Updated `ureq` to `2.1.0`
  - Updated `whoami` to `1.1.1`
  - Updated `wildmatch` to `2.0.0`

## 0.4.0

Released on 27/03/2021

> The UI refactoring update

- **New explorer features**:
  - **Execute** a command pressing `X`. This feature is supported on both local and remote hosts (only SFTP/SCP protocols support this feature).
  - **Find**: search for files pressing `F` using wild matches.
- Enhancements:
  - Input fields will now support **"input keys"** (such as moving cursor, DEL, END, HOME, ...)
  - Improved performance regarding configuration I/O (config client is now shared in the activity context)
  - Fetch latest version from Github once; cache previous value in the Context Storage.
- Bugfix:
  - Prevent resetting explorer index on remote tab after performing certain actions (list dir, exec, ...)
  - SCP file transfer: prevent infinite loops while performing `stat` on symbolic links pointing to themselves (e.g. `mylink -> mylink`)
  - Fixed a bug causing termscp to crash if removing a bookmark
  - Fixed file format cursor position in the GUI
  - Fixed a bug causing termscp to show two equal bookmarks when overwriting one.
  - Fixed system tests which deleted the termscp configuration when launched
- **LICENSE**: changed license to MIT
- Dependencies:
  - Removed `unicode-width`
  - Added `wildmatch 1.0.13`
- For developers:
  - Activity refactoring
    - Developed an internal library used to create components, components are then nested inside a View
    - The new engine works through properties and states, then returns Messages. I was inspired by both React and Elm.

## 0.3.3

Released on 28/02/2021

- **Format key attributes**:
  - Added `EXTRA` and `LENGTH` parameters to format keys.
  - Now keys are provided with this syntax `{KEY_NAME[:LEN[:EXTRA]}`
- **Check for updates**:
  - termscp will now check for updates on startup and will show in the main page if there is a new version available
  - This feature may be disabled from setup (Check for updates => No)
- Enhancements:
  - Default choice for deleting file set to "NO" (way too easy to delete files by mistake)
  - Added CLI options to set starting workind directory on both local and remote hosts
  - Parse remote host now uses a Regex to gather parts (increased stability).
  - Now bookmarks and recents are sorted in the UI (bookmarks are sorted by name; recents are sorted by connection datetime)
  - Improved stability

## 0.3.2

Released on 24/01/2021

- **Explorer Formatter**:
  - Added possibility to customize the format when listing files in the explorers (Read more on README)
  - Added `file_fmt` key to configuration (if missing, default will be used).
  - Added the text input to the Settings view to set the value for `file_fmt`.
- Bugfix:
  - Solved file index in explorer files at start of termscp, in case the first entry is an hidden file
  - SCP File transfer: when listing directory entries, check if a symlink points to a directory or to a file
- Dependencies:
  - updated `crossterm` to `0.19.0`
  - updated `rand` to `0.8.2`
  - updated `rpassword` to `5.0.1`
  - updated `serde` to `1.0.121`
  - updated `tui` to `0.14.0`
  - updated `whoami` to `1.1.0`

## 0.3.1

Released on 18/01/2021

- **Keyring to store secrets**
  - On both MacOS and Windows, the secret used to encrypt passwords in bookmarks it is now store in the OS secret vault. This provides much more security to store the password
- Enhancements:
  - Added connection timeout to 30 seconds to SFTP/SCP clients and improved name lookup system.
- Bugfix:
  - Solved index in explorer files list which was no more kept after 0.3.0
  - SCP file transfer: fixed possible wrong file size when sending file, due to a possible incoherent size between the file explorer and the actual file size.
- Breaking changes: on **MacOS / Windows systems only**, the password you saved for bookmarks won't be working anymore if you have support for the keyring crate. Because of the migration to keyring, the previously used secret hasn't been migrated to the storage, instead a new secret will be used. To solve this, just save the bookmark again with the password.

## 0.3.0

 Released on 10/01/2021

> The SSH Key Storage Update

- **SSH Key Storage**
  - Added the possibility to store SSH private keys to access to remote hosts; this feature is supported in both SFTP and SCP.
  - SSH Keys can be manipulated through the new **Setup Interface**
- **Setup Interface**
  - Added a new area in the interface, where is possible to customize termscp. Access to this interface is achieved pressing `<CTRL+C>` from the home page (`AuthActivity`).
- **Configuration**:
  - Added configuration; configuration is stored at
    - Linux: `/home/alice/.config/termscp/config.toml`
    - MacOS: `/Users/Alice/Library/Application Support/termscp/config.toml`
    - Windows: `C:\Users\Alice\AppData\Roaming\termscp\config.toml`
  - Added Text editor to configuration
  - Added Default File transfer protocol to configuration
  - Added "Show hidden files" to configuration
  - Added "Group directories" to configuration
  - Added SSH keys to configuration; SSH keys will be stored at
    - Linux: `/home/alice/.config/termscp/.ssh/`
    - MacOS: `/Users/Alice/Library/Application Support/termscp/.ssh/`
    - Windows: `C:\Users\Alice\AppData\Roaming\termscp\.ssh\`
- Enhancements:
  - Replaced `sha256` sum with last modification time check, to verify if a file has been changed in the text editor
  - **FTP**
    - Added `LIST` command parser for Windows server (DOS-like syntax)
  - Default protocol changed to default protocol in configuration when providing address as CLI argument
  - Explorers:
    - Hidden files are now not shown by default; use `A` to show hidden files.
    - Append `/` to directories name.
- Keybindings:
  - `A`: Toggle hidden files
  - `B`: Sort files by (name, size, creation time, modify time)
  - `N`: New file
- Bugfix:
  - SCP client didn't show file types for files
  - FTP client didn't show file types for files
  - FTP file transfer not working properly with `STOR` and `RETR`.
  - Fixed `0 B/S` transfer rate displayed after completing download in less than 1 second
- Dependencies:
  - added `bitflags 1.2.1`
  - removed `data-encoding`
  - updated `ftp` to `4.0.2`
  - updated `rand` to `0.8.0`
  - removed `ring`
  - updated `textwrap` to `0.13.1`
  - updated `toml` to `0.5.8`
  - updated `whoami` to `1.0.1`

## 0.2.0

Released on 21/12/2020

> The Bookmarks Update

- **Bookmarks**
  - Bookmarks and recent connections are now displayed in the home page
  - Bookmarks are saved at
    - Linux: `/home/alice/.config/termscp/bookmarks.toml`
    - MacOS: `/Users/Alice/Library/Application Support/termscp/bookmarks.toml`
    - Windows: `C:\Users\Alice\AppData\Roaming\termscp\bookmarks.toml`
- **Text Editor**
  - Added text editor feature to explorer view
  - Added `o` to keybindings to open a text file
- Keybindings:
  - `C`: Copy file/directory
  - `O`: Open text file in editor
- Enhancements:
  - User interface
    - Collpased borders to make everything more *aesthetic*
    - Rounded input field boards
    - File explorer:
      - Log how long it took to upload/download a file and the transfer speed
      - Display in progress bar the transfer speed (bytes/seconds)
- Bugfix:
  - File mode of file on remote is now reported on local file after being downloaded (unix, linux, macos only)
  - Scp: when username was not provided, it didn't fallback to current username
  - Explorer: fixed UID format in Windows

## 0.1.3

Released on 14/12/2020

- Enhancements:
  - File transfer:
    - Read buffer is now 65536 bytes long
  - File explorer:
    - Fixed color mismatch in local explorer
    - Explorer tabs have now 70% of layout height, while logging area is 30%
    - Highlight selected entry in tabs, only when the tab is active
  - Auth page:
    - align popup text to center
- Keybindings:
  - `L`: Refresh directory content
- Bugfix:
  - Fixed memory vulnerability in Windows version

## 0.1.2

Released on 13/12/2020

- General performance and code improvements
- Improved symlinks management
- Possibility to abort file transfers
- Enhancements:
  - File explorer:
    - When file index is at the end of the list, moving down will set the current index to the first element and viceversa.
    - Selected file has now colourful background, instead of foreground, for a better readability.
- Keybindings:
  - `E`: Delete file (Same as `DEL`); added because some keyboards don't have `DEL` (hey, that's my MacBook Air's keyboard!)
  - `Ctrl+C`: Abort transfer process

## 0.1.1

Released on 10/12/2020

- enhancements:
  - password prompt: ask before performing terminal clear
  - file explorer:
    - file names are now sorted ignoring capital letters
    - file names longer than 23, are now cut to 20 and followed by `...`
    - paths which exceed tab size in explorer are elided with the following formato `ANCESTOR[1]/.../PARENT/DIRNAME`
- keybindings:
  - `I`: show info about selected file or directory
  - Removed `CTRL`; just use keys now.
- bugfix:
  - prevent panic in set_progress, for progress values `> 100.0 or < 0.0`
  - Fixed FTP get, which didn't finalize the reader
- dependencies:
  - updated `textwrap` to `0.13.0`
  - updated `ftp4` to `4.0.1`

## 0.1.0

Released on 06/12/2020

- First release
