use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub highlight: Color,
    pub border: Color,
    pub text_secondary: Color,
}

pub struct ThemeManager;

impl ThemeManager {
    pub fn get_themes() -> Vec<Theme> {
        vec![
            // Kanagawa
            Theme {
                name: "Kanagawa".to_string(),
                background: Color::Rgb(31, 31, 46),
                foreground: Color::Rgb(210, 210, 210),
                primary: Color::Rgb(127, 173, 173),
                secondary: Color::Rgb(193, 167, 127),
                accent: Color::Rgb(224, 138, 138),
                highlight: Color::Rgb(60, 60, 80),
                border: Color::Rgb(80, 80, 100),
                text_secondary: Color::Rgb(150, 150, 150),
            },
            // Dracula
            Theme {
                name: "Dracula".to_string(),
                background: Color::Rgb(40, 42, 54),
                foreground: Color::Rgb(248, 248, 242),
                primary: Color::Rgb(189, 147, 249),
                secondary: Color::Rgb(80, 250, 123),
                accent: Color::Rgb(255, 85, 85),
                highlight: Color::Rgb(68, 71, 90),
                border: Color::Rgb(98, 114, 164),
                text_secondary: Color::Rgb(150, 150, 150),
            },
            // Gruvbox
            Theme {
                name: "Gruvbox".to_string(),
                background: Color::Rgb(40, 40, 40),
                foreground: Color::Rgb(235, 219, 178),
                primary: Color::Rgb(254, 128, 25),
                secondary: Color::Rgb(184, 187, 38),
                accent: Color::Rgb(251, 73, 52),
                highlight: Color::Rgb(60, 56, 54),
                border: Color::Rgb(124, 111, 100),
                text_secondary: Color::Rgb(168, 153, 132),
            },
            // Nord
            Theme {
                name: "Nord".to_string(),
                background: Color::Rgb(46, 52, 64),
                foreground: Color::Rgb(216, 222, 233),
                primary: Color::Rgb(136, 192, 208),
                secondary: Color::Rgb(163, 190, 140),
                accent: Color::Rgb(191, 97, 106),
                highlight: Color::Rgb(59, 66, 82),
                border: Color::Rgb(76, 86, 106),
                text_secondary: Color::Rgb(150, 150, 150),
            },
            // Solarized (Dark)
            Theme {
                name: "Solarized (Dark)".to_string(),
                background: Color::Rgb(0, 43, 54),
                foreground: Color::Rgb(131, 148, 150),
                primary: Color::Rgb(38, 139, 210),
                secondary: Color::Rgb(133, 153, 0),
                accent: Color::Rgb(220, 50, 47),
                highlight: Color::Rgb(7, 54, 66),
                border: Color::Rgb(88, 110, 117),
                text_secondary: Color::Rgb(101, 123, 131),
            },
            // Catppuccin (Mocha)
            Theme {
                name: "Catppuccin (Mocha)".to_string(),
                background: Color::Rgb(30, 30, 46),
                foreground: Color::Rgb(205, 214, 244),
                primary: Color::Rgb(137, 180, 250),
                secondary: Color::Rgb(166, 227, 161),
                accent: Color::Rgb(243, 139, 168),
                highlight: Color::Rgb(49, 50, 68),
                border: Color::Rgb(88, 91, 112),
                text_secondary: Color::Rgb(147, 153, 178),
            },
            // Tokyo Night
            Theme {
                name: "Tokyo Night".to_string(),
                background: Color::Rgb(24, 25, 38),
                foreground: Color::Rgb(169, 177, 214),
                primary: Color::Rgb(122, 162, 247),
                secondary: Color::Rgb(158, 206, 106),
                accent: Color::Rgb(247, 118, 142),
                highlight: Color::Rgb(31, 32, 48),
                border: Color::Rgb(56, 58, 79),
                text_secondary: Color::Rgb(92, 99, 122),
            },
            // One Dark
            Theme {
                name: "One Dark".to_string(),
                background: Color::Rgb(40, 44, 52),
                foreground: Color::Rgb(171, 178, 191),
                primary: Color::Rgb(97, 175, 239),
                secondary: Color::Rgb(152, 195, 121),
                accent: Color::Rgb(224, 108, 117),
                highlight: Color::Rgb(49, 54, 63),
                border: Color::Rgb(65, 70, 81),
                text_secondary: Color::Rgb(140, 140, 140),
            },
            // Monokai
            Theme {
                name: "Monokai".to_string(),
                background: Color::Rgb(39, 40, 34),
                foreground: Color::Rgb(248, 248, 242),
                primary: Color::Rgb(102, 217, 239),
                secondary: Color::Rgb(166, 226, 46),
                accent: Color::Rgb(249, 38, 114),
                highlight: Color::Rgb(56, 57, 50),
                border: Color::Rgb(73, 72, 62),
                text_secondary: Color::Rgb(150, 150, 150),
            },
            // Everforest (Dark)
            Theme {
                name: "Everforest (Dark)".to_string(),
                background: Color::Rgb(45, 52, 64),
                foreground: Color::Rgb(211, 208, 191),
                primary: Color::Rgb(116, 178, 123),
                secondary: Color::Rgb(229, 192, 123),
                accent: Color::Rgb(228, 108, 117),
                highlight: Color::Rgb(56, 64, 79),
                border: Color::Rgb(78, 87, 105),
                text_secondary: Color::Rgb(148, 142, 129),
            },
        ]
    }
}
