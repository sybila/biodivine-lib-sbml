//! The per-document report of a level, for the shape analysis of `notes/g5-report.md` section 7.
//!
//!     cargo run --release --example probe-c24-messages -- 2 > /tmp/l2.txt
use biodivine_lib_sbml::xml::{OptionalXmlChild, XmlProperty};
use biodivine_lib_sbml::Sbml;

fn main() {
    let level = std::env::args().nth(1).unwrap_or_else(|| "2".to_string());
    let mut dirs: Vec<_> = std::fs::read_dir("syntactic")
        .unwrap()
        .map(|it| it.unwrap().path())
        .collect();
    dirs.sort();
    for dir in dirs {
        if !dir.is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().map(|it| it != "xml").unwrap_or(true) {
                continue;
            }
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            if !name.contains(&format!("l{level}v")) {
                continue;
            }
            let Ok(src) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(doc) = Sbml::read_str(&src) else {
                continue;
            };
            let doc_level = doc.level().get_raw().unwrap_or_default();
            if doc_level != level {
                continue;
            }
            if let Ok(issues) = doc.validate() {
                for issue in issues {
                    println!(
                        "{name}|{}|{}",
                        issue.rule(),
                        issue.message().replace('\n', " ")
                    );
                }
            }
        }
    }
}
