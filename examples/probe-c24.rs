use biodivine_lib_sbml::Sbml;
fn main() {
    for path in std::env::args().skip(1) {
        let src = std::fs::read_to_string(&path).unwrap();
        let doc = Sbml::read_str(&src).unwrap();
        let rules: Vec<String> = doc
            .validate()
            .unwrap()
            .iter()
            .map(|i| i.rule().to_string())
            .collect();
        println!("{path}: {rules:?}");
    }
}
