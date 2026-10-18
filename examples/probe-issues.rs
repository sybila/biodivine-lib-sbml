//! Print every issue of one document, or of a whole corpus directory, one line per issue.
//!
//! The measurement tools of `scripts/` compare the library's report against the expectation
//! files of the corpus, which answers "which rule of a document disagrees" but not "what did the
//! library actually report". This probe answers the second question for one document, which is
//! what an implementation of a rule needs while it is being written:
//!
//!     cargo run --release --example probe-issues -- <file.xml>
//!     cargo run --release --example probe-issues -- <directory>      # one run per *.xml
//!
//! The output is one `path|rule|severity|element|message` line per issue, prefixed by a
//! `model=<bool>` marker per document, so that a missing report is distinguishable from a
//! document whose object model did not resolve.
use biodivine_lib_sbml::xml::OptionalXmlChild;
use biodivine_lib_sbml::Sbml;

fn main() {
    let target = std::env::args().nth(1).expect(
        "usage: probe-issues <file.xml> | <directory> (the corpus directory it recurses into)",
    );

    let mut paths = Vec::new();
    let metadata = std::fs::metadata(&target).expect("the target exists");
    if metadata.is_dir() {
        let mut stack = vec![std::path::PathBuf::from(&target)];
        while let Some(directory) = stack.pop() {
            let mut entries: Vec<_> = std::fs::read_dir(&directory)
                .expect("a readable directory")
                .map(|it| it.unwrap().path())
                .collect();
            entries.sort();
            for entry in entries {
                if entry.is_dir() {
                    stack.push(entry);
                } else if entry.extension().map(|it| it == "xml").unwrap_or(false) {
                    paths.push(entry);
                }
            }
        }
    } else {
        paths.push(std::path::PathBuf::from(&target));
    }
    paths.sort();

    for path in paths {
        let Ok(source) = std::fs::read_to_string(&path) else {
            println!("{}|UNREADABLE", path.display());
            continue;
        };
        match Sbml::read_str(&source) {
            Ok(document) => {
                println!(
                    "{}|model={}",
                    path.display(),
                    document.model().get().is_some()
                );
                match document.validate() {
                    Ok(issues) => {
                        for issue in issues {
                            println!(
                                "{}|{}|{:?}|{}|{}",
                                path.display(),
                                issue.rule(),
                                issue.severity(),
                                issue.element().local_name(),
                                issue.message()
                            );
                        }
                    }
                    Err(error) => println!("{}|CANCELLED|{}", path.display(), error),
                }
            }
            Err(error) => println!("{}|READ-ERROR|{}", path.display(), error),
        }
    }
}
