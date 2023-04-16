use egui::text::LayoutJob;

pub fn highlight(ctx: &egui::Context, code: &str) -> LayoutJob {
    impl egui::util::cache::ComputerMut<&str, LayoutJob> for Highlighter {
        fn compute(&mut self, code: &str) -> LayoutJob {
            self.highlight(code)
        }
    }

    type HighlightCache = egui::util::cache::FrameCache<LayoutJob, Highlighter>;

    ctx.memory().caches.cache::<HighlightCache>().get(code)
}

#[allow(dead_code)]
#[derive(Clone, Copy, Hash, PartialEq)]
enum SyntectTheme {
    Base16EightiesDark,
    Base16MochaDark,
    Base16OceanDark,
    Base16OceanLight,
    InspiredGitHub,
    SolarizedDark,
    SolarizedLight,
}

impl SyntectTheme {
    fn syntect_key_name(&self) -> &'static str {
        match self {
            Self::Base16EightiesDark => "base16-eighties.dark",
            Self::Base16MochaDark => "base16-mocha.dark",
            Self::Base16OceanDark => "base16-ocean.dark",
            Self::Base16OceanLight => "base16-ocean.light",
            Self::InspiredGitHub => "InspiredGitHub",
            Self::SolarizedDark => "Solarized (dark)",
            Self::SolarizedLight => "Solarized (light)",
        }
    }
}

const THEME: SyntectTheme = SyntectTheme::Base16EightiesDark;

const SYNTAX: &str = r#"
%YAML 1.2
---
name: jam1asm
file_extensions: [asm, inc]
scope: source.jam1asm

contexts:
  main:
    - match: \.[a-zA-Z_][a-zA-Z0-9_]*
      scope: keyword.directive.jam1asm
    - match: (?i)\b((nop)|(mov)|(inc)|(incc)|(dec)|(in)|(out)|(break)|(lodsb)|(stosb)|(call)|(ret)|(callbd)|(retbd)|(jmp)|(jo)|(jno)|(js)|(jns)|(jz)|(jnz)|(je)|(jne)|(jc)|(jnc)|(jnae)|(jb)|(jae)|(jnb)|(jbe)|(jna)|(ja)|(jnbe)|(jl)|(jnge)|(jge)|(jnl)|(jle)|(jng)|(jg)|(jnle)|(jlc)|(jnlc)|(push)|(pop)|(clc)|(shl)|(shr)|(add)|(addc)|(addac)|(sub)|(subb)|(subae)|(and)|(or)|(xor)|(not)|(cmp)|(test))\b
      scope: keyword.instruction.jam1asm
    - match: \b([0-9][a-zA-Z0-9_]*)\b
      scope: constant.numeric.jam1asm
    - match: '"'
      push: string
    - match: //
      push: line_comment
    - match: \;
      push: line_comment

  string:
    - meta_scope: string.quoted.double.jam1asm
    - match: \\(x[0-9a-fA-F]{2}|u[0-9a-fA-F]{4}|.)
      scope: constant.character.escape.jam1asm
    - match: '"'
      pop: true
    - match: $
      pop: true
    
  line_comment:
    - meta_scope: comment.line.jam1asm
    - match: $
      pop: true
"#;

struct Highlighter {
    ps: syntect::parsing::SyntaxSet,
    ts: syntect::highlighting::ThemeSet,
}

impl Default for Highlighter {
    fn default() -> Self {
        let mut builder = syntect::parsing::SyntaxSetBuilder::new();
        builder.add(syntect::parsing::SyntaxDefinition::load_from_str(SYNTAX, true, None).unwrap());

        Self {
            ps: builder.build(),
            ts: syntect::highlighting::ThemeSet::load_defaults(),
        }
    }
}

impl Highlighter {
    fn highlight(&self, code: &str) -> LayoutJob {
        self.highlight_impl(code).unwrap_or_else(|| {
            // Fallback:
            LayoutJob::simple(
                code.into(),
                egui::FontId::default(),
                egui::Color32::LIGHT_GRAY,
                f32::INFINITY,
            )
        })
    }

    fn highlight_impl(&self, text: &str) -> Option<LayoutJob> {
        use syntect::easy::HighlightLines;
        use syntect::highlighting::FontStyle;
        use syntect::util::LinesWithEndings;

        let syntax = self.ps.find_syntax_by_name("jam1asm")?;

        let theme = THEME.syntect_key_name();
        let mut h = HighlightLines::new(syntax, &self.ts.themes[theme]);

        use egui::text::{LayoutSection, TextFormat};

        let mut job = LayoutJob {
            text: text.into(),
            ..Default::default()
        };

        for line in LinesWithEndings::from(text) {
            for (style, range) in h.highlight_line(line, &self.ps).ok()? {
                let fg = style.foreground;
                let text_color = egui::Color32::from_rgb(fg.r, fg.g, fg.b);
                let italics = style.font_style.contains(FontStyle::ITALIC);
                let underline = style.font_style.contains(FontStyle::UNDERLINE);
                let underline = if underline {
                    egui::Stroke::new(1.0, text_color)
                } else {
                    egui::Stroke::NONE
                };
                job.sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: as_byte_range(text, range),
                    format: TextFormat {
                        font_id: egui::FontId::default(),
                        color: text_color,
                        italics,
                        underline,
                        ..Default::default()
                    },
                });
            }
        }

        Some(job)
    }
}

fn as_byte_range(whole: &str, range: &str) -> std::ops::Range<usize> {
    let whole_start = whole.as_ptr() as usize;
    let range_start = range.as_ptr() as usize;
    assert!(whole_start <= range_start);
    assert!(range_start + range.len() <= whole_start + whole.len());
    let offset = range_start - whole_start;
    offset..(offset + range.len())
}
