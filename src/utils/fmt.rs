//! ## Fmt
//!
//! `fmt` is the module which provides utilities for formatting

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
use crate::fs::UnixPex;

use chrono::prelude::*;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tuirealm::tui::style::Color;

/// ### fmt_pex
///
/// Convert permissions bytes of permissions value into ls notation (e.g. rwx,-wx,--x)
pub fn fmt_pex(pex: UnixPex) -> String {
    format!(
        "{}{}{}",
        match pex.can_read() {
            true => 'r',
            false => '-',
        },
        match pex.can_write() {
            true => 'w',
            false => '-',
        },
        match pex.can_execute() {
            true => 'x',
            false => '-',
        }
    )
}

/// ### instant_to_str
///
/// Format a `Instant` into a time string
pub fn fmt_time(time: SystemTime, fmt: &str) -> String {
    let datetime: DateTime<Local> = time.into();
    format!("{}", datetime.format(fmt))
}

/// ### fmt_millis
///
/// Format duration as {secs}.{millis}
pub fn fmt_millis(duration: Duration) -> String {
    let seconds: u128 = duration.as_millis() / 1000;
    let millis: u128 = duration.as_millis() % 1000;
    format!("{}.{:0width$}", seconds, millis, width = 3)
}

/// ### elide_path
///
/// Elide a path if longer than width
/// In this case, the path is formatted to {ANCESTOR[0]}/…/{PARENT[0]}/{BASENAME}
pub fn fmt_path_elide(p: &Path, width: usize) -> String {
    fmt_path_elide_ex(p, width, 0)
}

/// ### fmt_path_elide_ex
///
/// Elide a path if longer than width
/// In this case, the path is formatted to {ANCESTOR[0]}/…/{PARENT[0]}/{BASENAME}
/// This function allows to specify an extra length to consider to elide path
pub fn fmt_path_elide_ex(p: &Path, width: usize, extra_len: usize) -> String {
    let fmt_path: String = format!("{}", p.display());
    match fmt_path.len() + extra_len > width as usize {
        false => fmt_path,
        true => {
            // Elide
            let ancestors_len: usize = p.ancestors().count();
            let mut ancestors = p.ancestors();
            let mut elided_path: PathBuf = PathBuf::new();
            // If ancestors_len's size is bigger than 2, push count - 2
            if ancestors_len > 2 {
                elided_path.push(ancestors.nth(ancestors_len - 2).unwrap());
            }
            // If ancestors_len is bigger than 3, push '…' and parent too
            if ancestors_len > 3 {
                elided_path.push("…");
                if let Some(parent) = p.ancestors().nth(1) {
                    elided_path.push(parent.file_name().unwrap());
                }
            }
            // Push file_name
            if let Some(name) = p.file_name() {
                elided_path.push(name);
            }
            format!("{}", elided_path.display())
        }
    }
}

/// ### fmt_color
///
/// Format color
pub fn fmt_color(color: &Color) -> String {
    match color {
        Color::Black => "Black".to_string(),
        Color::Blue => "Blue".to_string(),
        Color::Cyan => "Cyan".to_string(),
        Color::DarkGray => "DarkGray".to_string(),
        Color::Gray => "Gray".to_string(),
        Color::Green => "Green".to_string(),
        Color::LightBlue => "LightBlue".to_string(),
        Color::LightCyan => "LightCyan".to_string(),
        Color::LightGreen => "LightGreen".to_string(),
        Color::LightMagenta => "LightMagenta".to_string(),
        Color::LightRed => "LightRed".to_string(),
        Color::LightYellow => "LightYellow".to_string(),
        Color::Magenta => "Magenta".to_string(),
        Color::Red => "Red".to_string(),
        Color::Reset => "Default".to_string(),
        Color::White => "White".to_string(),
        Color::Yellow => "Yellow".to_string(),
        Color::Indexed(_) => "Default".to_string(),
        // -- css colors
        Color::Rgb(240, 248, 255) => "aliceblue".to_string(),
        Color::Rgb(250, 235, 215) => "antiquewhite".to_string(),
        Color::Rgb(0, 255, 255) => "aqua".to_string(),
        Color::Rgb(127, 255, 212) => "aquamarine".to_string(),
        Color::Rgb(240, 255, 255) => "azure".to_string(),
        Color::Rgb(245, 245, 220) => "beige".to_string(),
        Color::Rgb(255, 228, 196) => "bisque".to_string(),
        Color::Rgb(0, 0, 0) => "black".to_string(),
        Color::Rgb(255, 235, 205) => "blanchedalmond".to_string(),
        Color::Rgb(0, 0, 255) => "blue".to_string(),
        Color::Rgb(138, 43, 226) => "blueviolet".to_string(),
        Color::Rgb(165, 42, 42) => "brown".to_string(),
        Color::Rgb(222, 184, 135) => "burlywood".to_string(),
        Color::Rgb(95, 158, 160) => "cadetblue".to_string(),
        Color::Rgb(127, 255, 0) => "chartreuse".to_string(),
        Color::Rgb(210, 105, 30) => "chocolate".to_string(),
        Color::Rgb(255, 127, 80) => "coral".to_string(),
        Color::Rgb(100, 149, 237) => "cornflowerblue".to_string(),
        Color::Rgb(255, 248, 220) => "cornsilk".to_string(),
        Color::Rgb(220, 20, 60) => "crimson".to_string(),
        Color::Rgb(0, 0, 139) => "darkblue".to_string(),
        Color::Rgb(0, 139, 139) => "darkcyan".to_string(),
        Color::Rgb(184, 134, 11) => "darkgoldenrod".to_string(),
        Color::Rgb(169, 169, 169) => "darkgray".to_string(),
        Color::Rgb(0, 100, 0) => "darkgreen".to_string(),
        Color::Rgb(189, 183, 107) => "darkkhaki".to_string(),
        Color::Rgb(139, 0, 139) => "darkmagenta".to_string(),
        Color::Rgb(85, 107, 47) => "darkolivegreen".to_string(),
        Color::Rgb(255, 140, 0) => "darkorange".to_string(),
        Color::Rgb(153, 50, 204) => "darkorchid".to_string(),
        Color::Rgb(139, 0, 0) => "darkred".to_string(),
        Color::Rgb(233, 150, 122) => "darksalmon".to_string(),
        Color::Rgb(143, 188, 143) => "darkseagreen".to_string(),
        Color::Rgb(72, 61, 139) => "darkslateblue".to_string(),
        Color::Rgb(47, 79, 79) => "darkslategray".to_string(),
        Color::Rgb(0, 206, 209) => "darkturquoise".to_string(),
        Color::Rgb(148, 0, 211) => "darkviolet".to_string(),
        Color::Rgb(255, 20, 147) => "deeppink".to_string(),
        Color::Rgb(0, 191, 255) => "deepskyblue".to_string(),
        Color::Rgb(105, 105, 105) => "dimgray".to_string(),
        Color::Rgb(30, 144, 255) => "dodgerblue".to_string(),
        Color::Rgb(178, 34, 34) => "firebrick".to_string(),
        Color::Rgb(255, 250, 240) => "floralwhite".to_string(),
        Color::Rgb(34, 139, 34) => "forestgreen".to_string(),
        Color::Rgb(255, 0, 255) => "fuchsia".to_string(),
        Color::Rgb(220, 220, 220) => "gainsboro".to_string(),
        Color::Rgb(248, 248, 255) => "ghostwhite".to_string(),
        Color::Rgb(255, 215, 0) => "gold".to_string(),
        Color::Rgb(218, 165, 32) => "goldenrod".to_string(),
        Color::Rgb(128, 128, 128) => "gray".to_string(),
        Color::Rgb(0, 128, 0) => "green".to_string(),
        Color::Rgb(173, 255, 47) => "greenyellow".to_string(),
        Color::Rgb(240, 255, 240) => "honeydew".to_string(),
        Color::Rgb(255, 105, 180) => "hotpink".to_string(),
        Color::Rgb(205, 92, 92) => "indianred".to_string(),
        Color::Rgb(75, 0, 130) => "indigo".to_string(),
        Color::Rgb(255, 255, 240) => "ivory".to_string(),
        Color::Rgb(240, 230, 140) => "khaki".to_string(),
        Color::Rgb(230, 230, 250) => "lavender".to_string(),
        Color::Rgb(255, 240, 245) => "lavenderblush".to_string(),
        Color::Rgb(124, 252, 0) => "lawngreen".to_string(),
        Color::Rgb(255, 250, 205) => "lemonchiffon".to_string(),
        Color::Rgb(173, 216, 230) => "lightblue".to_string(),
        Color::Rgb(240, 128, 128) => "lightcoral".to_string(),
        Color::Rgb(224, 255, 255) => "lightcyan".to_string(),
        Color::Rgb(250, 250, 210) => "lightgoldenrodyellow".to_string(),
        Color::Rgb(211, 211, 211) => "lightgray".to_string(),
        Color::Rgb(144, 238, 144) => "lightgreen".to_string(),
        Color::Rgb(255, 182, 193) => "lightpink".to_string(),
        Color::Rgb(255, 160, 122) => "lightsalmon".to_string(),
        Color::Rgb(32, 178, 170) => "lightseagreen".to_string(),
        Color::Rgb(135, 206, 250) => "lightskyblue".to_string(),
        Color::Rgb(119, 136, 153) => "lightslategray".to_string(),
        Color::Rgb(176, 196, 222) => "lightsteelblue".to_string(),
        Color::Rgb(255, 255, 224) => "lightyellow".to_string(),
        Color::Rgb(0, 255, 0) => "lime".to_string(),
        Color::Rgb(50, 205, 50) => "limegreen".to_string(),
        Color::Rgb(250, 240, 230) => "linen".to_string(),
        Color::Rgb(128, 0, 0) => "maroon".to_string(),
        Color::Rgb(102, 205, 170) => "mediumaquamarine".to_string(),
        Color::Rgb(0, 0, 205) => "mediumblue".to_string(),
        Color::Rgb(186, 85, 211) => "mediumorchid".to_string(),
        Color::Rgb(147, 112, 219) => "mediumpurple".to_string(),
        Color::Rgb(60, 179, 113) => "mediumseagreen".to_string(),
        Color::Rgb(123, 104, 238) => "mediumslateblue".to_string(),
        Color::Rgb(0, 250, 154) => "mediumspringgreen".to_string(),
        Color::Rgb(72, 209, 204) => "mediumturquoise".to_string(),
        Color::Rgb(199, 21, 133) => "mediumvioletred".to_string(),
        Color::Rgb(25, 25, 112) => "midnightblue".to_string(),
        Color::Rgb(245, 255, 250) => "mintcream".to_string(),
        Color::Rgb(255, 228, 225) => "mistyrose".to_string(),
        Color::Rgb(255, 228, 181) => "moccasin".to_string(),
        Color::Rgb(255, 222, 173) => "navajowhite".to_string(),
        Color::Rgb(0, 0, 128) => "navy".to_string(),
        Color::Rgb(253, 245, 230) => "oldlace".to_string(),
        Color::Rgb(128, 128, 0) => "olive".to_string(),
        Color::Rgb(107, 142, 35) => "olivedrab".to_string(),
        Color::Rgb(255, 165, 0) => "orange".to_string(),
        Color::Rgb(255, 69, 0) => "orangered".to_string(),
        Color::Rgb(218, 112, 214) => "orchid".to_string(),
        Color::Rgb(238, 232, 170) => "palegoldenrod".to_string(),
        Color::Rgb(152, 251, 152) => "palegreen".to_string(),
        Color::Rgb(175, 238, 238) => "paleturquoise".to_string(),
        Color::Rgb(219, 112, 147) => "palevioletred".to_string(),
        Color::Rgb(255, 239, 213) => "papayawhip".to_string(),
        Color::Rgb(255, 218, 185) => "peachpuff".to_string(),
        Color::Rgb(205, 133, 63) => "peru".to_string(),
        Color::Rgb(255, 192, 203) => "pink".to_string(),
        Color::Rgb(221, 160, 221) => "plum".to_string(),
        Color::Rgb(176, 224, 230) => "powderblue".to_string(),
        Color::Rgb(128, 0, 128) => "purple".to_string(),
        Color::Rgb(102, 51, 153) => "rebeccapurple".to_string(),
        Color::Rgb(255, 0, 0) => "red".to_string(),
        Color::Rgb(188, 143, 143) => "rosybrown".to_string(),
        Color::Rgb(65, 105, 225) => "royalblue".to_string(),
        Color::Rgb(139, 69, 19) => "saddlebrown".to_string(),
        Color::Rgb(250, 128, 114) => "salmon".to_string(),
        Color::Rgb(244, 164, 96) => "sandybrown".to_string(),
        Color::Rgb(46, 139, 87) => "seagreen".to_string(),
        Color::Rgb(255, 245, 238) => "seashell".to_string(),
        Color::Rgb(160, 82, 45) => "sienna".to_string(),
        Color::Rgb(192, 192, 192) => "silver".to_string(),
        Color::Rgb(135, 206, 235) => "skyblue".to_string(),
        Color::Rgb(106, 90, 205) => "slateblue".to_string(),
        Color::Rgb(112, 128, 144) => "slategray".to_string(),
        Color::Rgb(255, 250, 250) => "snow".to_string(),
        Color::Rgb(0, 255, 127) => "springgreen".to_string(),
        Color::Rgb(70, 130, 180) => "steelblue".to_string(),
        Color::Rgb(210, 180, 140) => "tan".to_string(),
        Color::Rgb(0, 128, 128) => "teal".to_string(),
        Color::Rgb(216, 191, 216) => "thistle".to_string(),
        Color::Rgb(255, 99, 71) => "tomato".to_string(),
        Color::Rgb(64, 224, 208) => "turquoise".to_string(),
        Color::Rgb(238, 130, 238) => "violet".to_string(),
        Color::Rgb(245, 222, 179) => "wheat".to_string(),
        Color::Rgb(255, 255, 255) => "white".to_string(),
        Color::Rgb(245, 245, 245) => "whitesmoke".to_string(),
        Color::Rgb(255, 255, 0) => "yellow".to_string(),
        Color::Rgb(154, 205, 50) => "yellowgreen".to_string(),
        // -- others
        Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
    }
}

/// ### shadow_password
///
/// Return a string with the same length of input string, but each character is replaced by '*'
pub fn shadow_password(s: &str) -> String {
    (0..s.len()).map(|_| '*').collect()
}

#[cfg(test)]
mod tests {

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_utils_fmt_pex() {
        assert_eq!(fmt_pex(UnixPex::from(7)), String::from("rwx"));
        assert_eq!(fmt_pex(UnixPex::from(5)), String::from("r-x"));
        assert_eq!(fmt_pex(UnixPex::from(6)), String::from("rw-"));
    }

    #[test]
    fn test_utils_fmt_time() {
        let system_time: SystemTime = SystemTime::from(SystemTime::UNIX_EPOCH);
        assert_eq!(
            fmt_time(system_time, "%Y-%m-%d"),
            String::from("1970-01-01")
        );
    }

    #[test]
    fn test_utils_fmt_millis() {
        assert_eq!(
            fmt_millis(Duration::from_millis(2048)),
            String::from("2.048")
        );
        assert_eq!(
            fmt_millis(Duration::from_millis(8192)),
            String::from("8.192")
        );
        assert_eq!(
            fmt_millis(Duration::from_millis(18192)),
            String::from("18.192")
        );
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_utils_fmt_path_elide() {
        let p: &Path = &Path::new("/develop/pippo");
        // Under max size
        assert_eq!(fmt_path_elide(p, 16), String::from("/develop/pippo"));
        // Above max size, only one ancestor
        assert_eq!(fmt_path_elide(p, 8), String::from("/develop/pippo"));
        let p: &Path = &Path::new("/develop/pippo/foo/bar");
        assert_eq!(fmt_path_elide(p, 16), String::from("/develop/…/foo/bar"));
    }

    #[test]
    fn test_utils_fmt_color() {
        assert_eq!(fmt_color(&Color::Black).as_str(), "Black");
        assert_eq!(fmt_color(&Color::Blue).as_str(), "Blue");
        assert_eq!(fmt_color(&Color::Cyan).as_str(), "Cyan");
        assert_eq!(fmt_color(&Color::DarkGray).as_str(), "DarkGray");
        assert_eq!(fmt_color(&Color::Gray).as_str(), "Gray");
        assert_eq!(fmt_color(&Color::Green).as_str(), "Green");
        assert_eq!(fmt_color(&Color::LightBlue).as_str(), "LightBlue");
        assert_eq!(fmt_color(&Color::LightCyan).as_str(), "LightCyan");
        assert_eq!(fmt_color(&Color::LightGreen).as_str(), "LightGreen");
        assert_eq!(fmt_color(&Color::LightMagenta).as_str(), "LightMagenta");
        assert_eq!(fmt_color(&Color::LightRed).as_str(), "LightRed");
        assert_eq!(fmt_color(&Color::LightYellow).as_str(), "LightYellow");
        assert_eq!(fmt_color(&Color::Magenta).as_str(), "Magenta");
        assert_eq!(fmt_color(&Color::Red).as_str(), "Red");
        assert_eq!(fmt_color(&Color::Reset).as_str(), "Default");
        assert_eq!(fmt_color(&Color::White).as_str(), "White");
        assert_eq!(fmt_color(&Color::Yellow).as_str(), "Yellow");
        assert_eq!(fmt_color(&Color::Indexed(16)).as_str(), "Default");
        assert_eq!(fmt_color(&Color::Rgb(204, 170, 22)).as_str(), "#ccaa16");
        assert_eq!(fmt_color(&Color::Rgb(204, 170, 0)).as_str(), "#ccaa00");
        // css colors
        assert_eq!(fmt_color(&Color::Rgb(240, 248, 255)).as_str(), "aliceblue");
        assert_eq!(
            fmt_color(&Color::Rgb(250, 235, 215)).as_str(),
            "antiquewhite"
        );
        assert_eq!(fmt_color(&Color::Rgb(0, 255, 255)).as_str(), "aqua");
        assert_eq!(fmt_color(&Color::Rgb(127, 255, 212)).as_str(), "aquamarine");
        assert_eq!(fmt_color(&Color::Rgb(240, 255, 255)).as_str(), "azure");
        assert_eq!(fmt_color(&Color::Rgb(245, 245, 220)).as_str(), "beige");
        assert_eq!(fmt_color(&Color::Rgb(255, 228, 196)).as_str(), "bisque");
        assert_eq!(fmt_color(&Color::Rgb(0, 0, 0)).as_str(), "black");
        assert_eq!(
            fmt_color(&Color::Rgb(255, 235, 205)).as_str(),
            "blanchedalmond"
        );
        assert_eq!(fmt_color(&Color::Rgb(0, 0, 255)).as_str(), "blue");
        assert_eq!(fmt_color(&Color::Rgb(138, 43, 226)).as_str(), "blueviolet");
        assert_eq!(fmt_color(&Color::Rgb(165, 42, 42)).as_str(), "brown");
        assert_eq!(fmt_color(&Color::Rgb(222, 184, 135)).as_str(), "burlywood");
        assert_eq!(fmt_color(&Color::Rgb(95, 158, 160)).as_str(), "cadetblue");
        assert_eq!(fmt_color(&Color::Rgb(127, 255, 0)).as_str(), "chartreuse");
        assert_eq!(fmt_color(&Color::Rgb(210, 105, 30)).as_str(), "chocolate");
        assert_eq!(fmt_color(&Color::Rgb(255, 127, 80)).as_str(), "coral");
        assert_eq!(
            fmt_color(&Color::Rgb(100, 149, 237)).as_str(),
            "cornflowerblue"
        );
        assert_eq!(fmt_color(&Color::Rgb(255, 248, 220)).as_str(), "cornsilk");
        assert_eq!(fmt_color(&Color::Rgb(220, 20, 60)).as_str(), "crimson");
        assert_eq!(fmt_color(&Color::Rgb(0, 0, 139)).as_str(), "darkblue");
        assert_eq!(fmt_color(&Color::Rgb(0, 139, 139)).as_str(), "darkcyan");
        assert_eq!(
            fmt_color(&Color::Rgb(184, 134, 11)).as_str(),
            "darkgoldenrod"
        );
        assert_eq!(fmt_color(&Color::Rgb(169, 169, 169)).as_str(), "darkgray");
        assert_eq!(fmt_color(&Color::Rgb(0, 100, 0)).as_str(), "darkgreen");
        assert_eq!(fmt_color(&Color::Rgb(189, 183, 107)).as_str(), "darkkhaki");
        assert_eq!(fmt_color(&Color::Rgb(139, 0, 139)).as_str(), "darkmagenta");
        assert_eq!(
            fmt_color(&Color::Rgb(85, 107, 47)).as_str(),
            "darkolivegreen"
        );
        assert_eq!(fmt_color(&Color::Rgb(255, 140, 0)).as_str(), "darkorange");
        assert_eq!(fmt_color(&Color::Rgb(153, 50, 204)).as_str(), "darkorchid");
        assert_eq!(fmt_color(&Color::Rgb(139, 0, 0)).as_str(), "darkred");
        assert_eq!(fmt_color(&Color::Rgb(233, 150, 122)).as_str(), "darksalmon");
        assert_eq!(
            fmt_color(&Color::Rgb(143, 188, 143)).as_str(),
            "darkseagreen"
        );
        assert_eq!(
            fmt_color(&Color::Rgb(72, 61, 139)).as_str(),
            "darkslateblue"
        );
        assert_eq!(fmt_color(&Color::Rgb(47, 79, 79)).as_str(), "darkslategray");
        assert_eq!(
            fmt_color(&Color::Rgb(0, 206, 209)).as_str(),
            "darkturquoise"
        );
        assert_eq!(fmt_color(&Color::Rgb(148, 0, 211)).as_str(), "darkviolet");
        assert_eq!(fmt_color(&Color::Rgb(255, 20, 147)).as_str(), "deeppink");
        assert_eq!(fmt_color(&Color::Rgb(0, 191, 255)).as_str(), "deepskyblue");
        assert_eq!(fmt_color(&Color::Rgb(105, 105, 105)).as_str(), "dimgray");
        assert_eq!(fmt_color(&Color::Rgb(30, 144, 255)).as_str(), "dodgerblue");
        assert_eq!(fmt_color(&Color::Rgb(178, 34, 34)).as_str(), "firebrick");
        assert_eq!(
            fmt_color(&Color::Rgb(255, 250, 240)).as_str(),
            "floralwhite"
        );
        assert_eq!(fmt_color(&Color::Rgb(34, 139, 34)).as_str(), "forestgreen");
        assert_eq!(fmt_color(&Color::Rgb(255, 0, 255)).as_str(), "fuchsia");
        assert_eq!(fmt_color(&Color::Rgb(220, 220, 220)).as_str(), "gainsboro");
        assert_eq!(fmt_color(&Color::Rgb(248, 248, 255)).as_str(), "ghostwhite");
        assert_eq!(fmt_color(&Color::Rgb(255, 215, 0)).as_str(), "gold");
        assert_eq!(fmt_color(&Color::Rgb(218, 165, 32)).as_str(), "goldenrod");
        assert_eq!(fmt_color(&Color::Rgb(128, 128, 128)).as_str(), "gray");
        assert_eq!(fmt_color(&Color::Rgb(0, 128, 0)).as_str(), "green");
        assert_eq!(fmt_color(&Color::Rgb(173, 255, 47)).as_str(), "greenyellow");
        assert_eq!(fmt_color(&Color::Rgb(240, 255, 240)).as_str(), "honeydew");
        assert_eq!(fmt_color(&Color::Rgb(255, 105, 180)).as_str(), "hotpink");
        assert_eq!(fmt_color(&Color::Rgb(205, 92, 92)).as_str(), "indianred");
        assert_eq!(fmt_color(&Color::Rgb(75, 0, 130)).as_str(), "indigo");
        assert_eq!(fmt_color(&Color::Rgb(255, 255, 240)).as_str(), "ivory");
        assert_eq!(fmt_color(&Color::Rgb(240, 230, 140)).as_str(), "khaki");
        assert_eq!(fmt_color(&Color::Rgb(230, 230, 250)).as_str(), "lavender");
        assert_eq!(
            fmt_color(&Color::Rgb(255, 240, 245)).as_str(),
            "lavenderblush"
        );
        assert_eq!(fmt_color(&Color::Rgb(124, 252, 0)).as_str(), "lawngreen");
        assert_eq!(
            fmt_color(&Color::Rgb(255, 250, 205)).as_str(),
            "lemonchiffon"
        );
        assert_eq!(fmt_color(&Color::Rgb(173, 216, 230)).as_str(), "lightblue");
        assert_eq!(fmt_color(&Color::Rgb(240, 128, 128)).as_str(), "lightcoral");
        assert_eq!(fmt_color(&Color::Rgb(224, 255, 255)).as_str(), "lightcyan");
        assert_eq!(
            fmt_color(&Color::Rgb(250, 250, 210)).as_str(),
            "lightgoldenrodyellow"
        );
        assert_eq!(fmt_color(&Color::Rgb(211, 211, 211)).as_str(), "lightgray");
        assert_eq!(fmt_color(&Color::Rgb(144, 238, 144)).as_str(), "lightgreen");
        assert_eq!(fmt_color(&Color::Rgb(255, 182, 193)).as_str(), "lightpink");
        assert_eq!(
            fmt_color(&Color::Rgb(255, 160, 122)).as_str(),
            "lightsalmon"
        );
        assert_eq!(
            fmt_color(&Color::Rgb(32, 178, 170)).as_str(),
            "lightseagreen"
        );
        assert_eq!(
            fmt_color(&Color::Rgb(135, 206, 250)).as_str(),
            "lightskyblue"
        );
        assert_eq!(
            fmt_color(&Color::Rgb(119, 136, 153)).as_str(),
            "lightslategray"
        );
        assert_eq!(
            fmt_color(&Color::Rgb(176, 196, 222)).as_str(),
            "lightsteelblue"
        );
        assert_eq!(
            fmt_color(&Color::Rgb(255, 255, 224)).as_str(),
            "lightyellow"
        );
        assert_eq!(fmt_color(&Color::Rgb(0, 255, 0)).as_str(), "lime");
        assert_eq!(fmt_color(&Color::Rgb(50, 205, 50)).as_str(), "limegreen");
        assert_eq!(fmt_color(&Color::Rgb(250, 240, 230)).as_str(), "linen");
        assert_eq!(fmt_color(&Color::Rgb(128, 0, 0)).as_str(), "maroon");
        assert_eq!(
            fmt_color(&Color::Rgb(102, 205, 170)).as_str(),
            "mediumaquamarine"
        );
        assert_eq!(fmt_color(&Color::Rgb(0, 0, 205)).as_str(), "mediumblue");
        assert_eq!(
            fmt_color(&Color::Rgb(186, 85, 211)).as_str(),
            "mediumorchid"
        );
        assert_eq!(
            fmt_color(&Color::Rgb(147, 112, 219)).as_str(),
            "mediumpurple"
        );
        assert_eq!(
            fmt_color(&Color::Rgb(60, 179, 113)).as_str(),
            "mediumseagreen"
        );
        assert_eq!(
            fmt_color(&Color::Rgb(123, 104, 238)).as_str(),
            "mediumslateblue"
        );
        assert_eq!(
            fmt_color(&Color::Rgb(0, 250, 154)).as_str(),
            "mediumspringgreen"
        );
        assert_eq!(
            fmt_color(&Color::Rgb(72, 209, 204)).as_str(),
            "mediumturquoise"
        );
        assert_eq!(
            fmt_color(&Color::Rgb(199, 21, 133)).as_str(),
            "mediumvioletred"
        );
        assert_eq!(fmt_color(&Color::Rgb(25, 25, 112)).as_str(), "midnightblue");
        assert_eq!(fmt_color(&Color::Rgb(245, 255, 250)).as_str(), "mintcream");
        assert_eq!(fmt_color(&Color::Rgb(255, 228, 225)).as_str(), "mistyrose");
        assert_eq!(fmt_color(&Color::Rgb(255, 228, 181)).as_str(), "moccasin");
        assert_eq!(
            fmt_color(&Color::Rgb(255, 222, 173)).as_str(),
            "navajowhite"
        );
        assert_eq!(fmt_color(&Color::Rgb(0, 0, 128)).as_str(), "navy");
        assert_eq!(fmt_color(&Color::Rgb(253, 245, 230)).as_str(), "oldlace");
        assert_eq!(fmt_color(&Color::Rgb(128, 128, 0)).as_str(), "olive");
        assert_eq!(fmt_color(&Color::Rgb(107, 142, 35)).as_str(), "olivedrab");
        assert_eq!(fmt_color(&Color::Rgb(255, 165, 0)).as_str(), "orange");
        assert_eq!(fmt_color(&Color::Rgb(255, 69, 0)).as_str(), "orangered");
        assert_eq!(fmt_color(&Color::Rgb(218, 112, 214)).as_str(), "orchid");
        assert_eq!(
            fmt_color(&Color::Rgb(238, 232, 170)).as_str(),
            "palegoldenrod"
        );
        assert_eq!(fmt_color(&Color::Rgb(152, 251, 152)).as_str(), "palegreen");
        assert_eq!(
            fmt_color(&Color::Rgb(175, 238, 238)).as_str(),
            "paleturquoise"
        );
        assert_eq!(
            fmt_color(&Color::Rgb(219, 112, 147)).as_str(),
            "palevioletred"
        );
        assert_eq!(fmt_color(&Color::Rgb(255, 239, 213)).as_str(), "papayawhip");
        assert_eq!(fmt_color(&Color::Rgb(255, 218, 185)).as_str(), "peachpuff");
        assert_eq!(fmt_color(&Color::Rgb(205, 133, 63)).as_str(), "peru");
        assert_eq!(fmt_color(&Color::Rgb(255, 192, 203)).as_str(), "pink");
        assert_eq!(fmt_color(&Color::Rgb(221, 160, 221)).as_str(), "plum");
        assert_eq!(fmt_color(&Color::Rgb(176, 224, 230)).as_str(), "powderblue");
        assert_eq!(fmt_color(&Color::Rgb(128, 0, 128)).as_str(), "purple");
        assert_eq!(
            fmt_color(&Color::Rgb(102, 51, 153)).as_str(),
            "rebeccapurple"
        );
        assert_eq!(fmt_color(&Color::Rgb(255, 0, 0)).as_str(), "red");
        assert_eq!(fmt_color(&Color::Rgb(188, 143, 143)).as_str(), "rosybrown");
        assert_eq!(fmt_color(&Color::Rgb(65, 105, 225)).as_str(), "royalblue");
        assert_eq!(fmt_color(&Color::Rgb(139, 69, 19)).as_str(), "saddlebrown");
        assert_eq!(fmt_color(&Color::Rgb(250, 128, 114)).as_str(), "salmon");
        assert_eq!(fmt_color(&Color::Rgb(244, 164, 96)).as_str(), "sandybrown");
        assert_eq!(fmt_color(&Color::Rgb(46, 139, 87)).as_str(), "seagreen");
        assert_eq!(fmt_color(&Color::Rgb(255, 245, 238)).as_str(), "seashell");
        assert_eq!(fmt_color(&Color::Rgb(160, 82, 45)).as_str(), "sienna");
        assert_eq!(fmt_color(&Color::Rgb(192, 192, 192)).as_str(), "silver");
        assert_eq!(fmt_color(&Color::Rgb(135, 206, 235)).as_str(), "skyblue");
        assert_eq!(fmt_color(&Color::Rgb(106, 90, 205)).as_str(), "slateblue");
        assert_eq!(fmt_color(&Color::Rgb(112, 128, 144)).as_str(), "slategray");
        assert_eq!(fmt_color(&Color::Rgb(255, 250, 250)).as_str(), "snow");
        assert_eq!(fmt_color(&Color::Rgb(0, 255, 127)).as_str(), "springgreen");
        assert_eq!(fmt_color(&Color::Rgb(70, 130, 180)).as_str(), "steelblue");
        assert_eq!(fmt_color(&Color::Rgb(210, 180, 140)).as_str(), "tan");
        assert_eq!(fmt_color(&Color::Rgb(0, 128, 128)).as_str(), "teal");
        assert_eq!(fmt_color(&Color::Rgb(216, 191, 216)).as_str(), "thistle");
        assert_eq!(fmt_color(&Color::Rgb(255, 99, 71)).as_str(), "tomato");
        assert_eq!(fmt_color(&Color::Rgb(64, 224, 208)).as_str(), "turquoise");
        assert_eq!(fmt_color(&Color::Rgb(238, 130, 238)).as_str(), "violet");
        assert_eq!(fmt_color(&Color::Rgb(245, 222, 179)).as_str(), "wheat");
        assert_eq!(fmt_color(&Color::Rgb(255, 255, 255)).as_str(), "white");
        assert_eq!(fmt_color(&Color::Rgb(245, 245, 245)).as_str(), "whitesmoke");
        assert_eq!(fmt_color(&Color::Rgb(255, 255, 0)).as_str(), "yellow");
        assert_eq!(fmt_color(&Color::Rgb(154, 205, 50)).as_str(), "yellowgreen");
    }

    #[test]
    fn test_utils_fmt_shadow_password() {
        assert_eq!(shadow_password("foobar"), String::from("******"));
    }
}
