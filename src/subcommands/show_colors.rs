use crate::cli;
use crate::color;
use crate::colors;
use crate::config;
use crate::delta;
use crate::env::DeltaEnv;
use crate::paint;
use crate::paint::BgShouldFill;
use crate::style;
use crate::utils::bat::output::{OutputType, PagingMode};

#[cfg(not(tarpaulin_include))]
pub fn show_colors() -> std::io::Result<()> {
    use crate::{delta::DiffType, utils};

    let args = std::env::args_os().collect::<Vec<_>>();
    let env = DeltaEnv::default();
    let assets = utils::bat::assets::load_highlighting_assets();

    let opt = match cli::Opt::from_args_and_git_config(args, &env, assets) {
        (cli::Call::Delta(_), Some(opt)) => opt,
        _ => panic!("non-Delta Call variant should not occur here"),
    };

    let config = config::Config::from(opt);
    let pagercfg = (&config).into();

    let mut output_type =
        OutputType::from_mode(&env, PagingMode::QuitIfOneScreen, None, &pagercfg).unwrap();
    let writer = output_type.handle().unwrap();

    let mut painter = paint::Painter::new(writer, &config);
    painter.set_syntax(Some("a.ts"));
    painter.set_highlighter();

    let title_style = ansi_term::Style::new().bold();
    let mut style = style::Style {
        is_syntax_highlighted: true,
        ..style::Style::default()
    };
    for (group, color_names) in colors::color_groups() {
        writeln!(painter.writer, "\n\n{}\n", title_style.paint(group))?;
        for (color_name, hex) in color_names {
            // Two syntax-highlighted lines without background color
            style.ansi_term_style.background = None;
            for line in [
                r#"export function color(): string {{ return "none" }}"#,
                r#"export function hex(): string {{ return "none" }}"#,
            ] {
                painter.syntax_highlight_and_paint_line(
                    line,
                    paint::StyleSectionSpecifier::Style(style),
                    delta::State::HunkZero(DiffType::Unified, None),
                    BgShouldFill::default(),
                )
            }
            // Two syntax-highlighted lines with background color
            let color =
                color::parse_color(color_name, config.true_color, config.git_config()).unwrap();
            style.ansi_term_style.background = Some(color);
            for line in [
                &format!(r#"export function color(): string {{ return "{color_name}" }}"#),
                &format!(r#"export function hex(): string {{ return "{hex}" }}"#),
            ] {
                painter.syntax_highlight_and_paint_line(
                    line,
                    paint::StyleSectionSpecifier::Style(style),
                    delta::State::HunkZero(DiffType::Unified, None),
                    BgShouldFill::default(),
                )
            }
            painter.emit()?;
        }
    }
    Ok(())
}
