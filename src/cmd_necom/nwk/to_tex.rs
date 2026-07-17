use anyhow::Context;
use clap::{Arg, ArgAction, ArgMatches, Command};
use necom::libs::phylo::tree::io::{compute_scale_bar, to_forest};
use necom::libs::phylo::tree::Tree;
use std::io::Read;

/// Build the clap subcommand for to-tex.
pub fn make_subcommand() -> Command {
    Command::new("to-tex")
        .about("Converts Newick trees to a full LaTeX document")
        .after_help(
            r###"
Convert Newick trees to a full LaTeX document.

Notes:
* Styles are stored in the comments of each node
* Drawing a cladogram by default
* Set `--bl` to draw a phylogenetic tree
* Underscore `_` is a control character in LaTeX
  * All `_`s in names, labels and comments will be replaced as spaces " "
* Other LaTeX special characters (`{ } \ # $ % & ~ ^`) in names, labels and comments are escaped automatically
* To compile the .tex files to pdf, you need LaTeX or Tectonic
  * `XeLaTeX` and `latexmk` for compiling unicode .tex
  * `xeCJK` package for East Asian characters
  * `Forest` package for drawing trees

Examples:
1. Generate LaTeX file:
   necom nwk to-tex tests/newick/catarrhini.nwk -o tree.tex

2. Compile with Tectonic (recommended):
   tectonic tree.tex

3. Compile with Latexmk:
   latexmk -xelatex tree.tex
   latexmk -c tree.tex
"###,
        )
        .arg(crate::cmd_necom::args::infile_arg_required())
        .arg(crate::cmd_necom::args::bl_arg())
        .arg(
            Arg::new("forest")
                .long("forest")
                .action(ArgAction::SetTrue)
                .help("Treat input as a file containing pre-generated Forest code (pass-through mode)"),
        )
        .arg(
            Arg::new("no_default_style")
                .long("no-default-style")
                .action(ArgAction::SetTrue)
                .help("Skip default font settings in the template to allow custom styles"),
        )
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the to-tex command.
///
/// The complete LaTeX document is built from `src/assets/template.tex`, which
/// must contain the markers `%FOREST_BEGIN`, `%FOREST_END`, `%STYLE_BEGIN`, and
/// `%STYLE_END`. Forest content replaces the `%FOREST_*` region. Unless
/// `--no-default-style` is given, the `%STYLE_*` region is replaced with a
/// `Noto Sans` font setup; otherwise the template's original font setup is kept.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer =
        necom::writer(outfile).with_context(|| format!("Failed to open writer for {}", outfile))?;
    let is_bl = args.get_flag("bl");
    let no_default_style = args.get_flag("no_default_style");

    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;

    let out_string = if args.get_flag("forest") {
        let mut reader = necom::reader(infile)
            .with_context(|| format!("Failed to open reader for {}", infile))?;
        let mut s = String::new();
        reader
            .read_to_string(&mut s)
            .with_context(|| format!("Failed to read from {}", infile))?;

        s
    } else {
        let tree = Tree::from_file(infile)?
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("no trees found in {}", infile))?;

        let height = if is_bl {
            tree.get_root()
                .map(|r| tree.get_height(r, true))
                .unwrap_or(0.0)
        } else {
            0.0
        };
        let mut s = to_forest(&tree, height);

        // a bar of unit length
        if is_bl && height > 0.0 {
            let (scale, bar_mm) = compute_scale_bar(height);

            // Draw scale bar
            s += "\\draw[-, grey, line width=1pt]";
            s += " ($(current bounding box.south east)+(-10mm,-2mm)$)";
            s += &format!(" --++ (-{}mm,0mm)", bar_mm);
            s += &format!(" node[midway, below]{{\\scriptsize{{{}}}}};\n", scale);
        }

        s
    };

    static FILE_TEMPLATE: &str = include_str!("../../assets/template.tex");
    let mut template = FILE_TEMPLATE.to_string();

    {
        // Section forest
        let begin = template
            .find("%FOREST_BEGIN")
            .ok_or_else(|| anyhow::anyhow!("template marker %FOREST_BEGIN missing"))?;
        let end = template
            .find("%FOREST_END")
            .ok_or_else(|| anyhow::anyhow!("template marker %FOREST_END missing"))?;
        anyhow::ensure!(begin < end, "template markers %FOREST out of order");
        let after_end = end + "%FOREST_END".len();
        template.replace_range(begin..after_end, &out_string);
    }

    let default_font = r#"\setmainfont{NotoSans}[
    Extension      = .ttf,
    UprightFont    = *-Regular,
    BoldFont       = *-Bold,
    ItalicFont     = *-Italic,
    BoldItalicFont = *-BoldItalic
]
"#;

    // Section style: always replace the marker region. With --no-default-style,
    // keep the template's original content between the markers; otherwise inject
    // the Noto Sans setup.
    let style_begin = template
        .find("%STYLE_BEGIN")
        .ok_or_else(|| anyhow::anyhow!("template marker %STYLE_BEGIN missing"))?;
    let style_end = template
        .find("%STYLE_END")
        .ok_or_else(|| anyhow::anyhow!("template marker %STYLE_END missing"))?;
    anyhow::ensure!(
        style_begin < style_end,
        "template markers %STYLE out of order"
    );
    let style_after_end = style_end + "%STYLE_END".len();
    let style_replacement = if no_default_style {
        template[style_begin + "%STYLE_BEGIN".len()..style_end].to_string()
    } else {
        default_font.to_string()
    };
    template.replace_range(style_begin..style_after_end, &style_replacement);

    writer.write_all(template.as_ref())?;

    writer.flush()?;
    Ok(())
}
