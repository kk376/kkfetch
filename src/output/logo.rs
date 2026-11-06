//! ASCII Art Logos and ANSI Color Layout Engine.
//!
//! Distribution ASCII art boundary outlines are based on classic designs from
//! Neofetch (MIT License, Copyright (c) 2016-2022 Dylan Araps), enhanced in
//! KKFetch with high-contrast white structural framing (`{w}`) and
//! distribution brand signature color emblems (`{c}`).

use crate::context::ColorLevel;
use crate::output::color::{resolve_color, RESET};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogoVariant {
    #[default]
    Auto,
    Full,
    Compact,
}

pub const WHITE_COLOR: &str = "\x1b[38;5;231m";

/// Standard 6-color palette tokens for vintage multi-color distribution art (e.g., 1977 Apple rainbow).
pub const RAINBOW_PALETTE: [(&str, &str); 6] = [
    ("{1}", "\x1b[38;5;76m"),  // Vintage Green
    ("{2}", "\x1b[38;5;220m"), // Vintage Yellow
    ("{3}", "\x1b[38;5;208m"), // Vintage Orange
    ("{4}", "\x1b[38;5;196m"), // Vintage Red
    ("{5}", "\x1b[38;5;129m"), // Vintage Purple
    ("{6}", "\x1b[38;5;39m"),  // Vintage Blue
];

#[derive(Debug, Clone)]
pub struct Logo {
    pub name: &'static str,
    pub raw_lines: &'static [&'static str],
    pub compact_lines: Option<&'static [&'static str]>,
    pub distro_color: &'static str,
    pub outer_color: &'static str,
}

impl Logo {
    /// Returns the compact ASCII lines if defined, or falls back to generic compact Tux penguin or full art.
    pub fn compact_lines_or_fallback(&self) -> &'static [&'static str] {
        if let Some(lines) = self.compact_lines {
            lines
        } else {
            ALL_LOGOS
                .iter()
                .find(|l| l.name == "generic")
                .and_then(|g| g.compact_lines)
                .unwrap_or(self.raw_lines)
        }
    }

    /// Returns the logo lines formatted with ANSI colors if enabled (backwards-compatible).
    /// The outer layer is styled with `outer_color` (white), and the
    /// highlighted inner emblem is styled with `distro_color`.
    /// Numbered palette tokens `{1}`..`{6}` provide multi-color stripe
    /// support for vintage logos such as the 1977 Apple rainbow.
    pub fn render_lines(&self, enable_color: bool) -> Vec<String> {
        let level = if enable_color {
            ColorLevel::Color256
        } else {
            ColorLevel::None
        };
        self.render_variant_lines(LogoVariant::Full, level)
    }

    /// Returns rendered lines for a specific logo variant and color level.
    pub fn render_variant_lines(
        &self,
        variant: LogoVariant,
        color_level: ColorLevel,
    ) -> Vec<String> {
        let lines = match variant {
            LogoVariant::Compact => self.compact_lines_or_fallback(),
            _ => self.raw_lines,
        };

        let enable_color = color_level != ColorLevel::None;
        let d_color = resolve_color(self.distro_color, color_level);
        let o_color = resolve_color(self.outer_color, color_level);

        lines
            .iter()
            .map(|line| {
                if enable_color {
                    let mut rendered = line
                        .replace("{c}", d_color)
                        .replace("{w}", o_color)
                        .replace("{0}", RESET);
                    for &(token, color) in &RAINBOW_PALETTE {
                        rendered = rendered.replace(token, resolve_color(color, color_level));
                    }

                    let has_color = line.contains("{c}")
                        || line.contains("{w}")
                        || RAINBOW_PALETTE
                            .iter()
                            .any(|(token, _)| line.contains(token));

                    if !has_color && !line.is_empty() {
                        rendered = format!("{}{}{}", d_color, line, RESET);
                    } else if !line.is_empty() {
                        rendered.push_str(RESET);
                    }
                    rendered
                } else {
                    let mut stripped = line
                        .replace("{c}", "")
                        .replace("{w}", "")
                        .replace("{0}", "");
                    for &(token, _) in &RAINBOW_PALETTE {
                        stripped = stripped.replace(token, "");
                    }
                    stripped
                }
            })
            .collect()
    }
}

pub const ALL_LOGOS: &[Logo] = &[
    Logo {
        name: "ferris",
        raw_lines: &[
            "        {w}/^^^\\     /^^^\\",
            "       {w}(  O  )   (  O  )",
            "      {c}.-'---'-----'---'-.",
            "     {c}/   _~^~^~^~^~_     \\",
            "    {c}|   /  {w}o     o{c}  \\     |",
            "    {c}|   |     {w}-{c}     |     |",
            "    {c}|   \\  {w}'---'{c}  /     |",
            "     {c}\\   '-------'     /",
            "      {c}'-._'-------'_.-'",
            "          {w}/       \\",
        ],
        compact_lines: Some(&[
            "  {w}_~^~^~_",
            " {w}\\) {c}/  o o  \\ {w}\\(",
            "  {c}'_   -   _'",
            "  {c}/ '-----' \\",
        ]),
        distro_color: "\x1b[38;5;208m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "debian",
        raw_lines: &[
            "{c}       _,met$$$$$gg.",
            "{c}    ,g$$$$$$$$$$$$$$$P.",
            "{c}  ,g$$P\"        \"\"\"Y$$.\".",
            "{c} ,$$P'              `$$$.",
            "{c}',$$P       {w},ggs.{c}     `$$b:",
            "{c}`d$$'     {w},$P\"'   .{c}    $$$",
            "{c} $$P      {w}d$'     ,{c}    $$P",
            "{c} $$:      {w}$$.   -{c}    ,d$$'",
            "{c} $$;      {w}Y$b._   _,d$P'{c}",
            "{c} Y$$.    {w}`.`\"Y$$$$P\"'{c}",
            "{c} `$$b      {w}\"-.__{c}",
            "{c}  `Y$$",
            "{c}   `Y$$.",
            "{c}     `$$b.",
            "{c}       `Y$$b.",
            "{c}          `\"Y$b._",
            "{c}              `\"\"\"",
        ],
        compact_lines: Some(&[
            "{c}  _____",
            "{c} /  __ \\",
            "{c}|  /    |",
            "{c}|  \\___-",
            "{c} -_",
            "{c}   --_",
        ]),
        distro_color: "\x1b[38;5;196m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "ubuntu",
        raw_lines: &[
            "{w}            .-/+oossssoo+\\-.",
            "{w}        ´:+ssssssssssssssssss+:`",
            "{w}      -+ssssssssssssssssssyyssss+-",
            "{w}    .ossssssssssssssssss{c}dMMMNy{w}sssso.",
            "{w}   /sssssssssss{c}hdmmNNmmyNMMMMh{w}ssssss\\",
            "{w}  +sssssssss{c}hm{w}yd{c}MMMMMMMNddddy{w}ssssssss+",
            "{w} /ssssssss{c}hNMMM{w}yh{c}hyyyyhmNMMMNh{w}ssssssss\\",
            "{w}.ssssssss{c}dMMMNh{w}ssssssssss{c}hNMMMd{w}ssssssss.",
            "{w}+ssss{c}hhhyNMMNy{w}ssssssssssss{c}yNMMMy{w}sssssss+",
            "{w}oss{c}yNMMMNyMMh{w}ssssssssssssss{c}hmmmh{w}ssssssso",
            "{w}oss{c}yNMMMNyMMh{w}ssssssssssssss{c}hmmmh{w}ssssssso",
            "{w}+ssss{c}hhhyNMMNy{w}ssssssssssss{c}yNMMMy{w}sssssss+",
            "{w}.ssssssss{c}dMMMNh{w}ssssssssss{c}hNMMMd{w}ssssssss.",
            "{w} \\ssssssss{c}hNMMM{w}yh{c}hyyyyhdNMMMNh{w}ssssssss/",
            "{w}  +sssssssss{c}dm{w}yd{c}MMMMMMMMddddy{w}ssssssss+",
            "{w}   \\sssssssssss{c}hdmNNNNmyNMMMMh{w}ssssss/",
            "{w}    .ossssssssssssssssss{c}dMMMNy{w}sssso.",
            "{w}      -+sssssssssssssssss{c}yyy{w}ssss+-",
            "{w}        `:+ssssssssssssssssss+:`",
            "{w}            .-\\+oossssoo+/-.",
        ],
        compact_lines: Some(&[
            "{c}         _",
            "{w}     ---{c}(_)",
            "{w}  _/{c}  ---  {w}\\",
            "{c} (_){w} |   |",
            "{w}   \\ {c} --- {w}_/",
            "{w}     ---{c}(_)",
        ]),
        distro_color: "\x1b[38;5;208m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "linuxmint",
        raw_lines: &[
            "{w}             ...-:::::-...",
            "{w}          .-MMMMMMMMMMMMMMM-.",
            "{w}      .-MMMM{c}`..-:::::::-..`{w}MMMM-.",
            "{w}    .:MMMM{c}.:MMMMMMMMMMMMMMM:.{w}MMMM:.",
            "{w}   -MMM{c}-M---MMMMMMMMMMMMMMMMMMM.{w}MMM-",
            "{w} `:MMM{c}:MM`  :MMMM:....::-...-MMMM:{w}MMM:`",
            "{w} :MMM{c}:MMM`  :MM:`  ``    ``  `:MMM:{w}MMM:",
            "{w}.MMM{c}.MMMM`  :MM.  -MM.  .MM-  `MMMM.{w}MMM.",
            "{w}:MMM{c}:MMMM`  :MM.  -MM-  .MM:  `MMMM-{w}MMM:",
            "{w}:MMM{c}:MMMM`  :MM.  -MM-  .MM:  `MMMM:{w}MMM:",
            "{w}:MMM{c}:MMMM`  :MM.  -MM-  .MM:  `MMMM-{w}MMM:",
            "{w}.MMM{c}.MMMM`  :MM:--:MM:--:MM:  `MMMM.{w}MMM.",
            "{w} :MMM{c}:MMM-  `-MMMMMMMMMMMM-`  -MMM-{w}MMM:",
            "{w}  :MMM{c}:MMM:`                `:MMM:{w}MMM:",
            "{w}   .MMM{c}.MMMM:--------------:MMMM.{w}MMM.",
            "{w}     '-MMMM{c}.-MMMMMMMMMMMMMMM-.{w}MMMM-'",
            "{w}       '.-MMMM{c}``--:::::--``{w}MMMM-.'",
            "{w}            '-MMMMMMMMMMMMM-'",
            "{w}               ``-:::::-``",
        ],
        compact_lines: Some(&[
            "{w} _________",
            "{w}|_        \\",
            "{w}  | | {c}_____{w} |",
            "{w}  | | {c}| | |{w} |",
            "{w}  | | {c}|_|_|{w} |",
            "{w}  | \\_____/|",
            "{w}  \\________/",
        ]),
        distro_color: "\x1b[38;5;46m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "fedora",
        raw_lines: &[
            "{w}             .',;::::;,'.",
            "{w}         .';:cccccccccccc:;,.",
            "{w}      .;cccccccccccccccccccccc;.",
            "{w}    .:cccccccccccccccccccccccccc:.",
            "{w}  .;ccccccccccccc;{c}.:dddl:.{w};ccccccc;.",
            "{w} .:ccccccccccccc;{c}OWMKOOXMWd{w};ccccccc:.",
            "{w}.:ccccccccccccc;{c}KMMc{w};cc;{c}xMMc{w};ccccccc:.",
            "{w},cccccccccccccc;{c}MMM.{w};cc;{c};WW:{w};cccccccc,",
            "{w}:cccccccccccccc;{c}MMM.{w};cccccccccccccccc:",
            "{w}:ccccccc;{c}oxOOOo{w};{c}MMM0OOk.{w};cccccccccccc:",
            "{w}cccccc;{c}0MMKxdd:{w};{c}MMMkddc.{w};cccccccccccc;",
            "{w}ccccc;{c}XM0'{w};cccc;{c}MMM.{w};cccccccccccccccc'",
            "{w}ccccc;{c}MMo{w};ccccc;{c}MMW.{w};ccccccccccccccc;",
            "{w}ccccc;{c}0MNc.{w}ccc{c}.xMMd{w};ccccccccccccccc;",
            "{w}cccccc;{c}dNMWXXXWM0:{w};cccccccccccccc:,",
            "{w}cccccccc;{c}.:odl:.{w};cccccccccccccc:,.",
            "{w}:cccccccccccccccccccccccccccc:'.",
            "{w}.:cccccccccccccccccccccc:;,..",
            "{w}  '::cccccccccccccc::;,.",
        ],
        compact_lines: Some(&[
            "{c}        ,'''''.",
            "{c}       |   {w},.  {c}|",
            "{c}       |  {w}|  '_'{c}",
            "{w}  ,....{c}|  {w}|..",
            "{w}.'  ,_;{c}|   {w}..'",
            "{w}|  |   {c}|  {w}|",
            "{w}|  ',_,'  {w}|",
            "{w} '.     ,'",
            "{w}   '''''",
        ]),
        distro_color: "\x1b[38;5;33m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "arch",
        raw_lines: &[
            "{w}                   -`",
            "{w}                  .o+`",
            "{w}                 `ooo/",
            "{w}                `+oooo:",
            "{w}               `+oooooo:",
            "{w}               -+oooooo+:",
            "{w}             `/:-:++oooo+:",
            "{w}            `/++++/+++++++:",
            "{w}           `/++++++++++++++:",
            "{w}          `/+++o{c}oooooooo{w}oooo/`",
            "{w}         ./{c}ooosssso++osssssso{w}+`",
            "{w}        .{c}oossssso-````/ossssss{w}+`",
            "{w}       -{c}osssssso.      :ssssssso{w}.",
            "{w}      :{c}osssssss/        {w}osssso+++.",
            "{w}     /{c}ossssssss/        {w}+ssssooo/-",
            "{w}   `/{c}ossssso+/:-        {w}-:/+osssso+-",
            "{w}  `+sso+:-`                 `.-/+oso:",
            "{w} `++:.                           `-/+/",
            "{w} .`                                 `/",
        ],
        compact_lines: Some(&[
            "{c}      /\\",
            "{c}     /  \\",
            "{c}    /\\   \\",
            "{c}   /      \\",
            "{c}  /   ,,   \\",
            "{c} /   |  |  -\\",
            "{c}/_-''    ''-_/",
        ]),
        distro_color: "\x1b[38;5;39m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "rhel",
        raw_lines: &[
            "{w}             `.-..........`",
            "{w}            `////////::.`-/.",
            "{w}            -: ....-////////.",
            "{w}            //:-::///////////`",
            "{w}     `--::: `-://////////////:",
            "{w}     //////-    ``.-:///////// .`",
            "{w}     `://////:-.`    :///////::///:`",
            "{w}       .-/////////:---/////////////:",
            "{w}          .-://////////////////////.",
            "{c}         yMN+`.-{w}::///////////////-`",
            "{c}      .-`:NMMNMs`  {w}`..-------..`",
            "{c}       MN+/mMMMMMhoooyysshsss",
            "{c}MMM    MMMMMMMMMMMMMMyyddMMM+",
            "{c} MMMM   MMMMMMMMMMMMMNdyNMMh`     {w}hyhMMM",
            "{c}  MMMMMMMMMMMMMMMMyoNNNMMM+.   {w}MMMMMMMM",
            "{c}   MMNMMMNNMMMMMNM+ mhsMNyyyyMNMMMMsMM",
        ],
        compact_lines: Some(&[
            "{c}      .M.:MMM",
            "{c}     MMMMMMMMMM.",
            "{c}    ,MMMMMMMMMMM",
            "{c} .MM MMMMMMMMMMM",
            "{c}MMMM   MMMMMMMMM",
            "{c}MMMMMM           MM",
            "{c} MMMMMMMMM     ,MMMM",
            "{c}   MMMMMMMMMMMMMMMM:",
            "{c}      `MMMMMMMMMMMM",
        ]),
        distro_color: "\x1b[38;5;196m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "rocky",
        raw_lines: &[
            "{w}    `-/+++++++++/-.`",
            "{w} `-+++++++++++++++++-`",
            "{w}.+++++++++++++++++++++.",
            "{w}-+++++++++++++++++++++++.",
            "{c}+++++++++++++++{w}/-/{c}+++++++",
            "{c}+++++++++++++/.   {w}./+++++",
            "{c}+++++++++++:.       {w}./+++",
            "{c}+++++++++:`   `:/:`   {w}.:/",
            "{c}-++++++:`   .:+++++:`",
            "{c} .+++-`   ./+++++++++:`",
            "{c}  `-`   ./+++++++++++-",
            "{c}       -+++++++++:-.`",
        ],
        compact_lines: Some(&[
            "{c}    `-/+++++++++/-.`",
            "{c} `-+++++++++++++++++-`",
            "{c}.+++++++++++++++++++++.",
            "{c}-+++++++++++++++++++++++.",
            "{c}+++++++++++++++/-/+++++++",
            "{c}+++++++++++++/.   ./+++++",
            "{c}+++++++++++:.       ./+++",
            "{c}+++++++++:`   `:/:`   .:/",
            "{c}-++++++:`   .:+++++:`",
            "{c} .+++-`   ./+++++++++:`",
            "{c}  `-`   ./+++++++++++-",
            "{c}       -+++++++++:-.`",
        ]),
        distro_color: "\x1b[38;5;35m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "almalinux",
        raw_lines: &[
            "{w}         'c:.",
            "{w}        lkkkx, ..       {c}..   ,cc,",
            "{w}        okkkk:ckkx'  {c}.lxkkx.okkkkd",
            "{w}        .:llcokkx'  {c}:kkkxkko:xkkd,",
            "{w}      .xkkkkdood:  {c};kx,  .lkxlll;",
            "{w}       xkkx.       {c}xk'     xkkkkk:",
            "{w}       'xkx.       {c}xd      .....,.",
            "{c}      .. {w}:xkl'     {c}:c      ..''..",
            "{c}    .dkx'  {w}.:ldl:'. {c}'  ':lollldkkxo;",
            "{c}  .''lkkko'                     ckkkx.",
            "{c}'xkkkd:kkd.       ..  {w};'        {c}:kkxo.",
            "{c},xkkkd;kk'      ,d;    {w}ld.   {c}':dkd::cc,",
            "{c} .,,.;xkko'.';lxo.      {w}dx,  {c}:kkk'xkkkkc",
            "{c}     'dkkkkkxo:.        {w};kx  {c}.kkk:;xkkd.",
            "{c}       .....   {w}.;dk:.   lkk.  {c}:;,",
            "{w}             :kkkkkkkdoxkkx",
            "{w}              ,c,,;;;:xkkd.",
            "{w}                ;kkkkl...",
            "{w}                ;kkkkl",
            "{w}                 ,od;",
        ],
        compact_lines: Some(&[
            "{4}   'c:.    {1}..  ,cc,",
            "{4}  lkkk; {1}.lxkk.okkd",
            "{4} .xkkkd {1};kx.  .lkx",
            "{6}.'lkkko'     {2}ckkkx.",
            "{6}'xkkd:kk' {5}ld.{2}:kkxo",
            "{6}  'dkkko: {5}lkk.{2}:;,",
            "{5}    :kkkkkdoxkkx",
            "{5}      ;kkkkl...",
        ]),
        distro_color: "\x1b[38;5;39m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "endeavouros",
        raw_lines: &[
            "{w}                     ./{c}o{w}.",
            "{w}                   ./{c}sssso{w}-",
            "{w}                 `:{c}osssssss+{w}-",
            "{w}               `:+{c}sssssssssso{w}/.",
            "{w}             `-/o{c}ssssssssssssso{w}/.",
            "{w}           `-/+{c}sssssssssssssssso{w}+:`",
            "{w}         `-:/+{c}sssssssssssssssssso{w}+/.",
            "{w}       `.://o{c}sssssssssssssssssssso{w}++-",
            "{w}      .://+{c}ssssssssssssssssssssssso{w}++:",
            "{w}    .:///o{c}ssssssssssssssssssssssssso{w}++:",
            "{w}  `:////{c}ssssssssssssssssssssssssssso{w}+++.",
            "{w}`-////+{c}ssssssssssssssssssssssssssso{w}++++-",
            "{w} `..-+{c}oosssssssssssssssssssssssso{w}+++++/`",
            "{w}   ./++++++++++++++++++++++++++++++/:.",
            "{w}  `:::::::::::::::::::::::::------``",
        ],
        compact_lines: Some(&[
            "{c}      /\\",
            "{c}    //  \\\\",
            "{w}  //     \\\\",
            "{c} //       \\\\",
            "{w}/__________\\\\",
        ]),
        distro_color: "\x1b[38;5;127m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "manjaro",
        raw_lines: &[
            "{c}██████████████████  {w}████████",
            "{c}██████████████████  {w}████████",
            "{c}██████████████████  {w}████████",
            "{c}██████████████████  {w}████████",
            "{c}████████            {w}████████",
            "{c}████████  {w}████████  {w}████████",
            "{c}████████  {w}████████  {w}████████",
            "{c}████████  {w}████████  {w}████████",
            "{c}████████  {w}████████  {w}████████",
            "{c}████████  {w}████████  {w}████████",
            "{c}████████  {w}████████  {w}████████",
            "{c}████████  {w}████████  {w}████████",
            "{c}████████  {w}████████  {w}████████",
            "{c}████████  {w}████████  {w}████████",
        ],
        compact_lines: Some(&[
            "{c}||||||||| ||||",
            "{c}||||||||| ||||",
            "{c}||||      ||||",
            "{c}|||| |||| ||||",
            "{c}|||| |||| ||||",
            "{c}|||| |||| ||||",
            "{c}|||| |||| ||||",
        ]),
        distro_color: "\x1b[38;5;34m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "generic",
        raw_lines: &[
            "{w}        #####",
            "{w}       #######",
            "{w}       ##{c}O{w}#{c}O{w}##",
            "{w}       #{w}#####{w}#",
            "{w}     ##{c}##{w}###{c}##{w}##",
            "{w}    #{c}#######{w}####{c}#",
            "{w}   #{c}#############{w}#",
            "{w}  #{c}###############{w}#",
            "{w} #{c}#################{w}#",
            "{w} #{c}#################{w}#",
            "{w} #{c}#################{w}#",
            "{w} #{c}#################{w}#",
            "{w}  #{c}###############{w}#",
            "{w}   {c}#{w}#{c}###########{w}#{c}#",
            "{c}    ####{w}#####{c}####",
            "{c}    ###       ###",
        ],
        compact_lines: Some(&[
            "{w}   .--.",
            "{w}  |{c}o_o {w}|",
            "{w}  |{c}:_/ {w}|",
            "{w} //   \\ \\",
            "{w}(|     | )",
            "{c}/'\\_   _/`\\",
            "{c}\\___)=(___/",
        ]),
        distro_color: "\x1b[38;5;220m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "opensuse",
        raw_lines: &[
            "{w}           .;ldkO0000Okdl;.",
            "{w}       .;d00xl:^''''''^:ok00d;.",
            "{w}     .d00l'                'o00d.",
            "{w}   .d0Kd'{c}  Okxol:;,.          {w}:O0d",
            "{w}  .OK{c}KKK0kOKKKKKKKKKKOxo:,      {w}lKO.",
            "{w} ,0K{c}KKKKKKKKKKKKKKK0P^{w},,,{c}^dx:{w}    ;00,",
            "{w}.OK{c}KKKKKKKKKKKKKKKk'{w}.oOPPb.{c}'0k.{w}   cKO.",
            "{w}:KK{c}KKKKKKKKKKKKKKK: {w}kKx..dd {c}lKd{w}   'OK:",
            "{w}dKK{c}KKKKKKKKKOx0KKKd {w}^0KKKO' {c}kKKc{w}   dKd",
            "{w}dKK{c}KKKKKKKKKK;.;oOKx,..{w}^{c}..;kKKK0.{w}  dKd",
            "{w}:KK{c}KKKKKKKKKK0o;...^cdxxOK0O/^^'  {w}.0K:",
            "{w} kKK{c}KKKKKKKKKKKKK0x;,,......,;od  {w}lKk",
            "{w} '0K{c}KKKKKKKKKKKKKKKKKKKK00KKOo^  {w}c00'",
            "{w}  'kK{c}KKOxddxkOO00000Okxoc;''   {w}.dKk'",
            "{w}    l0Ko.                    .c00l'",
            "{w}     'l0Kk:.              .;xK0l'",
            "{w}        'lkK0xl:;,,,,;:ldO0kl'",
            "{w}            '^:ldxkkkkxdl:^'",
        ],
        compact_lines: Some(&[
            "{c}  _______",
            "{c}__|   __ \\",
            "{c}     / .\\ \\",
            "{c}     \\__/ |",
            "{c}   _______|",
            "{c}   \\_______",
            "{c}__________/",
        ]),
        distro_color: "\x1b[38;5;71m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "alpine",
        raw_lines: &[
            "{w}       .hddddddddddddddddddddddh.",
            "{w}      :dddddddddddddddddddddddddd:",
            "{w}     /dddddddddddddddddddddddddddd/",
            "{w}    +dddddddddddddddddddddddddddddd+",
            "{w}  `sdddddddddddddddddddddddddddddddds`",
            "{w} `ydddddddddddd++hdddddddddddddddddddy`",
            "{w}.hddddddddddd+`  `+ddddh:-sdddddddddddh.",
            "{w}hdddddddddd+`      `+y:    .sddddddddddh",
            "{w}ddddddddh+`   `//`   `.`     -sddddddddd",
            "{w}ddddddh+`   `/hddh/`   `:s-    -sddddddd",
            "{w}ddddh+`   `/+/dddddh/`   `+s-    -sddddd",
            "{w}ddd+`   `/o` :dddddddh/`   `oy-    .yddd",
            "{w}hdddyo+ohddyosdddddddddho+oydddy++ohdddh",
            "{w}.hddddddddddddddddddddddddddddddddddddh.",
            "{w} `yddddddddddddddddddddddddddddddddddy`",
            "{w}  `sdddddddddddddddddddddddddddddddds`",
            "{w}    +dddddddddddddddddddddddddddddd+",
            "{w}     /dddddddddddddddddddddddddddd/",
            "{w}      :dddddddddddddddddddddddddd:",
            "{w}       .hddddddddddddddddddddddh.",
        ],
        compact_lines: Some(&[
            "{w}   /\\ /\\",
            "{c}  // \\  \\",
            "{c} //   \\  \\",
            "{c}///    \\  \\",
            "{c}//      \\  \\",
            "{c}         \\",
        ]),
        distro_color: "\x1b[38;5;32m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "gentoo",
        raw_lines: &[
            "{w}         -/oyddmdhs+:.",
            "{w}       -o{c}dNMMMMMMMMNNmhy+{w}-`",
            "{w}     -y{c}NMMMMMMMMMMMNNNmmdhy{w}+-",
            "{w}   `o{c}mMMMMMMMMMMMMNmdmmmmddhhy{w}/`",
            "{w}   om{c}MMMMMMMMMMMN{w}hhyyyo{c}hmdddhhhdN{w}`",
            "{w}  .yd{c}MMMMMMMMMMMM{w}hhhhhyo{c}yhhdddddhny{w}",
            "{w}  `sh{c}MMMMMMMMMMMM{w}hhhhhhhy{c}yyyyyhddddy{w}",
            "{w}   `d{c}NMMMMMMMMMMM{w}hhhhhhhy{c}yyyyyyddddh{w}`",
            "{w}    .d{c}NMMMMMMMMMM{w}yhhhhhhy{c}yyyyyhddddh{w}-",
            "{w}      -d{c}NMMMMMMMM{w}hhhhhyo{c}yyyyydddddy{w}+",
            "{w}       `+h{c}NMMMMMM{w}hyso/{c}yyyyydddddy{w}+",
            "{w}          `:+syd{w}hyyyyyhddddds{w}+",
            "{w}              `-:+syhhhysoo+.",
        ],
        compact_lines: Some(&[
            "{c} _-----_",
            "{c}(       \\",
            "{c}\\    {w}0{c}  \\",
            "{c} \\       )",
            "{c} /      /",
            "{c}(_____-'",
        ]),
        distro_color: "\x1b[38;5;141m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "void",
        raw_lines: &[
            "{w}                __.;=====;.__",
            "{w}            _.=+==++=++=+=+===;.",
            "{w}             -=+++=+===+=+=+++++=_",
            "{w}        .     -=:``     `--==+=++==.",
            "{w}       :@=._                `-====+=",
            "{w}      ;@@@@@@={c}         .        {w}=*+",
            "{w}    .========{c}         :@@@=._     {w}=",
            "{w}   =++====++={c}        .@@@@@@@@=    {w}:",
            "{w}  :+++=++++=={c}        +@@@@@@@@@.   {w}.",
            "{w}  *+++=++++++{c}       .@@@@@@@@@@    {w}:",
            "{w}  :+++=+++++={c}       .@@@@@@@@@+    {w}.",
            "{w}   =++====++={c}        @@@@@@@@@     {w}:",
            "{w}    .========{c}        :@@@@@@@=    {w}=",
            "{w}      ;@@@@@@={c}         `*@@*'     {w}=*+",
            "{w}       :@=._                `-====+=",
            "{w}        .     -=:``     `--==+=++==.",
            "{w}             -=+++=+===+=+=+++++=_",
            "{w}            _.=+==++=++=+=+===;.",
            "{w}                __.;=====;.__",
        ],
        compact_lines: Some(&[
            "{c}    _______",
            "{c} _ \\______ -",
            "{c}| \\  {w}___  {c}\\ |",
            "{c}| | {w}/   \\ {c}| |",
            "{c}| | {w}\\___/ {c}| |",
            "{c}| \\______ \\_|",
            "{c} -_______\\",
        ]),
        distro_color: "\x1b[38;5;35m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "pop",
        raw_lines: &[
            "{w}             `.-:::-.`",
            "{w}           -+ydmNNNNNNNmdy+-",
            "{w}        .+dNmdhs+//////+shdmdo.",
            "{w}      .smmy+-`             ./sdy:",
            "{w}     +mds-`                   -sdy`",
            "{w}    ymd-   {c}:++o-      -o++:    {w}-dmy",
            "{w}   hmh`    {c}:ydN+      +Ndy:     {w}`hmh",
            "{w}  -mm+     {c}:ydN+      +Ndy:      {w}+mm-",
            "{w}  +mm.     {c}:ydN+      +Ndy:      {w}.mm+",
            "{w}  -mm+     {c}:ydN+      +Ndy:      {w}+mm-",
            "{w}   hmh`    {c}:ydN+      +Ndy:     {w}`hmh",
            "{w}    ymd-   {c}:ydN+      +Ndy:    {w}-dmy",
            "{w}     +mds- {c}:ydN+      +Ndy:  {w}-sdy`",
            "{w}      .smmy{c}:ydN+      +Ndy:{w}./sdy:",
            "{w}        .+dN{c}shs+//////+shd{w}mdo.",
            "{w}           -+ydmNNNNNNNmdy+-",
            "{w}             `.-:::-.`",
        ],
        compact_lines: Some(&[
            "{c}______",
            "{c}\\   _ \\",
            "{c} \\ \\_) )",
            "{c}  \\  __/",
            "{c}   \\_\\",
        ]),
        distro_color: "\x1b[38;5;37m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "nixos",
        raw_lines: &[
            "{w}          ▗▄▄▄       {c}▗▄▄▄▄    ▄▄▄▖",
            "{w}          ▜███▙       {c}▜███▙  ▟███▛",
            "{w}           ▜███▙       {c}▜███▙▟███▛",
            "{w}            ▜███▙       {c}▜██████▛",
            "{w}     ▟█▙     ▜███▙       {c}▜████▛",
            "{w}    ▟███▙     ▜███▙     {c}▟██████▙",
            "{w}   ▟█████▙     ▜███▙   {c}▟███▛▜███▙",
            "{w}  ▟███▛▜███▙    ▜███▙ {c}▟███▛  ▜███▙",
            "{w} ▟███▛  ▜███▙    ▜███{c}▟███▛    ▜███▙",
            "{w}▝▀▀▀▀    ▀▀▀▀▘    ▀▀▀▀▀▀▀      ▀▀▀▀▘",
            "{c}     ▟█████████████████▙ {w}▜████▛     {c}▟▙",
            "{c}    ▟███████████████████▙ {w}▜███▙    {c}▟██▙",
            "{c}           ▄▄▄▄▖           ▜███▙  {w}▟███▛",
            "{c}          ▟███▛             ▜██▛ {w}▟███▛",
            "{c}         ▟███▛               ▜▛ {w}▟███▛",
            "{c}▟███████████▛                  {w}▟██████████▙",
            "{c}▜██████████▛                  {w}▟███████████▛",
            "{c}      ▟███▛ {w}▟▙               {c}▟███▛",
            "{c}     ▟███▛ {w}▟██▙             {c}▟███▛",
            "{c}    ▟███▛  {w}▜███▙           {c}▝▀▀▀▀",
            "{c}    ▜██▛    {w}▜███▙ {c}▜██████████████████▛",
            "{c}     ▜▛     {w}▟████▙ {c}▜████████████████▛",
            "{w}           ▟██████▙       {c}▜███▙",
            "{w}          ▟███▛▜███▙       {c}▜███▙",
            "{w}         ▟███▛  ▜███▙       {c}▜███▙",
            "{w}         ▝▀▀▀    ▀▀▀▀▘       {c}▀▀▀▘",
        ],
        compact_lines: Some(&[
            "{c}  \\\\  {w}\\\\ //",
            "{c} ==\\\\__{w}\\\\/ {c}//",
            "{w}   //   {w}\\\\{c}//",
            "{w}==//     {c}//==",
            "{w} //{c}\\\\{w}___{c}//",
            "{w}// {c}/\\\\  {w}\\\\==",
            "{c}  // \\\\  {w}\\\\",
        ]),
        distro_color: "\x1b[38;5;75m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "kali",
        raw_lines: &[
            "{w}..............",
            "{w}            ..,;:ccc,.",
            "{w}          ......''';lxO.",
            "{w}.....''''..........,:ld;",
            "{w}           .';;;:::;,,.x,",
            "{w}      ..'''.            {c}0Xxoc:,.  ...{w}",
            "{w}  ....                {c},ONkc;,;cokOdc',.{w}",
            "{w} .                   {c}OMo           ':ddo.{w}",
            "{w}                    {c}dMc               :OO;{w}",
            "{w}                    {c}0M.                 .:o.{w}",
            "{w}                    {c};Wd{w}",
            "{w}                     {c};XO,{w}",
            "{w}                       {c},d0Odlc;,..{w}",
            "{w}                           {c}..',;:cdOOd::,.{w}",
            "{w}                                    {c}.:d;.':;.{w}",
            "{w}                                       {c}'d,  .'{w}",
            "{w}                                         {c};l   ..{w}",
            "{w}                                          {c}.o{w}",
            "{w}                                            {c}c{w}",
            "{w}                                            {c}.'{w}",
            "{w}                                             {c}.{w}",
        ],
        compact_lines: Some(&[
            "{c}     -#. #",
            "{c}      @###",
            "{c}  -######",
            "{c} @#########",
            "{c}=##.  .#####",
            "{c}##      ## ##",
            "{c}##       ## #",
            "{c}##       @###",
            "{c}##.        ###",
            "{c} ##%       ##-",
            "{c}  -##%    -*",
            "{c}   :*##+",
            "{c}     :*#*",
            "{c}       -#",
            "{c}        @",
        ]),
        distro_color: "\x1b[38;5;33m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "freebsd",
        raw_lines: &[
            "{w}   ```                        {c}`",
            "{w}  ` `.....---...{c}....--.```   -/{w}",
            "{w}  +o   .--`         {c}/y:`      +.{w}",
            "{w}   yo`:.            {c}:o      `+-{w}",
            "{w}    y/               {c}-/`   -o/{w}",
            "{w}   .-                  {c}::/sy+:.{w}",
            "{w}   /                     {c}`--  /{w}",
            "{w}  `:                          {c}:`{w}",
            "{w}  `:                          {c}:`{w}",
            "{w}   /                          {c}/{w}",
            "{w}   .-                        {c}-.{w}",
            "{w}    --                      {c}-.{w}",
            "{w}     `:`                  {c}`:`{w}",
            "{w}       .--             `--.",
            "{w}          .---.....----.",
        ],
        compact_lines: Some(&[
            "{c}/\\,-'''''-,/\\",
            "{c}\\_)       (_/",
            "{c}|           |",
            "{c}|           |",
            "{c} ;         ;",
            "{c}  '-_____-'",
        ]),
        distro_color: "\x1b[38;5;196m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "slackware",
        raw_lines: &[
            "{w}                  ::::::: ",
            "{w}            ::::::::::::::::::: ",
            "{w}         ::::::::::::::::::::::::: ",
            "{w}       ::::::::{c}cllcccccllllllll{w}:::::: ",
            "{w}    :::::::::{c}lc               dc{w}::::::: ",
            "{w}   ::::::::{c}cl   clllccllll    oc{w}::::::::: ",
            "{w}  :::::::::{c}o   lc{w}::::::::{c}co   oc{w}:::::::::: ",
            "{w} ::::::::::{c}o    cccclc{w}:::::{c}clcc{w}:::::::::::: ",
            "{w} :::::::::::{c}lc        cclccclc{w}::::::::::::: ",
            "{w}::::::::::::::{c}lcclcc          lc{w}:::::::::::: ",
            "{w}::::::::::{c}cclcc{w}:::::{c}lccclc     oc{w}::::::::::: ",
            "{w}::::::::::{c}o    l{w}::::::::::{c}l    lc{w}::::::::::: ",
            "{w} :::::{c}cll{w}:{c}o     clcllcccll     o{w}::::::::::: ",
            "{w} :::::{c}occ{w}:{c}o                  clc{w}::::::::::: ",
            "{w}  ::::{c}ocl{w}:{c}ccslclccclclccclclc{w}::::::::::::: ",
            "{w}   :::{c}oclcccccccccccccllllllllllllll{w}::::: ",
            "{w}    ::{c}lcc1lcccccccccccccccccccccccco{w}:::: ",
            "{w}      :::::::::::::::::::::::::::::::: ",
            "{w}        :::::::::::::::::::::::::::: ",
            "{w}           :::::::::::::::::::::: ",
            "{w}                :::::::::::: ",
        ],
        compact_lines: Some(&[
            "{c}   ________",
            "{c}  /  _____/",
            "{c}  \\____  \\",
            "{c}  /       \\",
            "{c} /______  /",
            "{c}        \\/",
        ]),
        distro_color: "\x1b[38;5;61m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "artix",
        raw_lines: &[
            "{w}                   '",
            "{w}                  'o'",
            "{w}                 'ooo'",
            "{w}                'ooxoo'",
            "{w}               'ooxxxoo'",
            "{w}              'oookkxxoo'",
            "{w}             'oiioxkkxxoo'",
            "{w}            ':;:iiiioxxxoo'",
            "{w}               `'.;::ioxxoo'",
            "{w}          '-.      `':;j{c}iooo{w}'",
            "{w}         'oooio-..     `'{c}i:io{w}'",
            "{w}        'ooooxxxxoio:,.   `'{c}-;'{w}",
            "{w}       'ooooxxxxxkkxoooIi:-.  `'{c}",
            "{w}      'ooooxxxxxkkkkxoiiiiiji'{c}",
            "{w}     'ooooxxxxxkxxoiiii:'`     .{c}i'",
            "{w}    'ooooxxxxxoi:::'`       .;i{c}oxo'",
            "{w}   'ooooxooi::'`         .:iii{c}xkxxo'",
            "{w}  'ooooi:'`                `'';{c}ioxxo'",
            "{w} 'i:'`                          ''{c}:io'",
            "{w}'`                                   `'",
        ],
        compact_lines: Some(&[
            "{c}      /\\",
            "{c}     /  \\",
            "{c}    /`'.,\\",
            "{c}   /     ',",
            "{c}  /      ,/",
            "{c} /__,.-'`",
        ]),
        distro_color: "\x1b[38;5;39m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "zorin",
        raw_lines: &[
            "{w}        `osssssssssssssssssssso`",
            "{w}       .osssssssssssssssssssssso.",
            "{w}      .+oooooooooooooooooooooooo+.",
            "",
            "",
            "{c}  `::::::::::::::::::::::.         .:`",
            "{c} `+ssssssssssssssssss+:.`     `.:+ssso`",
            "{c}.ossssssssssssssso/.       `-+ossssssso.",
            "{c}ssssssssssssso/-`      `-/osssssssssssss",
            "{c}.ossssssso/-`      .-/ossssssssssssssso.",
            "{c} `+sss+:.      `.:+ssssssssssssssssss+`",
            "{c}  `:.         .::::::::::::::::::::::`",
            "",
            "",
            "{w}      .+oooooooooooooooooooooooo+.",
            "{w}       -osssssssssssssssssssssso-",
            "{w}        `osssssssssssssssssssso`",
        ],
        compact_lines: Some(&[
            "{c} ______",
            "{c}|___  /",
            "{c}   / / ",
            "{c}  / /__",
            "{c} /_____|",
        ]),
        distro_color: "\x1b[38;5;39m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "windows11",
        raw_lines: &[
            "{w}                                ..,",
            "{w}                    ....,,:;+ccllll",
            "{w}      ...,,+:;  cllllllllllllllllll",
            "{c},cclllllllllll  {w}lllllllllllllllllll",
            "{c}llllllllllllll  {w}lllllllllllllllllll",
            "{c}llllllllllllll  {w}lllllllllllllllllll",
            "{c}llllllllllllll  {w}lllllllllllllllllll",
            "{c}llllllllllllll  {w}lllllllllllllllllll",
            "{c}llllllllllllll  {w}lllllllllllllllllll",
            "",
            "{w}llllllllllllll  {c}lllllllllllllllllll",
            "{w}llllllllllllll  {c}lllllllllllllllllll",
            "{w}llllllllllllll  {c}lllllllllllllllllll",
            "{w}llllllllllllll  {c}lllllllllllllllllll",
            "{w}llllllllllllll  {c}lllllllllllllllllll",
            "{w}`'ccllllllllll  {c}lllllllllllllllllll",
            "{w}       `' \\*::  {c}:ccllllllllllllllll",
            "{w}                       ````''*::cll",
            "{w}                                 ``",
        ],
        compact_lines: Some(&[
            "{c}####  ####",
            "{c}####  ####",
            "",
            "{c}####  ####",
            "{c}####  ####",
        ]),
        distro_color: "\x1b[38;5;39m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "windows10",
        raw_lines: &[
            "{w}                                ..,",
            "{w}                    ....,,:;+ccllll",
            "{w}      ...,,+:;  cllllllllllllllllll",
            "{c},cclllllllllll  {w}lllllllllllllllllll",
            "{c}llllllllllllll  {w}lllllllllllllllllll",
            "{c}llllllllllllll  {w}lllllllllllllllllll",
            "{c}llllllllllllll  {w}lllllllllllllllllll",
            "{c}llllllllllllll  {w}lllllllllllllllllll",
            "{c}llllllllllllll  {w}lllllllllllllllllll",
            "",
            "{w}llllllllllllll  {c}lllllllllllllllllll",
            "{w}llllllllllllll  {c}lllllllllllllllllll",
            "{w}llllllllllllll  {c}lllllllllllllllllll",
            "{w}llllllllllllll  {c}lllllllllllllllllll",
            "{w}llllllllllllll  {c}lllllllllllllllllll",
            "{w}`'ccllllllllll  {c}lllllllllllllllllll",
            "{w}       `' \\*::  {c}:ccllllllllllllllll",
            "{w}                       ````''*::cll",
            "{w}                                 ``",
        ],
        compact_lines: Some(&[
            "{c}   ,.  .....",
            "{c},$$$  $$$$$",
            "{c}$$$$  $$$$$",
            "{c}            ",
            "{c}$$$$  $$$$$",
            "{c}`$$$  $$$$$",
        ]),
        distro_color: "\x1b[38;5;33m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "windows7",
        raw_lines: &[
            "{4}        ,.=:!!t3Z3z.,",
            "{4}       :tt:::tt333EE3",
            "{4}       Et:::ztt33EEEL{1} @Ee.,      ..,",
            "{4}      ;tt:::tt333EE7{1} ;EEEEEEttttt33#",
            "{4}     :Et:::zt333EEQ.{1} $EEEEEttttt33QL",
            "{4}     it::::tt333EEF{1} @EEEEEEttttt33F",
            "{4}    ;3=*^```\"*4EEV{1} :EEEEEEttttt33@.",
            "{6}    ,.=::::!t=., {4}`{1} @EEEEEEtttz33QF",
            "{6}   ;::::::::zt33){1}   \"4EEEtttji3P*",
            "{6}  :t::::::::tt33.{2}:Z3z..{1}  ``{2} ,..g.",
            "{6}  i::::::::zt33F{2} AEEEtttt::::ztF",
            "{6} ;:::::::::t33V{2} ;EEEttttt::::t3",
            "{6} E::::::::zt33L{2} @EEEtttt::::z3F",
            "{6}{3=*^```\"*4E3){2} ;EEEtttt:::::tZ`",
            "{6}             `{2} :EEEEtttt::::z7",
            "{2}                 \"VEzjt:;;z>*`",
        ],
        compact_lines: Some(&[
            "{4} ,.=:!!t3Z.  {1} @Ee.,",
            "{4}:tt:::tt333  {1};EEEEtt33#",
            "{4}Et:::ztt33E  {1}@EEEEtt33F",
            "{6};::::zt33)   {2} AEEtttztF",
            "{6}:t:::tt33.   {2} ;EEEtt::3",
            "{6} `\"*4E3)     {2} :EEEEttz7",
        ]),
        distro_color: "\x1b[38;5;33m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "android",
        raw_lines: &[
            "{c}         -o          o-",
            "{c}          +hydNNNNdyh+",
            "{c}        +mMMMMMMMMMMMMm+",
            "{c}      `dMM{w}m:{c}NMMMMMMN{w}:m{c}MMd`",
            "{c}      hMMMMMMMMMMMMMMMMMMh",
            "{c}  ..  yyyyyyyyyyyyyyyyyyyy  ..",
            "{c}.mMMm`MMMMMMMMMMMMMMMMMMMM`mMMm.",
            "{c}:MMMM-MMMMMMMMMMMMMMMMMMMM-MMMM:",
            "{c}:MMMM-MMMMMMMMMMMMMMMMMMMM-MMMM:",
            "{c}:MMMM-MMMMMMMMMMMMMMMMMMMM-MMMM:",
            "{c}:MMMM-MMMMMMMMMMMMMMMMMMMM-MMMM:",
            "{c}-MMMM-MMMMMMMMMMMMMMMMMMMM-MMMM-",
            "{c} +yy+ MMMMMMMMMMMMMMMMMMMM +yy+",
            "{c}      MMMMMMMMMMMMMMMMMMMM",
            "{c}      MMMMMMMMMMMMMMMMMMMM",
            "{c}      /++MMMMh++++hMMMM++/",
            "{c}         MMMMo    oMMMM",
            "{c}         MMMMo    oMMMM",
            "{c}         ooss     ssoo",
        ],
        compact_lines: Some(&[
            "{c}  ;,           ,;",
            "{c}   ';,.-----.,;'",
            "{c}  ,'           ',",
            "{c} /    {w}O     O{c}    \\",
            "{c}|                 |",
            "{c}'-----------------'",
        ]),
        distro_color: "\x1b[38;5;118m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "macos",
        raw_lines: &[
            "{1}                    c.'",
            "{1}                 ,xNMM.",
            "{1}               .OMMMMo",
            "{1}               lMM\"",
            "{1}     .;loddo:.  .olloddol;.",
            "{2}   cKMMMMMMMMMMNWMMMMMMMMMM0:",
            "{2} .KMMMMMMMMMMMMMMMMMMMMMMMWd.",
            "{3} XMMMMMMMMMMMMMMMMMMMMMMMX.",
            "{3};MMMMMMMMMMMMMMMMMMMMMMMM:",
            "{4}:MMMMMMMMMMMMMMMMMMMMMMMM:",
            "{4}.MMMMMMMMMMMMMMMMMMMMMMMMX.",
            "{5} kMMMMMMMMMMMMMMMMMMMMMMMMWd.",
            "{5} 'XMMMMMMMMMMMMMMMMMMMMMMMMMMk",
            "{5}  'XMMMMMMMMMMMMMMMMMMMMMMMMK.",
            "{6}    kMMMMMMMMMMMMMMMMMMMMMMd",
            "{6}     ;KMMMMMMMWXXWMMMMMMMk.",
            "{6}       \"cooc*\"    \"*coo'\"",
        ],
        compact_lines: Some(&[
            "{1}     .:'",
            "{2}  __ :'__",
            "{3}.'  `-'  `.",
            "{4}:          :",
            "{5}:          :",
            "{6} :        :",
            "{6}  `-.__.-'",
        ]),
        distro_color: "\x1b[38;5;250m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "openbsd",
        raw_lines: &[
            "{6}                                     _",
            "{6}                                    (_)",
            "{c}              |    .",
            "{c}          .   |L  /|   .         {6} _",
            "{c}      _ . |\\ _| \\--+._/| .       {6}(_)",
            "{c}     / ||\\| Y J  )   / |/| ./",
            "{c}    J  |)'( |        ` F`.'/       {6} _",
            "{c}  -<|  F         __     .-<        {6}(_)",
            "{c}    | /       .-'{6}. {c}`.  /{6}-. {c}L___",
            "{c}    J \\      <    {6}\\ {c} | | {6}O{6}\\{c}|.-'  {6}_",
            "{c}  _J \\  .-    \\{6}/ {6}O {6}| {c}| \\  |{c}F    {6}(_)",
            "{c} '-F  -<_.     \\   .-'  `-' L__",
            "{c}__J  _   _.     >-'  {c}){4}._.   {c}|-'",
            "{c} `-|.'   /_.          {4}\\_|  {c} F",
            "{c}  /.-   .                _.<",
            "{c} /'    /.'             .'  `\\",
            "{c}  /L  /'   |/      _.-'-\\",
            "{c} /'J       ___.---'\\|",
            "{c}   |\\  .--' V  | `. `",
            "{c}   |/`. `-.     `._)",
            "{c}      / .-.\\",
            "{c}      \\ (  `\\",
            "{c}       `.\\",
        ],
        compact_lines: Some(&[
            "{c}      _____",
            "{c}    \\-     -/",
            "{c} \\_/         \\",
            "{c} |        {6}O O{c} |",
            "{c} |_  <   )  3 )",
            "{c} / \\         /",
            "{c}    /-_____-\\",
        ]),
        distro_color: "\x1b[38;5;220m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "netbsd",
        raw_lines: &[
            "{c}                     `-/oshdmNMNdhyo+:-`",
            "{w}y{c}/s+:-``    `.-:+oydNMMMMNhs/-``",
            "{w}-m+{c}NMMMMMMMMMMMMMMMMMMMNdhmNMMMmdhs+/-`",
            "{w} -m+{c}NMMMMMMMMMMMMMMMMMMMMmy+:`",
            "{w}  -N/{c}dMMMMMMMMMMMMMMMds:`",
            "{w}   -N/{c}hMMMMMMMMMmho:`",
            "{w}    -N/{c}-:/++/:.`",
            "{w}     :M+",
            "{w}      :Mo",
            "{w}       :Ms",
            "{w}        :Ms",
            "{w}         :Ms",
            "{w}          :Ms",
            "{w}           :Ms",
            "{w}            :Ms",
            "{w}             :Ms",
            "{w}              :Ms",
        ],
        compact_lines: Some(&[
            "{w} \\`{c}-______,----__",
            "{w}  \\       {c} __,---`_",
            "{w}   \\      {c} `.____",
            "{w}    \\{c}-______,----`-",
            "{w}     \\",
            "{w}      \\",
            "{w}       \\",
            "{w}        \\",
        ]),
        distro_color: "\x1b[38;5;208m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "cachyos",
        raw_lines: &[
            "{w}           .-------------------------:",
            "{w}          .+=========================.",
            "{w}         :++===++==================-       {c}:++-",
            "{w}        :*++====+++++=============-        {c}.==:",
            "{w}       -*+++=====+***++==========:",
            "{w}      =*++++========------------:",
            "{w}     =*+++++=====-                     {c}...",
            "{w}   .+*+++++=-===:                    {c}.=+++=:",
            "{w}  :++++=====-==:                     {c}-*****+",
            "{w} :++========-=.                      {c}.=+**+.",
            "{w}.+==========-.                          {c}.",
            "{w} :+++++++====-                                {c}.--==-.",
            "{w}  :++==========.                             {c}:+++++++:",
            "{w}   .-===========.                            {c}=*****+*+",
            "{w}    .-===========:                           {c}.+*****+:",
            "{w}      -=======++++:::::::::::::::::::::::::-:  {c}.---:",
            "{w}       :======++++====+++******************=.",
            "{w}        :=====+++==========++++++++++++++*-",
            "{w}         .====++==============++++++++++*-",
            "{w}          .===+==================+++++++:",
            "{w}           .-=======================+++:",
            "{w}             ..........................",
        ],
        compact_lines: Some(&[
            "{c}     ...........",
            "{w}    /,,,,{c}...../",
            "{w}   /,,,{c}..{w},,{c}../   {w}()",
            "{w}  /,,,,{c}../",
            "{w} /,,,{c}.../     {c}/'\\",
            "{w}/,{c}...../      {w}\\,/",
            "{w}\\,,,,,,\\\\       {c}_",
            "{w} \\,,{c}....\\\\     {c}/ \\",
            "{c}  \\...{w},{c}..\\\\   {c}\\_/",
            "{c}   \\..{w},,,,,,,,,,,,/",
            "{c}    \\{w},,{c}........{w},,/",
        ]),
        distro_color: "\x1b[38;5;43m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "omarchy",
        raw_lines: &[
            "{c}██████████████████████████████████",
            "{c}██              {w}██              {c}██",
            "{c}██   {w}█████████████       ████   {c}██",
            "{c}██   {w}██                    ██   {c}██",
            "{c}██   {w}██                    ██   {c}██",
            "{c}██   {w}██                    ██   {c}██",
            "{c}██   {w}██                    ██   {c}██",
            "{c}███████                    {w}██   {c}██",
            "{c}██   {w}██                    ██   {c}██",
            "{c}██   {w}██                    ██   {c}██",
            "{c}██   {w}██                    ██   {c}██",
            "{c}██   {w}██                    ██   {c}██",
            "{c}██   {w}████████████████████████   {c}██",
            "{c}██              {w}██              {c}██",
            "{c}██████████████████   █████████████",
        ],
        compact_lines: Some(&[
            "{c}███████████████",
            "{c}██     {w}██    {c}██",
            "{c}██  {w}█████  █ {c}██",
            "{c}██  {w}█      █ {c}██",
            "{c}█████      {w}█ {c}██",
            "{c}██  {w}████████ {c}██",
            "{c}██     {w}██    {c}██",
            "{c}████████  {w}█████",
        ]),
        distro_color: "\x1b[38;5;39m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "garuda",
        raw_lines: &[
            "{w}                   .%;888:8898898:",
            "{w}                 x;XxXB%89b8:b8%b88:",
            "{w}              .8Xxd                8X:.",
            "{w}            .8Xx;                    8x:.",
            "{w}          .tt8x          {c}.d            {w}x88;",
            "{w}       .@8x8;          {c}.db:              {w}xx@;",
            "{w}     ,tSXX°          {c}.bbbbbbbbbbbbbbbbbbbB8x@;",
            "{w}   .SXxx            {c}bBBBBBBBBBBBBBBBBBBBbSBX8;",
            "{w} ,888S              {c}                       pd!",
            "{w}8X88/               {c}                        q",
            "{w}8X88/",
            "{w}GBB.",
            "{w} x%88        {c}d888@8@X@X@X88X@@XX@@X@8@X.",
            "{w}   dxXd    {c}dB8b8b8B8B08bB88b998888b88x.",
            "{w}    dxx8o  {c}                    .@@;.",
            "{w}      dx88 {c}                  .t@x.",
            "{w}        d:SS@8ba89aa67a853Sxxad.",
            "{w}          .d988999889889899dd.",
        ],
        compact_lines: Some(&[
            "{c}     .----.",
            "{c}   .'   {w},{c}  '.",
            "{c} .'    {w}'-----|{c}",
            "{c}'.   {w}-----,{c}",
            "{c}  '.____.'",
        ]),
        distro_color: "\x1b[38;5;208m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "blackarch",
        raw_lines: &[
            "{c}                   00",
            "{c}                   11",
            "{c}                  ====",
            "{w}                  .//",
            "{w}                 `o//:",
            "{w}                `+o//o:",
            "{w}               `+oo//oo:",
            "{w}               -+oo//oo+:",
            "{w}             `/:-:+//ooo+:",
            "{w}            `/+++++//+++++:",
            "{w}           `/++++++//++++++:",
            "{w}          `/+++oooo{c}//{w}ooooooo/`",
            "{w}         ./ooosssso{c}//{w}osssssso+`",
            "{w}        .oossssso-`{c}//{w}`/ossssss+`",
            "{w}       -osssssso.  {c}//{w}  :ssssssso.",
            "{w}      :osssssss/   {c}//{w}   osssso+++.",
            "{w}     /ossssssss/   {c}//{w}   +ssssooo/-",
            "{w}   `/ossssso+/:-   {c}//{w}   -:/+osssso+-",
            "{w}  `+sso+:-`        {c}//{w}       `.-/+oso:",
            "{w} `++:.             {c}//{w}            `-/+/",
            "{w} .`                {c}/{w}                `/",
        ],
        compact_lines: Some(&[
            "{c}      /\\",
            "{c}     /  \\",
            "{c}    / || \\",
            "{c}   /  ||  \\",
            "{c}  /___||___\\",
        ]),
        distro_color: "\x1b[38;5;196m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "parrot",
        raw_lines: &[
            "{c}  `:oho/-`",
            "{c}`mMMMMMMMMMMMNmmdhy-",
            "{c} dMMMMMMMMMMMMMMMMMMs`",
            "{c} +MMsohNMMMMMMMMMMMMMm/",
            "{c} .My   {w}.+dMMMMMMMMMMMMMh.",
            "{c}  +       {w}:NMMMMMMMMMMMMNo",
            "{w}           `yMMMMMMMMMMMMMm:",
            "{w}             /NMMMMMMMMMMMMMy`",
            "{w}              .hMMMMMMMMMMMMMN+",
            "{w}                  ``-NMMMMMMMMMd-",
            "{w}                     /MMMMMMMMMMMs`",
            "{w}                      mMMMMMMMsyNMN/",
            "{w}                      +MMMMMMMo  :sNh.",
            "{w}                      `NMMMMMMm     -o/",
            "{w}                       oMMMMMMM.",
            "{w}                       `NMMMMMM+",
            "{w}                        +MMd/NMh",
            "{w}                         mMm -mN`",
            "{w}                         /MM  `h:",
            "{w}                          dM`   .",
            "{w}                          :M-",
            "{w}                           d:",
            "{w}                           -+",
            "{w}                            -",
        ],
        compact_lines: Some(&[
            "{c}    `:oho/-`",
            "{c}  `mMMMMMMMMh-",
            "{c}  +M{w}o{c}hNMMMMMMm/",
            "{c}  .My  .+dMMMMh.",
            "{c}   +      :NMMNo",
            "{c}           `yMMm:",
            "{c}             /NMMy`",
            "{c}              .hMN+",
            "{c}                 /",
        ]),
        distro_color: "\x1b[38;5;44m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "kubuntu",
        raw_lines: &[
            "{c}           `.:/ossyyyysso/:.",
            "{c}        .:oyyyyyyyyyyyyyyyyyyo:`",
            "{c}      -oyyyyyyy{w}odMMy{c}yyyyyyysyyyyo-",
            "{c}    -syyyyyyyyyy{w}dMMy{c}oyyyy{w}dmMMy{c}yyyys-",
            "{c}   oyyys{w}dMy{c}syyyy{w}dMMMMMMMMMMMMMy{c}yyyyyyo",
            "{c} `oyyyy{w}dMMMMy{c}syysoooooo{w}dMMMMy{c}yyyyyyyyo`",
            "{c} oyyyyyy{w}dMMMMy{c}yyyyyyyyyyys{w}dMMy{c}sssssyyyo",
            "{c}-yyyyyyyy{w}dMy{c}syyyyyyyyyyyyyys{w}dMMMMMy{c}syyy-",
            "{c}oyyyysoo{w}dMy{c}yyyyyyyyyyyyyyyyyy{w}dMMMMy{c}syyyo",
            "{c}yyys{w}dMMMMMy{c}yyyyyyyyyyyyyyyyyysosyyyyyyyy",
            "{c}yyys{w}dMMMMMy{c}yyyyyyyyyyyyyyyyyyyyyyyyyyyyy",
            "{c}oyyyyysos{w}dy{c}yyyyyyyyyyyyyyyyyy{w}dMMMMy{c}syyyo",
            "{c}-yyyyyyyy{w}dMy{c}syyyyyyyyyyyyyys{w}dMMMMMy{c}syyy-",
            "{c} oyyyyyy{w}dMMMy{c}yyyyyyyyyyys{w}dMMy{c}oyyyoyyyo",
            "{c} `oyyyy{w}dMMMy{c}syyyoooooo{w}dMMMMy{c}oyyyyyyyyo`",
            "{c}   oyyysyyoyyyys{w}dMMMMMMMMMMMy{c}yyyyyyyo",
            "{c}    -syyyyyyyyy{w}dMMMy{c}syyy{w}dMMMy{c}syyyys-",
            "{c}      -oyyyyyyy{w}dMMy{c}yyyyyysosyyyyo-",
            "{c}        ./oyyyyyyyyyyyyyyyyyyo/.",
            "{c}           `.:/oosyyyysso/:.`",
        ],
        compact_lines: Some(&[
            "{c}   .:/::/::.",
            "{c}  /  {w}||  //{c} `.",
            "{c} |{w}== || //{c}    |",
            "{c} |   {w}||<<{c}     |",
            "{c} |{w}== || \\\\{c}    |",
            "{c}  \\  {w}||  \\\\{c} .'",
            "{c}   `:|::|::'",
        ]),
        distro_color: "\x1b[38;5;33m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "nobara",
        raw_lines: &[
            "{w}      ⢀⣤⣴⣶⣶⣶⣦⣤⡀⠀⣀⣠⣤⣴⣶⣶⣶⣶⣶⣶⣶⣶⣤⣤⣀⡀",
            "{w}    ⠀⣰⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣶⣤⡀",
            "{w}    ⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄",
            "{w}    ⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄",
            "{w}    ⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣧",
            "{w}    ⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠟⠋⠉⠁⠀⠀⠉⠉⠛⠿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣧",
            "{w}    ⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠟⠁⠀⠀⠀{c}⢀⣀⣀⡀{w}⠀⠀⠀⠈⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇",
            "{w}    ⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡏⠀⠀⠀{c}⢠⣾⣿⣿⣿⣿⣷⡄{w}⠀⠀⠀⠻⠿⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿",
            "{w}    ⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠁⠀⠀⠀{c}⣿⣿⣿⣿⣿⣿⣿⡇{w}⠀⠀⠀⠀⠀⣀⣀⣬⣽⣿⣿⣿⣿⣿⣿",
            "{w}    ⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀{c}⠈⠻⢿⣿⣿⡿⠟⠁{w}⠀⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿",
            "{w}    ⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿",
            "{c} ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣤⣤⣄⣀⡀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿",
            "{c} ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄⠀⠀⠀⠀⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿",
            "{c} ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣇⠀⠀⠀⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿",
            "{c} ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠟⠛⠉⠉⠛⠛⢿⣿⣿⠀⠀⠀⠀⠀⠸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿",
            "{c} ⠘⢿⣿⣿⣿⣿⣿⣿⣿⡿⠋⠀⠀⠀⠀⠀⠀⠀⠀⠈⢿⠀⠀⠀⠀⠀⠀⠙⢿⣿⣿⣿⣿⣿⣿⣿⠟⠁",
            "{c}   ⠈⠙⠛⠛⠛⠋⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠛⠛⠛⠛⠉⠁",
        ],
        compact_lines: Some(&[
            "{w} ⢀⣤⣶⣤⣀",
            "{w} ⣿⣿⣿⣿⣿⣷⣄",
            "{w} ⣿⣿{c}⢠⣶⣄{w}⠈⢿⣿",
            "{c} ⣿⣿{c}⠻⠿⠁{c}  ⢸⣿",
            "{c} ⠘⢿⣿⣶⣤⣤⣾⠟",
        ]),
        distro_color: "\x1b[38;5;197m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "bazzite",
        raw_lines: &[
            "{w}     %%%%%%====%%%%%%%%%%",
            "{w}   %%%%%%%%    %%%%%%%%%%%%%%",
            "{w}  %%%%%%%%%    %%%%%%%%%%%%%%%%",
            "{w}  %%%%%%%%%    %%%%%%%%%%%%%%%{c}###",
            "{w}  %%%%%%%%%    %%%%%%%%%%%%%{c}######",
            "{w}  ==                  =======######",
            "{w}  ==                  ========={c}#####",
            "{w}  %%%%%%%%%    %%%%%%%{c}####======#####",
            "{w}  %%%%%%%%%    %%%%%{c}#######=====#####",
            "{w}  %%%%%%%%%    %%%{c}#########=====#####",
            "{w}  %%%%%%%%%    %{c}%##########=====#####",
            "{w}  %%%%%%%%%===={c}###########=====######",
            "{w}   %%%%%%%%===={c}#########======######",
            "{w}    %%%%%%%====={c}#####========######",
            "{w}     %%%%{c}###===============#######",
            "{w}      %{c}#######==========#########",
            "{c}        #######################",
            "{c}          ###################",
            "{c}              ###########",
        ],
        compact_lines: Some(&[
            "{w} %%%   %%%%",
            "{w} %%%   %%%%{c}##",
            "{w} =       =={c}###",
            "{w} %%%  %{c}###==##",
            "{w}  %%{c}#####==##",
            "{c}    #######",
            "{c}      ###",
        ]),
        distro_color: "\x1b[38;5;141m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "centos",
        raw_lines: &[
            "{w}                 ..",
            "{w}               .PLTJ.",
            "{w}              <><><><>",
            "{w}     KKSSV' 4KKK {c}LJ{w} KKKL.'VSSKK",
            "{w}     KKV' 4KKKKK {c}LJ{w} KKKKAL 'VKK",
            "{w}     V' ' 'VKKKK {c}LJ{w} KKKKV' ' 'V",
            "{w}     .4MA.' 'VKK {c}LJ{w} KKV' '.4Mb.",
            "{w}   . KKKKKA.' 'V {c}LJ{w} V' '.4KKKKK .",
            "{w} .4D KKKKKKKA.'' {c}LJ{w} ''.4KKKKKKK FA.",
            "{w}<QDD ++++++++++++  ++++++++++++ GFD>",
            "{w} 'VD KKKKKKKK'.. {c}LJ{w} ..'KKKKKKKK FV",
            "{w}   ' VKKKKK'. .4 {c}LJ{w} K. .'KKKKKV '",
            "{w}      'VK'. .4KK {c}LJ{w} KKA. .'KV'",
            "{w}     A. . .4KKKK {c}LJ{w} KKKKA. . .4",
            "{w}     KKA. 'KKKKK {c}LJ{w} KKKKK' .4KK",
            "{w}     KKSSA. VKKK {c}LJ{w} KKKV .4SSKK",
            "{w}              <><><><>",
            "{w}               'MKKM'",
            "{w}                 ''",
        ],
        compact_lines: Some(&[
            " {1}____{2}^{3}____",
            " {1}|\\  {2}|{3}  /|",
            " {1}| \\ {2}|{3} / |",
            "{3}<---- {6}---->",
            " {6}| / {1}|{2} \\ |",
            " {6}|/__{1}|{2}__\\|",
            "     {1}v",
        ]),
        distro_color: "\x1b[38;5;214m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "vylen",
        raw_lines: &[
            "{c}          .^^^^^^:    :^^^^^^.",
            "{c}        .:^^^^^^^:    :^^^^^^^:.",
            "{c}      .^^^^^^^^^^:    :^^^^^^^^^^.",
            "{c}    .^^^^^^^^^^^^:    :^^^^^^^^^^^^.",
            "{c}  .^^^^^^^^^^^^^^:    :^^^^^^^^^^^^^^.",
            "{c}.^^^^^^^^^^^^^^^^:    :^^^^^^^^^^^^^^^^:",
            "{c}^^^^^^^^^^^^^^^^^:    :^^^^^^^^^^^^^^^^^",
            "{c}^^^^^^^^^^^^^^^^~:    :~^^^^^^^^^^^^^^^^",
            "{c}..................    ..................",
            "",
            "{c}..................    ..................",
            "{c}^^^^^^^^^^^^^^^^~:    :~^^^^^^^^^^^^^^^^",
            "{c}^^^^^^^^^^^^^^^^^:    :^^^^^^^^^^^^^^^^^",
            "{c}.^^^^^^^^^^^^^^^^:    :^^^^^^^^^^^^^^^^.",
            "{c}  .^^^^^^^^^^^^^^^::::^^^^^^^^^^^^^^^.",
            "{c}    .^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^.",
            "{c}      .^^^^^^^^^^^^^^^^^^^^^^^^^^.",
            "{c}        .:^^^^^^^^^^^^^^^^^^^^:.",
            "{c}          .^^^^^^^^^^^^^^^^^^.",
        ],
        compact_lines: Some(&[
            "{c} \\       /",
            "{c}  \\     / ",
            "{c}   \\   /  ",
            "{c}    \\ /   ",
            "{c}     v    ",
        ]),
        distro_color: "\x1b[38;5;2m",
        outer_color: WHITE_COLOR,
    },
];

/// Resolves a matching `Logo` based on the detected OS string or user override.
pub fn match_logo(
    logo_override: Option<&str>,
    distro_id: &str,
    distro_like: &[String],
) -> Option<&'static Logo> {
    if let Some(name) = logo_override {
        let name_lower = name.to_lowercase();
        if name_lower == "none" || name_lower == "off" {
            return None;
        }
        for logo in ALL_LOGOS {
            if logo.name.eq_ignore_ascii_case(&name_lower) {
                return Some(logo);
            }
        }
        let alias_target = match name_lower.as_str() {
            "kkfetch" | "rust" => Some("ferris"),
            "mint" => Some("linuxmint"),
            "suse" => Some("opensuse"),
            "tux" | "linux" => Some("generic"),
            "win" | "windows" | "win11" => Some("windows11"),
            "win10" => Some("windows10"),
            "win7" => Some("windows7"),
            "mac" | "darwin" | "apple" | "osx" => Some("macos"),
            "bsd" => Some("freebsd"),
            "cachyos" | "cachy" => Some("cachyos"),
            "omarchy" => Some("omarchy"),
            "garuda" => Some("garuda"),
            "blackarch" => Some("blackarch"),
            "parrot" => Some("parrot"),
            "kubuntu" => Some("kubuntu"),
            "centos" | "cent" => Some("centos"),
            "nobara" => Some("nobara"),
            "bazzite" => Some("bazzite"),
            "lfs" | "linuxfromscratch" => Some("generic"),
            _ => None,
        };
        if let Some(target) = alias_target {
            return ALL_LOGOS.iter().find(|l| l.name == target);
        }
    }

    let id_clean = distro_id.to_lowercase().replace(' ', "");

    // 1. Direct exact match
    for logo in ALL_LOGOS {
        if id_clean == logo.name {
            return Some(logo);
        }
    }

    // 2. Specific OS & Distribution keyword matching
    const KEYWORD_LOGOS: &[(&[&str], &str)] = &[
        (&["android", "termux"], "android"),
        (&["macos", "darwin", "osx", "apple"], "macos"),
        (&["freebsd"], "freebsd"),
        (&["openbsd"], "openbsd"),
        (&["netbsd"], "netbsd"),
        (&["windows11", "win11"], "windows11"),
        (&["windows10", "win10"], "windows10"),
        (&["windows7", "win7"], "windows7"),
        (&["windows", "win"], "windows11"),
        (&["cachyos", "cachy"], "cachyos"),
        (&["omarchy"], "omarchy"),
        (&["garuda"], "garuda"),
        (&["blackarch"], "blackarch"),
        (&["endeavour"], "endeavouros"),
        (&["manjaro"], "manjaro"),
        (&["artix"], "artix"),
        (&["arch"], "arch"),
        (&["kubuntu"], "kubuntu"),
        (&["ubuntu"], "ubuntu"),
        (&["mint", "linuxmint"], "linuxmint"),
        (&["parrot"], "parrot"),
        (&["pop"], "pop"),
        (&["kali"], "kali"),
        (&["zorin"], "zorin"),
        (&["debian"], "debian"),
        (&["nobara"], "nobara"),
        (&["bazzite"], "bazzite"),
        (&["vylen"], "vylen"),
        (&["centos", "cent"], "centos"),
        (&["rocky"], "rocky"),
        (&["alma"], "almalinux"),
        (&["fedora"], "fedora"),
        (&["redhat", "rhel"], "rhel"),
        (&["suse", "opensuse"], "opensuse"),
        (&["nix"], "nixos"),
        (&["void"], "void"),
        (&["gentoo"], "gentoo"),
        (&["alpine"], "alpine"),
        (&["lfs", "linuxfromscratch"], "generic"),
        (&["slackware"], "slackware"),
    ];

    for (keywords, logo_name) in KEYWORD_LOGOS {
        if keywords.iter().any(|&k| id_clean.contains(k)) {
            return ALL_LOGOS.iter().find(|l| l.name == *logo_name);
        }
    }

    const LIKE_LOGOS: &[(&str, &str)] = &[
        ("android", "android"),
        ("ubuntu", "ubuntu"),
        ("debian", "debian"),
        ("arch", "arch"),
        ("fedora", "fedora"),
        ("rhel", "rhel"),
        ("centos", "centos"),
        ("rocky", "rocky"),
        ("alma", "almalinux"),
        ("suse", "opensuse"),
    ];

    // 3. Parent ID_LIKE fallbacks
    for like in distro_like {
        let like_lower = like.to_lowercase();
        for (pattern, logo_name) in LIKE_LOGOS {
            if like_lower.contains(pattern) {
                return ALL_LOGOS.iter().find(|l| l.name == *logo_name);
            }
        }
    }

    // Default fallback to generic Linux or ferris crab
    ALL_LOGOS
        .iter()
        .find(|l| l.name == "generic")
        .or_else(|| ALL_LOGOS.iter().find(|l| l.name == "ferris"))
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_match_logo_direct() {
        let logo = match_logo(Some("ubuntu"), "ubuntu", &[]).unwrap();
        assert_eq!(logo.name, "ubuntu");
    }

    #[test]
    fn test_match_logo_like_fallback() {
        let logo = match_logo(None, "my_custom_distro", &["ubuntu".to_string()]).unwrap();
        assert_eq!(logo.name, "ubuntu");
    }

    #[test]
    fn test_match_logo_none_override() {
        assert!(match_logo(Some("none"), "ubuntu", &[]).is_none());
        assert!(match_logo(Some("off"), "ubuntu", &[]).is_none());
    }

    #[test]
    fn test_match_logo_override() {
        let logo = match_logo(Some("arch"), "ubuntu", &[]).unwrap();
        assert_eq!(logo.name, "arch");
    }

    #[test]
    fn test_match_logo_unknown_fallback_to_ferris() {
        let logo = match_logo(None, "unknown_distro", &[]).unwrap();
        assert_eq!(logo.name, "generic");
    }

    #[test]
    fn test_match_logo_kkfetch_and_ferris_aliases() {
        let logo_kk = match_logo(Some("kkfetch"), "generic", &[]).unwrap();
        assert_eq!(logo_kk.name, "ferris");
        let logo_rust = match_logo(Some("rust"), "generic", &[]).unwrap();
        assert_eq!(logo_rust.name, "ferris");
    }

    #[test]
    fn test_match_logo_windows() {
        let logo = match_logo(None, "windows 11", &[]).unwrap();
        assert_eq!(logo.name, "windows11");
        let logo_win10 = match_logo(None, "windows 10", &[]).unwrap();
        assert_eq!(logo_win10.name, "windows10");
        let logo_win7 = match_logo(None, "windows 7", &[]).unwrap();
        assert_eq!(logo_win7.name, "windows7");
    }

    #[test]
    fn test_match_logo_android() {
        let logo = match_logo(None, "android", &[]).unwrap();
        assert_eq!(logo.name, "android");
        let logo_termux = match_logo(None, "termux", &[]).unwrap();
        assert_eq!(logo.name, "android");
        assert_eq!(logo_termux.name, "android");
    }

    #[test]
    fn test_match_logo_macos_and_bsd() {
        let logo_mac = match_logo(None, "macos", &[]).unwrap();
        assert_eq!(logo_mac.name, "macos");
        assert_eq!(logo_mac.raw_lines.len(), 17);

        // Test colored rendering contains rainbow escape sequences
        let colored = logo_mac.render_lines(true);
        assert_eq!(colored.len(), 17);
        assert!(colored[0].contains("\x1b[38;5;76m")); // Green top
        assert!(colored[16].contains("\x1b[38;5;39m")); // Blue bottom

        // Test monochrome rendering cleanly strips all tokens and ANSI escapes
        let plain = logo_mac.render_lines(false);
        assert_eq!(plain.len(), 17);
        assert!(!plain.iter().any(|l| l.contains("\x1b")));
        assert!(!plain.iter().any(|l| l.contains("{1}") || l.contains("{6}")));
        assert!(plain[0].contains("c.'"));

        let logo_darwin = match_logo(None, "darwin", &[]).unwrap();
        assert_eq!(logo_darwin.name, "macos");

        let logo_freebsd = match_logo(None, "freebsd", &[]).unwrap();
        assert_eq!(logo_freebsd.name, "freebsd");
        let logo_openbsd = match_logo(None, "openbsd", &[]).unwrap();
        assert_eq!(logo_openbsd.name, "openbsd");
        let logo_netbsd = match_logo(None, "netbsd", &[]).unwrap();
        assert_eq!(logo_netbsd.name, "netbsd");
    }

    #[test]
    fn test_match_logo_linux_generic_does_not_match_mint() {
        let logo = match_logo(None, "linux", &[]).unwrap();
        assert_eq!(logo.name, "generic");
        assert_ne!(logo.name, "linuxmint");
    }

    #[test]
    fn test_arch_family_logos_and_fallbacks() {
        assert_eq!(match_logo(None, "cachyos", &[]).unwrap().name, "cachyos");
        assert_eq!(match_logo(None, "cachy", &[]).unwrap().name, "cachyos");
        assert_eq!(
            match_logo(None, "endeavouros", &[]).unwrap().name,
            "endeavouros"
        );
        assert_eq!(match_logo(None, "omarchy", &[]).unwrap().name, "omarchy");
        assert_eq!(match_logo(None, "manjaro", &[]).unwrap().name, "manjaro");
        assert_eq!(match_logo(None, "garuda", &[]).unwrap().name, "garuda");
        assert_eq!(
            match_logo(None, "blackarch", &[]).unwrap().name,
            "blackarch"
        );
        // Unknown Arch derivative with ID_LIKE="arch"
        assert_eq!(
            match_logo(None, "hyperbola", &["arch".to_string()])
                .unwrap()
                .name,
            "arch"
        );
    }

    #[test]
    fn test_debian_family_logos_and_fallbacks() {
        assert_eq!(match_logo(None, "ubuntu", &[]).unwrap().name, "ubuntu");
        assert_eq!(
            match_logo(None, "linuxmint", &[]).unwrap().name,
            "linuxmint"
        );
        assert_eq!(match_logo(None, "debian", &[]).unwrap().name, "debian");
        assert_eq!(match_logo(None, "kali", &[]).unwrap().name, "kali");
        assert_eq!(match_logo(None, "pop", &[]).unwrap().name, "pop");
        assert_eq!(match_logo(None, "parrot", &[]).unwrap().name, "parrot");
        assert_eq!(match_logo(None, "kubuntu", &[]).unwrap().name, "kubuntu");
        assert_eq!(match_logo(None, "zorin", &[]).unwrap().name, "zorin");
        // Unknown Debian derivative with ID_LIKE="debian"
        assert_eq!(
            match_logo(None, "pureos", &["debian".to_string()])
                .unwrap()
                .name,
            "debian"
        );
    }

    #[test]
    fn test_redhat_family_logos_and_fallbacks() {
        assert_eq!(match_logo(None, "fedora", &[]).unwrap().name, "fedora");
        assert_eq!(match_logo(None, "rhel", &[]).unwrap().name, "rhel");
        assert_eq!(match_logo(None, "rocky", &[]).unwrap().name, "rocky");
        assert_eq!(match_logo(None, "centos", &[]).unwrap().name, "centos");
        assert_eq!(match_logo(None, "opensuse", &[]).unwrap().name, "opensuse");
        assert_eq!(match_logo(None, "nobara", &[]).unwrap().name, "nobara");
        assert_eq!(match_logo(None, "bazzite", &[]).unwrap().name, "bazzite");
        // Unknown RHEL derivative with ID_LIKE="rhel"
        assert_eq!(
            match_logo(None, "oracle", &["rhel".to_string()])
                .unwrap()
                .name,
            "rhel"
        );
    }

    #[test]
    fn test_niche_family_logos_and_fallbacks() {
        assert_eq!(match_logo(None, "nixos", &[]).unwrap().name, "nixos");
        assert_eq!(match_logo(None, "void", &[]).unwrap().name, "void");
        assert_eq!(match_logo(None, "gentoo", &[]).unwrap().name, "gentoo");
        assert_eq!(match_logo(None, "alpine", &[]).unwrap().name, "alpine");
        assert_eq!(match_logo(None, "lfs", &[]).unwrap().name, "generic");
        assert_eq!(
            match_logo(None, "linuxfromscratch", &[]).unwrap().name,
            "generic"
        );
        // Completely unknown niche distro without ID_LIKE falls back to generic Tux penguin
        assert_eq!(
            match_logo(None, "obscuredistro", &[]).unwrap().name,
            "generic"
        );
        assert_eq!(match_logo(None, "vylen", &[]).unwrap().name, "vylen");
    }

    #[test]
    fn test_compact_logo_variants_defined_and_renderable() {
        for logo in ALL_LOGOS {
            let compact = logo.compact_lines_or_fallback();
            assert!(
                !compact.is_empty(),
                "Compact lines for {} must not be empty",
                logo.name
            );

            // Render both plain and colored variant lines
            let plain = logo.render_variant_lines(LogoVariant::Compact, ColorLevel::None);
            assert_eq!(plain.len(), compact.len());
            assert!(
                !plain.iter().any(|l| l.contains("\x1b")),
                "Monochrome compact lines must strip escapes"
            );

            let colored = logo.render_variant_lines(LogoVariant::Compact, ColorLevel::Color256);
            assert_eq!(colored.len(), compact.len());
        }
    }

    #[test]
    fn test_compact_logo_dimensions_smaller_than_full() {
        // Arch
        let arch = match_logo(Some("arch"), "arch", &[]).unwrap();
        assert!(arch.compact_lines_or_fallback().len() < arch.raw_lines.len());

        // Debian
        let debian = match_logo(Some("debian"), "debian", &[]).unwrap();
        assert!(debian.compact_lines_or_fallback().len() < debian.raw_lines.len());

        // Ubuntu
        let ubuntu = match_logo(Some("ubuntu"), "ubuntu", &[]).unwrap();
        assert!(ubuntu.compact_lines_or_fallback().len() < ubuntu.raw_lines.len());

        // Fedora
        let fedora = match_logo(Some("fedora"), "fedora", &[]).unwrap();
        assert!(fedora.compact_lines_or_fallback().len() < fedora.raw_lines.len());
    }

    #[test]
    fn test_render_variant_lines_color_level_basic16() {
        let debian = match_logo(Some("debian"), "debian", &[]).unwrap();
        let rendered_basic16 = debian.render_variant_lines(LogoVariant::Full, ColorLevel::Basic16);
        // Debian distro_color is 196 (red). On basic16, it degrades to \x1b[91m or \x1b[31m, never \x1b[38;5;
        assert!(!rendered_basic16.iter().any(|l| l.contains("\x1b[38;5;")));
        assert!(rendered_basic16
            .iter()
            .any(|l| l.contains("\x1b[91m") || l.contains("\x1b[31m")));
    }
}
