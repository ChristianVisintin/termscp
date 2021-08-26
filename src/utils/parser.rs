//! ## Parser
//!
//! `parser` is the module which provides utilities for parsing different kind of stuff

/**
 * MIT License
 *
 * termscp - Copyright (c) 2021 Christian Visintin
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
// Locals
use crate::filetransfer::{FileTransferParams, FileTransferProtocol};
#[cfg(not(test))] // NOTE: don't use configuration during tests
use crate::system::config_client::ConfigClient;
#[cfg(not(test))] // NOTE: don't use configuration during tests
use crate::system::environment;

// Ext
use chrono::format::ParseError;
use chrono::prelude::*;
use regex::Regex;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use tuirealm::tui::style::Color;

// Regex
lazy_static! {
    /**
     * Regex matches:
     *  - group 1: Some(protocol) | None
     *  - group 2: Some(user) | None
     *  - group 3: Address
     *  - group 4: Some(port) | None
     *  - group 5: Some(path) | None
     */
    static ref REMOTE_OPT_REGEX: Regex = Regex::new(r"(?:([a-z]+)://)?(?:([^@]+)@)?(?:([^:]+))(?::((?:[0-9]{1,4}|[1-5][0-9]{4}|6[0-4][0-9]{3}|65[0-4][0-9]{2}|655[0-2][0-9]|6553[0-5])(?:[0-9]{1,4}|[1-5][0-9]{4}|6[0-4][0-9]{3}|65[0-4][0-9]{2}|655[0-2][0-9]|6553[0-5])))?(?::([^:]+))?").ok().unwrap();
    /**
     * Regex matches:
     * - group 1: Version
     * E.g. termscp-0.3.2 => 0.3.2
     *      v0.4.0 => 0.4.0
     */
    static ref SEMVER_REGEX: Regex = Regex::new(r".*(:?[0-9]\.[0-9]\.[0-9])").unwrap();
    /**
     * Regex matches:
     * - group 1: Red
     * - group 2: Green
     * - group 3: Blue
     */
    static ref COLOR_HEX_REGEX: Regex = Regex::new(r"#(:?[0-9a-fA-F]{2})(:?[0-9a-fA-F]{2})(:?[0-9a-fA-F]{2})").unwrap();
    /**
     * Regex matches:
     * - group 2: Red
     * - group 4: Green
     * - group 6: blue
     */
    static ref COLOR_RGB_REGEX: Regex = Regex::new(r"^(rgb)?\(?([01]?\d\d?|2[0-4]\d|25[0-5])(\W+)([01]?\d\d?|2[0-4]\d|25[0-5])\W+(([01]?\d\d?|2[0-4]\d|25[0-5])\)?)").unwrap();
}

/// ### parse_remote_opt
///
/// Parse remote option string. Returns in case of success a RemoteOptions struct
/// For ssh if username is not provided, current user will be used.
/// In case of error, message is returned
/// If port is missing default port will be used for each protocol
///     SFTP => 22
///     FTP => 21
/// The option string has the following syntax
/// [protocol://][username@]{address}[:port][:path]
/// The only argument which is mandatory is address
/// NOTE: possible strings
/// - 172.26.104.1
/// - root@172.26.104.1
/// - sftp://root@172.26.104.1
/// - sftp://172.26.104.1:4022
/// - sftp://172.26.104.1
/// - ...
pub fn parse_remote_opt(remote: &str) -> Result<FileTransferParams, String> {
    // Set protocol to default protocol
    #[cfg(not(test))] // NOTE: don't use configuration during tests
    let mut protocol: FileTransferProtocol = match environment::init_config_dir() {
        Ok(p) => match p {
            Some(p) => {
                // Create config client
                let (config_path, ssh_key_path) = environment::get_config_paths(p.as_path());
                match ConfigClient::new(config_path.as_path(), ssh_key_path.as_path()) {
                    Ok(cli) => cli.get_default_protocol(),
                    Err(_) => FileTransferProtocol::Sftp,
                }
            }
            None => FileTransferProtocol::Sftp,
        },
        Err(_) => FileTransferProtocol::Sftp,
    };
    #[cfg(test)] // NOTE: during test set protocol just to Sftp
    let mut protocol: FileTransferProtocol = FileTransferProtocol::Sftp;
    // Match against regex
    match REMOTE_OPT_REGEX.captures(remote) {
        Some(groups) => {
            // Match protocol
            let mut port: u16 = 22;
            if let Some(group) = groups.get(1) {
                // Set protocol from group
                let (m_protocol, m_port) = match FileTransferProtocol::from_str(group.as_str()) {
                    Ok(proto) => match proto {
                        FileTransferProtocol::Ftp(_) => (proto, 21),
                        FileTransferProtocol::Scp => (proto, 22),
                        FileTransferProtocol::Sftp => (proto, 22),
                    },
                    Err(_) => return Err(format!("Unknown protocol \"{}\"", group.as_str())),
                };
                // NOTE: tuple destructuring assignment is not supported yet :(
                protocol = m_protocol;
                port = m_port;
            }
            // Match user
            let username: Option<String> = match groups.get(2) {
                Some(group) => Some(group.as_str().to_string()),
                None => match protocol {
                    // If group is empty, set to current user
                    FileTransferProtocol::Scp | FileTransferProtocol::Sftp => {
                        Some(whoami::username())
                    }
                    _ => None,
                },
            };
            // Get address
            let address: String = match groups.get(3) {
                Some(group) => group.as_str().to_string(),
                None => return Err(String::from("Missing address")),
            };
            // Get port
            if let Some(group) = groups.get(4) {
                port = match group.as_str().parse::<u16>() {
                    Ok(p) => p,
                    Err(err) => return Err(format!("Bad port \"{}\": {}", group.as_str(), err)),
                };
            }
            // Get workdir
            let entry_directory: Option<PathBuf> =
                groups.get(5).map(|group| PathBuf::from(group.as_str()));
            Ok(FileTransferParams::new(address)
                .port(port)
                .protocol(protocol)
                .username(username)
                .entry_directory(entry_directory))
        }
        None => Err(String::from("Bad remote host syntax!")),
    }
}

/// ### parse_lstime
///
/// Convert ls syntax time to System Time
/// ls time has two possible syntax:
/// 1. if year is current: %b %d %H:%M (e.g. Nov 5 13:46)
/// 2. else: %b %d %Y (e.g. Nov 5 2019)
pub fn parse_lstime(tm: &str, fmt_year: &str, fmt_hours: &str) -> Result<SystemTime, ParseError> {
    let datetime: NaiveDateTime = match NaiveDate::parse_from_str(tm, fmt_year) {
        Ok(date) => {
            // Case 2.
            // Return NaiveDateTime from NaiveDate with time 00:00:00
            date.and_hms(0, 0, 0)
        }
        Err(_) => {
            // Might be case 1.
            // We need to add Current Year at the end of the string
            let this_year: i32 = Utc::now().year();
            let date_time_str: String = format!("{} {}", tm, this_year);
            // Now parse
            NaiveDateTime::parse_from_str(
                date_time_str.as_ref(),
                format!("{} %Y", fmt_hours).as_ref(),
            )?
        }
    };
    // Convert datetime to system time
    let sys_time: SystemTime = SystemTime::UNIX_EPOCH;
    Ok(sys_time
        .checked_add(Duration::from_secs(datetime.timestamp() as u64))
        .unwrap_or(SystemTime::UNIX_EPOCH))
}

/// ### parse_datetime
///
/// Parse date time string representation and transform it into `SystemTime`
#[allow(dead_code)]
pub fn parse_datetime(tm: &str, fmt: &str) -> Result<SystemTime, ParseError> {
    match NaiveDateTime::parse_from_str(tm, fmt) {
        Ok(dt) => {
            let sys_time: SystemTime = SystemTime::UNIX_EPOCH;
            Ok(sys_time
                .checked_add(Duration::from_secs(dt.timestamp() as u64))
                .unwrap_or(SystemTime::UNIX_EPOCH))
        }
        Err(err) => Err(err),
    }
}

/// ### parse_semver
///
/// Parse semver string
pub fn parse_semver(haystack: &str) -> Option<String> {
    match SEMVER_REGEX.captures(haystack) {
        Some(groups) => groups.get(1).map(|version| version.as_str().to_string()),
        None => None,
    }
}

/// ### parse_color
///
/// Parse color from string into a `Color` enum.
///
/// Color may be in different format:
///
/// 1. color name:
///     - Black,
///     - Blue,
///     - Cyan,
///     - DarkGray,
///     - Gray,
///     - Green,
///     - LightBlue,
///     - LightCyan,
///     - LightGreen,
///     - LightMagenta,
///     - LightRed,
///     - LightYellow,
///     - Magenta,
///     - Red,
///     - Reset,
///     - White,
///     - Yellow,
/// 2. Hex format:
///     - #f0ab05
///     - #AA33BC
/// 3. Rgb format:
///     - rgb(255, 64, 32)
///     - rgb(255,64,32)
///     - 255, 64, 32
pub fn parse_color(color: &str) -> Option<Color> {
    match color.to_lowercase().as_str() {
        // -- lib colors
        "black" => Some(Color::Black),
        "blue" => Some(Color::Blue),
        "cyan" => Some(Color::Cyan),
        "darkgray" | "darkgrey" => Some(Color::DarkGray),
        "default" => Some(Color::Reset),
        "gray" => Some(Color::Gray),
        "green" => Some(Color::Green),
        "lightblue" => Some(Color::LightBlue),
        "lightcyan" => Some(Color::LightCyan),
        "lightgreen" => Some(Color::LightGreen),
        "lightmagenta" => Some(Color::LightMagenta),
        "lightred" => Some(Color::LightRed),
        "lightyellow" => Some(Color::LightYellow),
        "magenta" => Some(Color::Magenta),
        "red" => Some(Color::Red),
        "white" => Some(Color::White),
        "yellow" => Some(Color::Yellow),
        // -- css colors
        "aliceblue" => Some(Color::Rgb(240, 248, 255)),
        "antiquewhite" => Some(Color::Rgb(250, 235, 215)),
        "aqua" => Some(Color::Rgb(0, 255, 255)),
        "aquamarine" => Some(Color::Rgb(127, 255, 212)),
        "azure" => Some(Color::Rgb(240, 255, 255)),
        "beige" => Some(Color::Rgb(245, 245, 220)),
        "bisque" => Some(Color::Rgb(255, 228, 196)),
        "blanchedalmond" => Some(Color::Rgb(255, 235, 205)),
        "blueviolet" => Some(Color::Rgb(138, 43, 226)),
        "brown" => Some(Color::Rgb(165, 42, 42)),
        "burlywood" => Some(Color::Rgb(222, 184, 135)),
        "cadetblue" => Some(Color::Rgb(95, 158, 160)),
        "chartreuse" => Some(Color::Rgb(127, 255, 0)),
        "chocolate" => Some(Color::Rgb(210, 105, 30)),
        "coral" => Some(Color::Rgb(255, 127, 80)),
        "cornflowerblue" => Some(Color::Rgb(100, 149, 237)),
        "cornsilk" => Some(Color::Rgb(255, 248, 220)),
        "crimson" => Some(Color::Rgb(220, 20, 60)),
        "darkblue" => Some(Color::Rgb(0, 0, 139)),
        "darkcyan" => Some(Color::Rgb(0, 139, 139)),
        "darkgoldenrod" => Some(Color::Rgb(184, 134, 11)),
        "darkgreen" => Some(Color::Rgb(0, 100, 0)),
        "darkkhaki" => Some(Color::Rgb(189, 183, 107)),
        "darkmagenta" => Some(Color::Rgb(139, 0, 139)),
        "darkolivegreen" => Some(Color::Rgb(85, 107, 47)),
        "darkorange" => Some(Color::Rgb(255, 140, 0)),
        "darkorchid" => Some(Color::Rgb(153, 50, 204)),
        "darkred" => Some(Color::Rgb(139, 0, 0)),
        "darksalmon" => Some(Color::Rgb(233, 150, 122)),
        "darkseagreen" => Some(Color::Rgb(143, 188, 143)),
        "darkslateblue" => Some(Color::Rgb(72, 61, 139)),
        "darkslategray" | "darkslategrey" => Some(Color::Rgb(47, 79, 79)),
        "darkturquoise" => Some(Color::Rgb(0, 206, 209)),
        "darkviolet" => Some(Color::Rgb(148, 0, 211)),
        "deeppink" => Some(Color::Rgb(255, 20, 147)),
        "deepskyblue" => Some(Color::Rgb(0, 191, 255)),
        "dimgray" | "dimgrey" => Some(Color::Rgb(105, 105, 105)),
        "dodgerblue" => Some(Color::Rgb(30, 144, 255)),
        "firebrick" => Some(Color::Rgb(178, 34, 34)),
        "floralwhite" => Some(Color::Rgb(255, 250, 240)),
        "forestgreen" => Some(Color::Rgb(34, 139, 34)),
        "fuchsia" => Some(Color::Rgb(255, 0, 255)),
        "gainsboro" => Some(Color::Rgb(220, 220, 220)),
        "ghostwhite" => Some(Color::Rgb(248, 248, 255)),
        "gold" => Some(Color::Rgb(255, 215, 0)),
        "goldenrod" => Some(Color::Rgb(218, 165, 32)),
        "greenyellow" => Some(Color::Rgb(173, 255, 47)),
        "grey" => Some(Color::Rgb(128, 128, 128)),
        "honeydew" => Some(Color::Rgb(240, 255, 240)),
        "hotpink" => Some(Color::Rgb(255, 105, 180)),
        "indianred" => Some(Color::Rgb(205, 92, 92)),
        "indigo" => Some(Color::Rgb(75, 0, 130)),
        "ivory" => Some(Color::Rgb(255, 255, 240)),
        "khaki" => Some(Color::Rgb(240, 230, 140)),
        "lavender" => Some(Color::Rgb(230, 230, 250)),
        "lavenderblush" => Some(Color::Rgb(255, 240, 245)),
        "lawngreen" => Some(Color::Rgb(124, 252, 0)),
        "lemonchiffon" => Some(Color::Rgb(255, 250, 205)),
        "lightcoral" => Some(Color::Rgb(240, 128, 128)),
        "lightgoldenrodyellow" => Some(Color::Rgb(250, 250, 210)),
        "lightgray" | "lightgrey" => Some(Color::Rgb(211, 211, 211)),
        "lightpink" => Some(Color::Rgb(255, 182, 193)),
        "lightsalmon" => Some(Color::Rgb(255, 160, 122)),
        "lightseagreen" => Some(Color::Rgb(32, 178, 170)),
        "lightskyblue" => Some(Color::Rgb(135, 206, 250)),
        "lightslategray" | "lightslategrey" => Some(Color::Rgb(119, 136, 153)),
        "lightsteelblue" => Some(Color::Rgb(176, 196, 222)),
        "lime" => Some(Color::Rgb(0, 255, 0)),
        "limegreen" => Some(Color::Rgb(50, 205, 50)),
        "linen" => Some(Color::Rgb(250, 240, 230)),
        "maroon" => Some(Color::Rgb(128, 0, 0)),
        "mediumaquamarine" => Some(Color::Rgb(102, 205, 170)),
        "mediumblue" => Some(Color::Rgb(0, 0, 205)),
        "mediumorchid" => Some(Color::Rgb(186, 85, 211)),
        "mediumpurple" => Some(Color::Rgb(147, 112, 219)),
        "mediumseagreen" => Some(Color::Rgb(60, 179, 113)),
        "mediumslateblue" => Some(Color::Rgb(123, 104, 238)),
        "mediumspringgreen" => Some(Color::Rgb(0, 250, 154)),
        "mediumturquoise" => Some(Color::Rgb(72, 209, 204)),
        "mediumvioletred" => Some(Color::Rgb(199, 21, 133)),
        "midnightblue" => Some(Color::Rgb(25, 25, 112)),
        "mintcream" => Some(Color::Rgb(245, 255, 250)),
        "mistyrose" => Some(Color::Rgb(255, 228, 225)),
        "moccasin" => Some(Color::Rgb(255, 228, 181)),
        "navajowhite" => Some(Color::Rgb(255, 222, 173)),
        "navy" => Some(Color::Rgb(0, 0, 128)),
        "oldlace" => Some(Color::Rgb(253, 245, 230)),
        "olive" => Some(Color::Rgb(128, 128, 0)),
        "olivedrab" => Some(Color::Rgb(107, 142, 35)),
        "orange" => Some(Color::Rgb(255, 165, 0)),
        "orangered" => Some(Color::Rgb(255, 69, 0)),
        "orchid" => Some(Color::Rgb(218, 112, 214)),
        "palegoldenrod" => Some(Color::Rgb(238, 232, 170)),
        "palegreen" => Some(Color::Rgb(152, 251, 152)),
        "paleturquoise" => Some(Color::Rgb(175, 238, 238)),
        "palevioletred" => Some(Color::Rgb(219, 112, 147)),
        "papayawhip" => Some(Color::Rgb(255, 239, 213)),
        "peachpuff" => Some(Color::Rgb(255, 218, 185)),
        "peru" => Some(Color::Rgb(205, 133, 63)),
        "pink" => Some(Color::Rgb(255, 192, 203)),
        "plum" => Some(Color::Rgb(221, 160, 221)),
        "powderblue" => Some(Color::Rgb(176, 224, 230)),
        "purple" => Some(Color::Rgb(128, 0, 128)),
        "rebeccapurple" => Some(Color::Rgb(102, 51, 153)),
        "rosybrown" => Some(Color::Rgb(188, 143, 143)),
        "royalblue" => Some(Color::Rgb(65, 105, 225)),
        "saddlebrown" => Some(Color::Rgb(139, 69, 19)),
        "salmon" => Some(Color::Rgb(250, 128, 114)),
        "sandybrown" => Some(Color::Rgb(244, 164, 96)),
        "seagreen" => Some(Color::Rgb(46, 139, 87)),
        "seashell" => Some(Color::Rgb(255, 245, 238)),
        "sienna" => Some(Color::Rgb(160, 82, 45)),
        "silver" => Some(Color::Rgb(192, 192, 192)),
        "skyblue" => Some(Color::Rgb(135, 206, 235)),
        "slateblue" => Some(Color::Rgb(106, 90, 205)),
        "slategray" | "slategrey" => Some(Color::Rgb(112, 128, 144)),
        "snow" => Some(Color::Rgb(255, 250, 250)),
        "springgreen" => Some(Color::Rgb(0, 255, 127)),
        "steelblue" => Some(Color::Rgb(70, 130, 180)),
        "tan" => Some(Color::Rgb(210, 180, 140)),
        "teal" => Some(Color::Rgb(0, 128, 128)),
        "thistle" => Some(Color::Rgb(216, 191, 216)),
        "tomato" => Some(Color::Rgb(255, 99, 71)),
        "turquoise" => Some(Color::Rgb(64, 224, 208)),
        "violet" => Some(Color::Rgb(238, 130, 238)),
        "wheat" => Some(Color::Rgb(245, 222, 179)),
        "whitesmoke" => Some(Color::Rgb(245, 245, 245)),
        "yellowgreen" => Some(Color::Rgb(154, 205, 50)),
        // -- hex and rgb
        other => {
            // Try as hex
            if let Some(color) = parse_hex_color(other) {
                Some(color)
            } else {
                parse_rgb_color(other)
            }
        }
    }
}

/// ### parse_hex_color
///
/// Try to parse a color in hex format, such as:
///
///     - #f0ab05
///     - #AA33BC
fn parse_hex_color(color: &str) -> Option<Color> {
    COLOR_HEX_REGEX.captures(color).map(|groups| {
        Color::Rgb(
            u8::from_str_radix(groups.get(1).unwrap().as_str(), 16)
                .ok()
                .unwrap(),
            u8::from_str_radix(groups.get(2).unwrap().as_str(), 16)
                .ok()
                .unwrap(),
            u8::from_str_radix(groups.get(3).unwrap().as_str(), 16)
                .ok()
                .unwrap(),
        )
    })
}

/// ### parse_rgb_color
///
/// Try to parse a color in rgb format, such as:
///
///     - rgb(255, 64, 32)
///     - rgb(255,64,32)
///     - 255, 64, 32
fn parse_rgb_color(color: &str) -> Option<Color> {
    COLOR_RGB_REGEX.captures(color).map(|groups| {
        Color::Rgb(
            u8::from_str(groups.get(2).unwrap().as_str()).ok().unwrap(),
            u8::from_str(groups.get(4).unwrap().as_str()).ok().unwrap(),
            u8::from_str(groups.get(6).unwrap().as_str()).ok().unwrap(),
        )
    })
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::utils::fmt::fmt_time;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_utils_parse_remote_opt() {
        // Base case
        let result: FileTransferParams = parse_remote_opt(&String::from("172.26.104.1"))
            .ok()
            .unwrap();
        assert_eq!(result.address, String::from("172.26.104.1"));
        assert_eq!(result.port, 22);
        assert_eq!(result.protocol, FileTransferProtocol::Sftp);
        assert!(result.username.is_some());
        // User case
        let result: FileTransferParams = parse_remote_opt(&String::from("root@172.26.104.1"))
            .ok()
            .unwrap();
        assert_eq!(result.address, String::from("172.26.104.1"));
        assert_eq!(result.port, 22);
        assert_eq!(result.protocol, FileTransferProtocol::Sftp);
        assert_eq!(result.username.unwrap(), String::from("root"));
        assert!(result.entry_directory.is_none());
        // User + port
        let result: FileTransferParams = parse_remote_opt(&String::from("root@172.26.104.1:8022"))
            .ok()
            .unwrap();
        assert_eq!(result.address, String::from("172.26.104.1"));
        assert_eq!(result.port, 8022);
        assert_eq!(result.protocol, FileTransferProtocol::Sftp);
        assert_eq!(result.username.unwrap(), String::from("root"));
        assert!(result.entry_directory.is_none());
        // Port only
        let result: FileTransferParams = parse_remote_opt(&String::from("172.26.104.1:4022"))
            .ok()
            .unwrap();
        assert_eq!(result.address, String::from("172.26.104.1"));
        assert_eq!(result.port, 4022);
        assert_eq!(result.protocol, FileTransferProtocol::Sftp);
        assert!(result.username.is_some());
        assert!(result.entry_directory.is_none());
        // Protocol
        let result: FileTransferParams = parse_remote_opt(&String::from("ftp://172.26.104.1"))
            .ok()
            .unwrap();
        assert_eq!(result.address, String::from("172.26.104.1"));
        assert_eq!(result.port, 21); // Fallback to ftp default
        assert_eq!(result.protocol, FileTransferProtocol::Ftp(false));
        assert!(result.username.is_none()); // Doesn't fall back
        assert!(result.entry_directory.is_none());
        // Protocol
        let result: FileTransferParams = parse_remote_opt(&String::from("sftp://172.26.104.1"))
            .ok()
            .unwrap();
        assert_eq!(result.address, String::from("172.26.104.1"));
        assert_eq!(result.port, 22); // Fallback to sftp default
        assert_eq!(result.protocol, FileTransferProtocol::Sftp);
        assert!(result.username.is_some()); // Doesn't fall back
        assert!(result.entry_directory.is_none());
        let result: FileTransferParams = parse_remote_opt(&String::from("scp://172.26.104.1"))
            .ok()
            .unwrap();
        assert_eq!(result.address, String::from("172.26.104.1"));
        assert_eq!(result.port, 22); // Fallback to scp default
        assert_eq!(result.protocol, FileTransferProtocol::Scp);
        assert!(result.username.is_some()); // Doesn't fall back
        assert!(result.entry_directory.is_none());
        // Protocol + user
        let result: FileTransferParams =
            parse_remote_opt(&String::from("ftps://anon@172.26.104.1"))
                .ok()
                .unwrap();
        assert_eq!(result.address, String::from("172.26.104.1"));
        assert_eq!(result.port, 21); // Fallback to ftp default
        assert_eq!(result.protocol, FileTransferProtocol::Ftp(true));
        assert_eq!(result.username.unwrap(), String::from("anon"));
        assert!(result.entry_directory.is_none());
        // Path
        let result: FileTransferParams =
            parse_remote_opt(&String::from("root@172.26.104.1:8022:/var"))
                .ok()
                .unwrap();
        assert_eq!(result.address, String::from("172.26.104.1"));
        assert_eq!(result.port, 8022);
        assert_eq!(result.protocol, FileTransferProtocol::Sftp);
        assert_eq!(result.username.unwrap(), String::from("root"));
        assert_eq!(result.entry_directory.unwrap(), PathBuf::from("/var"));
        // Port only
        let result: FileTransferParams = parse_remote_opt(&String::from("172.26.104.1:home"))
            .ok()
            .unwrap();
        assert_eq!(result.address, String::from("172.26.104.1"));
        assert_eq!(result.port, 22);
        assert_eq!(result.protocol, FileTransferProtocol::Sftp);
        assert!(result.username.is_some());
        assert_eq!(result.entry_directory.unwrap(), PathBuf::from("home"));
        // All together now
        let result: FileTransferParams =
            parse_remote_opt(&String::from("ftp://anon@172.26.104.1:8021:/tmp"))
                .ok()
                .unwrap();
        assert_eq!(result.address, String::from("172.26.104.1"));
        assert_eq!(result.port, 8021); // Fallback to ftp default
        assert_eq!(result.protocol, FileTransferProtocol::Ftp(false));
        assert_eq!(result.username.unwrap(), String::from("anon"));
        assert_eq!(result.entry_directory.unwrap(), PathBuf::from("/tmp"));
        // bad syntax
        // Bad protocol
        assert!(parse_remote_opt(&String::from("omar://172.26.104.1")).is_err());
        // Bad port
        assert!(parse_remote_opt(&String::from("scp://172.26.104.1:650000")).is_err());
    }

    #[test]
    fn test_utils_parse_lstime() {
        // Good cases
        assert_eq!(
            fmt_time(
                parse_lstime("Nov 5 16:32", "%b %d %Y", "%b %d %H:%M")
                    .ok()
                    .unwrap(),
                "%m %d %M"
            )
            .as_str(),
            "11 05 32"
        );
        assert_eq!(
            fmt_time(
                parse_lstime("Dec 2 21:32", "%b %d %Y", "%b %d %H:%M")
                    .ok()
                    .unwrap(),
                "%m %d %M"
            )
            .as_str(),
            "12 02 32"
        );
        assert_eq!(
            parse_lstime("Nov 5 2018", "%b %d %Y", "%b %d %H:%M")
                .ok()
                .unwrap()
                .duration_since(SystemTime::UNIX_EPOCH)
                .ok()
                .unwrap(),
            Duration::from_secs(1541376000)
        );
        assert_eq!(
            parse_lstime("Mar 18 2018", "%b %d %Y", "%b %d %H:%M")
                .ok()
                .unwrap()
                .duration_since(SystemTime::UNIX_EPOCH)
                .ok()
                .unwrap(),
            Duration::from_secs(1521331200)
        );
        // bad cases
        assert!(parse_lstime("Oma 31 2018", "%b %d %Y", "%b %d %H:%M").is_err());
        assert!(parse_lstime("Feb 31 2018", "%b %d %Y", "%b %d %H:%M").is_err());
        assert!(parse_lstime("Feb 15 25:32", "%b %d %Y", "%b %d %H:%M").is_err());
    }

    #[test]
    fn test_utils_parse_datetime() {
        assert_eq!(
            parse_datetime("04-08-14  03:09PM", "%d-%m-%y %I:%M%p")
                .ok()
                .unwrap()
                .duration_since(SystemTime::UNIX_EPOCH)
                .ok()
                .unwrap(),
            Duration::from_secs(1407164940)
        );
        // Not enough argument for datetime
        assert!(parse_datetime("04-08-14", "%d-%m-%y").is_err());
    }

    #[test]
    fn test_utils_parse_semver() {
        assert_eq!(
            parse_semver("termscp-0.3.2").unwrap(),
            String::from("0.3.2")
        );
        assert_eq!(parse_semver("v0.4.1").unwrap(), String::from("0.4.1"),);
        assert_eq!(parse_semver("1.0.0").unwrap(), String::from("1.0.0"),);
        assert!(parse_semver("v1.1").is_none());
    }

    #[test]
    fn test_utils_parse_color_hex() {
        assert_eq!(
            parse_hex_color("#f0f0f0").unwrap(),
            Color::Rgb(240, 240, 240)
        );
        assert_eq!(
            parse_hex_color("#60AAcc").unwrap(),
            Color::Rgb(96, 170, 204)
        );
        assert!(parse_hex_color("#fatboy").is_none());
    }

    #[test]
    fn test_utils_parse_color_rgb() {
        assert_eq!(
            parse_rgb_color("rgb(255, 64, 32)").unwrap(),
            Color::Rgb(255, 64, 32)
        );
        assert_eq!(
            parse_rgb_color("rgb(255,64,32)").unwrap(),
            Color::Rgb(255, 64, 32)
        );
        assert_eq!(
            parse_rgb_color("(255,64,32)").unwrap(),
            Color::Rgb(255, 64, 32)
        );
        assert_eq!(
            parse_rgb_color("255,64,32").unwrap(),
            Color::Rgb(255, 64, 32)
        );
        assert!(parse_rgb_color("(300, 128, 512)").is_none());
    }

    #[test]
    fn test_utils_parse_color() {
        assert_eq!(parse_color("Black").unwrap(), Color::Black);
        assert_eq!(parse_color("BLUE").unwrap(), Color::Blue);
        assert_eq!(parse_color("Cyan").unwrap(), Color::Cyan);
        assert_eq!(parse_color("DarkGray").unwrap(), Color::DarkGray);
        assert_eq!(parse_color("Gray").unwrap(), Color::Gray);
        assert_eq!(parse_color("Green").unwrap(), Color::Green);
        assert_eq!(parse_color("LightBlue").unwrap(), Color::LightBlue);
        assert_eq!(parse_color("LightCyan").unwrap(), Color::LightCyan);
        assert_eq!(parse_color("LightGreen").unwrap(), Color::LightGreen);
        assert_eq!(parse_color("LightMagenta").unwrap(), Color::LightMagenta);
        assert_eq!(parse_color("LightRed").unwrap(), Color::LightRed);
        assert_eq!(parse_color("LightYellow").unwrap(), Color::LightYellow);
        assert_eq!(parse_color("Magenta").unwrap(), Color::Magenta);
        assert_eq!(parse_color("Red").unwrap(), Color::Red);
        assert_eq!(parse_color("Default").unwrap(), Color::Reset);
        assert_eq!(parse_color("White").unwrap(), Color::White);
        assert_eq!(parse_color("Yellow").unwrap(), Color::Yellow);
        assert_eq!(parse_color("#f0f0f0").unwrap(), Color::Rgb(240, 240, 240));
        // -- css colors
        assert_eq!(parse_color("aliceblue"), Some(Color::Rgb(240, 248, 255)));
        assert_eq!(parse_color("antiquewhite"), Some(Color::Rgb(250, 235, 215)));
        assert_eq!(parse_color("aqua"), Some(Color::Rgb(0, 255, 255)));
        assert_eq!(parse_color("aquamarine"), Some(Color::Rgb(127, 255, 212)));
        assert_eq!(parse_color("azure"), Some(Color::Rgb(240, 255, 255)));
        assert_eq!(parse_color("beige"), Some(Color::Rgb(245, 245, 220)));
        assert_eq!(parse_color("bisque"), Some(Color::Rgb(255, 228, 196)));
        assert_eq!(
            parse_color("blanchedalmond"),
            Some(Color::Rgb(255, 235, 205))
        );
        assert_eq!(parse_color("blueviolet"), Some(Color::Rgb(138, 43, 226)));
        assert_eq!(parse_color("brown"), Some(Color::Rgb(165, 42, 42)));
        assert_eq!(parse_color("burlywood"), Some(Color::Rgb(222, 184, 135)));
        assert_eq!(parse_color("cadetblue"), Some(Color::Rgb(95, 158, 160)));
        assert_eq!(parse_color("chartreuse"), Some(Color::Rgb(127, 255, 0)));
        assert_eq!(parse_color("chocolate"), Some(Color::Rgb(210, 105, 30)));
        assert_eq!(parse_color("coral"), Some(Color::Rgb(255, 127, 80)));
        assert_eq!(
            parse_color("cornflowerblue"),
            Some(Color::Rgb(100, 149, 237))
        );
        assert_eq!(parse_color("cornsilk"), Some(Color::Rgb(255, 248, 220)));
        assert_eq!(parse_color("crimson"), Some(Color::Rgb(220, 20, 60)));
        assert_eq!(parse_color("darkblue"), Some(Color::Rgb(0, 0, 139)));
        assert_eq!(parse_color("darkcyan"), Some(Color::Rgb(0, 139, 139)));
        assert_eq!(parse_color("darkgoldenrod"), Some(Color::Rgb(184, 134, 11)));
        assert_eq!(parse_color("darkgreen"), Some(Color::Rgb(0, 100, 0)));
        assert_eq!(parse_color("darkkhaki"), Some(Color::Rgb(189, 183, 107)));
        assert_eq!(parse_color("darkmagenta"), Some(Color::Rgb(139, 0, 139)));
        assert_eq!(parse_color("darkolivegreen"), Some(Color::Rgb(85, 107, 47)));
        assert_eq!(parse_color("darkorange"), Some(Color::Rgb(255, 140, 0)));
        assert_eq!(parse_color("darkorchid"), Some(Color::Rgb(153, 50, 204)));
        assert_eq!(parse_color("darkred"), Some(Color::Rgb(139, 0, 0)));
        assert_eq!(parse_color("darksalmon"), Some(Color::Rgb(233, 150, 122)));
        assert_eq!(parse_color("darkseagreen"), Some(Color::Rgb(143, 188, 143)));
        assert_eq!(parse_color("darkslateblue"), Some(Color::Rgb(72, 61, 139)));
        assert_eq!(parse_color("darkslategray"), Some(Color::Rgb(47, 79, 79)));
        assert_eq!(parse_color("darkslategrey"), Some(Color::Rgb(47, 79, 79)));
        assert_eq!(parse_color("darkturquoise"), Some(Color::Rgb(0, 206, 209)));
        assert_eq!(parse_color("darkviolet"), Some(Color::Rgb(148, 0, 211)));
        assert_eq!(parse_color("deeppink"), Some(Color::Rgb(255, 20, 147)));
        assert_eq!(parse_color("deepskyblue"), Some(Color::Rgb(0, 191, 255)));
        assert_eq!(parse_color("dimgray"), Some(Color::Rgb(105, 105, 105)));
        assert_eq!(parse_color("dimgrey"), Some(Color::Rgb(105, 105, 105)));
        assert_eq!(parse_color("dodgerblue"), Some(Color::Rgb(30, 144, 255)));
        assert_eq!(parse_color("firebrick"), Some(Color::Rgb(178, 34, 34)));
        assert_eq!(parse_color("floralwhite"), Some(Color::Rgb(255, 250, 240)));
        assert_eq!(parse_color("forestgreen"), Some(Color::Rgb(34, 139, 34)));
        assert_eq!(parse_color("fuchsia"), Some(Color::Rgb(255, 0, 255)));
        assert_eq!(parse_color("gainsboro"), Some(Color::Rgb(220, 220, 220)));
        assert_eq!(parse_color("ghostwhite"), Some(Color::Rgb(248, 248, 255)));
        assert_eq!(parse_color("gold"), Some(Color::Rgb(255, 215, 0)));
        assert_eq!(parse_color("goldenrod"), Some(Color::Rgb(218, 165, 32)));
        assert_eq!(parse_color("greenyellow"), Some(Color::Rgb(173, 255, 47)));
        assert_eq!(parse_color("honeydew"), Some(Color::Rgb(240, 255, 240)));
        assert_eq!(parse_color("hotpink"), Some(Color::Rgb(255, 105, 180)));
        assert_eq!(parse_color("indianred"), Some(Color::Rgb(205, 92, 92)));
        assert_eq!(parse_color("indigo"), Some(Color::Rgb(75, 0, 130)));
        assert_eq!(parse_color("ivory"), Some(Color::Rgb(255, 255, 240)));
        assert_eq!(parse_color("khaki"), Some(Color::Rgb(240, 230, 140)));
        assert_eq!(parse_color("lavender"), Some(Color::Rgb(230, 230, 250)));
        assert_eq!(
            parse_color("lavenderblush"),
            Some(Color::Rgb(255, 240, 245))
        );
        assert_eq!(parse_color("lawngreen"), Some(Color::Rgb(124, 252, 0)));
        assert_eq!(parse_color("lemonchiffon"), Some(Color::Rgb(255, 250, 205)));
        assert_eq!(parse_color("lightcoral"), Some(Color::Rgb(240, 128, 128)));
        assert_eq!(
            parse_color("lightgoldenrodyellow"),
            Some(Color::Rgb(250, 250, 210))
        );
        assert_eq!(parse_color("lightpink"), Some(Color::Rgb(255, 182, 193)));
        assert_eq!(parse_color("lightsalmon"), Some(Color::Rgb(255, 160, 122)));
        assert_eq!(parse_color("lightseagreen"), Some(Color::Rgb(32, 178, 170)));
        assert_eq!(parse_color("lightskyblue"), Some(Color::Rgb(135, 206, 250)));
        assert_eq!(
            parse_color("lightslategray"),
            Some(Color::Rgb(119, 136, 153))
        );
        assert_eq!(
            parse_color("lightslategrey"),
            Some(Color::Rgb(119, 136, 153))
        );
        assert_eq!(
            parse_color("lightsteelblue"),
            Some(Color::Rgb(176, 196, 222))
        );
        assert_eq!(parse_color("lime"), Some(Color::Rgb(0, 255, 0)));
        assert_eq!(parse_color("limegreen"), Some(Color::Rgb(50, 205, 50)));
        assert_eq!(parse_color("linen"), Some(Color::Rgb(250, 240, 230)));
        assert_eq!(parse_color("maroon"), Some(Color::Rgb(128, 0, 0)));
        assert_eq!(
            parse_color("mediumaquamarine"),
            Some(Color::Rgb(102, 205, 170))
        );
        assert_eq!(parse_color("mediumblue"), Some(Color::Rgb(0, 0, 205)));
        assert_eq!(parse_color("mediumorchid"), Some(Color::Rgb(186, 85, 211)));
        assert_eq!(parse_color("mediumpurple"), Some(Color::Rgb(147, 112, 219)));
        assert_eq!(
            parse_color("mediumseagreen"),
            Some(Color::Rgb(60, 179, 113))
        );
        assert_eq!(
            parse_color("mediumslateblue"),
            Some(Color::Rgb(123, 104, 238))
        );
        assert_eq!(
            parse_color("mediumspringgreen"),
            Some(Color::Rgb(0, 250, 154))
        );
        assert_eq!(
            parse_color("mediumturquoise"),
            Some(Color::Rgb(72, 209, 204))
        );
        assert_eq!(
            parse_color("mediumvioletred"),
            Some(Color::Rgb(199, 21, 133))
        );
        assert_eq!(parse_color("midnightblue"), Some(Color::Rgb(25, 25, 112)));
        assert_eq!(parse_color("mintcream"), Some(Color::Rgb(245, 255, 250)));
        assert_eq!(parse_color("mistyrose"), Some(Color::Rgb(255, 228, 225)));
        assert_eq!(parse_color("moccasin"), Some(Color::Rgb(255, 228, 181)));
        assert_eq!(parse_color("navajowhite"), Some(Color::Rgb(255, 222, 173)));
        assert_eq!(parse_color("navy"), Some(Color::Rgb(0, 0, 128)));
        assert_eq!(parse_color("oldlace"), Some(Color::Rgb(253, 245, 230)));
        assert_eq!(parse_color("olive"), Some(Color::Rgb(128, 128, 0)));
        assert_eq!(parse_color("olivedrab"), Some(Color::Rgb(107, 142, 35)));
        assert_eq!(parse_color("orange"), Some(Color::Rgb(255, 165, 0)));
        assert_eq!(parse_color("orangered"), Some(Color::Rgb(255, 69, 0)));
        assert_eq!(parse_color("orchid"), Some(Color::Rgb(218, 112, 214)));
        assert_eq!(
            parse_color("palegoldenrod"),
            Some(Color::Rgb(238, 232, 170))
        );
        assert_eq!(parse_color("palegreen"), Some(Color::Rgb(152, 251, 152)));
        assert_eq!(
            parse_color("paleturquoise"),
            Some(Color::Rgb(175, 238, 238))
        );
        assert_eq!(
            parse_color("palevioletred"),
            Some(Color::Rgb(219, 112, 147))
        );
        assert_eq!(parse_color("papayawhip"), Some(Color::Rgb(255, 239, 213)));
        assert_eq!(parse_color("peachpuff"), Some(Color::Rgb(255, 218, 185)));
        assert_eq!(parse_color("peru"), Some(Color::Rgb(205, 133, 63)));
        assert_eq!(parse_color("pink"), Some(Color::Rgb(255, 192, 203)));
        assert_eq!(parse_color("plum"), Some(Color::Rgb(221, 160, 221)));
        assert_eq!(parse_color("powderblue"), Some(Color::Rgb(176, 224, 230)));
        assert_eq!(parse_color("purple"), Some(Color::Rgb(128, 0, 128)));
        assert_eq!(parse_color("rebeccapurple"), Some(Color::Rgb(102, 51, 153)));
        assert_eq!(parse_color("rosybrown"), Some(Color::Rgb(188, 143, 143)));
        assert_eq!(parse_color("royalblue"), Some(Color::Rgb(65, 105, 225)));
        assert_eq!(parse_color("saddlebrown"), Some(Color::Rgb(139, 69, 19)));
        assert_eq!(parse_color("salmon"), Some(Color::Rgb(250, 128, 114)));
        assert_eq!(parse_color("sandybrown"), Some(Color::Rgb(244, 164, 96)));
        assert_eq!(parse_color("seagreen"), Some(Color::Rgb(46, 139, 87)));
        assert_eq!(parse_color("seashell"), Some(Color::Rgb(255, 245, 238)));
        assert_eq!(parse_color("sienna"), Some(Color::Rgb(160, 82, 45)));
        assert_eq!(parse_color("silver"), Some(Color::Rgb(192, 192, 192)));
        assert_eq!(parse_color("skyblue"), Some(Color::Rgb(135, 206, 235)));
        assert_eq!(parse_color("slateblue"), Some(Color::Rgb(106, 90, 205)));
        assert_eq!(parse_color("slategray"), Some(Color::Rgb(112, 128, 144)));
        assert_eq!(parse_color("slategrey"), Some(Color::Rgb(112, 128, 144)));
        assert_eq!(parse_color("snow"), Some(Color::Rgb(255, 250, 250)));
        assert_eq!(parse_color("springgreen"), Some(Color::Rgb(0, 255, 127)));
        assert_eq!(parse_color("steelblue"), Some(Color::Rgb(70, 130, 180)));
        assert_eq!(parse_color("tan"), Some(Color::Rgb(210, 180, 140)));
        assert_eq!(parse_color("teal"), Some(Color::Rgb(0, 128, 128)));
        assert_eq!(parse_color("thistle"), Some(Color::Rgb(216, 191, 216)));
        assert_eq!(parse_color("tomato"), Some(Color::Rgb(255, 99, 71)));
        assert_eq!(parse_color("turquoise"), Some(Color::Rgb(64, 224, 208)));
        assert_eq!(parse_color("violet"), Some(Color::Rgb(238, 130, 238)));
        assert_eq!(parse_color("wheat"), Some(Color::Rgb(245, 222, 179)));
        assert_eq!(parse_color("whitesmoke"), Some(Color::Rgb(245, 245, 245)));
        assert_eq!(parse_color("yellowgreen"), Some(Color::Rgb(154, 205, 50)));
        // -- hex and rgb
        assert_eq!(
            parse_color("rgb(255, 64, 32)").unwrap(),
            Color::Rgb(255, 64, 32)
        );
        assert!(parse_color("redd").is_none());
    }
}
