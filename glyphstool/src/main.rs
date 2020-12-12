use std::fs;
use std::path::{Path, PathBuf};

use structopt::StructOpt;

mod inco_fix;

#[derive(StructOpt, Debug)]
enum Cmd {
    Merge(MergeCmd),
    IncoFix(IncoFixCmd),
    IncoScale(IncoScaleCmd),
    IncoSyms(IncoSymsCmd),
}

#[derive(StructOpt, Debug)]
struct MergeCmd {
    /// The font file to merge in.
    #[structopt(parse(from_os_str))]
    font: PathBuf,

    /// The other font file, to use as a source.
    #[structopt(parse(from_os_str))]
    other: PathBuf,

    /// The layer to merge (UUID).
    layer: String,
}

#[derive(StructOpt, Debug)]
struct IncoFixCmd {
    /// The font file to operate on.
    #[structopt(parse(from_os_str))]
    font: PathBuf,
}

#[derive(StructOpt, Debug)]
struct IncoScaleCmd {
    /// The font file to operate on.
    #[structopt(parse(from_os_str))]
    font: PathBuf,

    /// Subcommand. 0: numerics. 1: ord.
    ///
    /// This should be an enum, but int is easier.
    subcmd: i32,
}

#[derive(StructOpt, Debug)]
struct IncoSymsCmd {
    /// The font file to operate on.
    #[structopt(parse(from_os_str))]
    font: PathBuf,
}

use glyphstool::{ops, Font, FromPlist, Plist, ToPlist};

fn read_font(path: &Path) -> Font {
    let contents = fs::read_to_string(path).expect("error reading font file");
    let plist = Plist::parse(&contents).expect("error parsing font file");
    FromPlist::from_plist(plist)
}

fn write_font(path: &Path, font: Font) {
    let plist = font.to_plist();
    fs::write(path, &plist.to_string()).unwrap();
}

fn main() {
    let cmd = Cmd::from_args();

    match cmd {
        Cmd::Merge(m) => {
            println!("merge {:?}", m);
            let mut font = read_font(&m.font);
            let other = read_font(&m.other);
            ops::merge(&mut font, &other, &m.layer);
            write_font(&m.font, font);
        }
        Cmd::IncoFix(m) => {
            let mut font = read_font(&m.font);
            inco_fix::inco_fix(&mut font);
            write_font(&m.font, font);
        }
        Cmd::IncoScale(m) => {
            let mut font = read_font(&m.font);
            inco_fix::inco_scale(&mut font, m.subcmd);
            write_font(&m.font, font);
        }
        Cmd::IncoSyms(m) => {
            let mut font = read_font(&m.font);
            inco_fix::inco_syms(&mut font);
            write_font(&m.font, font);
        }
    }
    /*
    let mut filename = None;
    for arg in env::args().skip(1) {
        if filename.is_none() {
            filename = Some(arg);
        }
    }
    if filename.is_none() {
        usage();
        return;
    }
    let filename = filename.unwrap();
    let contents = fs::read_to_string(filename).expect("error reading font");
    let plist = Plist::parse(&contents).expect("parse error");
    //println!("Plist: {:?}", plist);
    /*
    let font = Font::from_plist(plist);
    for glyph in font.glyphs() {
        println!("glyphname: {}", glyph.glyphname());
        for layer in glyph.layers() {
            println!("  layer: {}, width = {}", layer.layer_id(), layer.width());
        }
    }
    */
    let mut font: Font = FromPlist::from_plist(plist);
    //println!("{:?}", font);
    stretch(&mut font, 0.5, "051EFAE4-8BBE-4FBB-A016-4335C3E52F59");
    let plist = font.to_plist();
    println!("{}", plist.to_string());
    */
}
