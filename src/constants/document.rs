use const_format::formatcp;

const XML_VERSION: &str = "1.0";
const XML_ENCODING: &str = "UTF-8";
const XML_STANDALONE: &str = "no";
const XML_DEFAULT_HEADER: &str = formatcp!(
    "<?xml version=\'{}\' encoding=\'{}\' standalone=\'{}\'?>",
    XML_VERSION,
    XML_ENCODING,
    XML_STANDALONE
);

const SBML_DEFAULT_LEVEL: &str = "3";
const SBML_DEFAULT_VERSION: &str = "2";
const SBML_DEFAULT_NAMESPACE: &str = formatcp!(
    "http://www.sbml.org/sbml/level{}/version{}/core",
    SBML_DEFAULT_LEVEL,
    SBML_DEFAULT_VERSION
);
const SBML_DEFAULT_ROOT: &str = formatcp!(
    "<sbml xmlns=\"{}\" level=\"{}\" version=\"{}\"></sbml>",
    SBML_DEFAULT_NAMESPACE,
    SBML_DEFAULT_LEVEL,
    SBML_DEFAULT_VERSION
);

pub const SBML_DEFAULT_DOCUMENT: &str = formatcp!("{}{}", XML_DEFAULT_HEADER, SBML_DEFAULT_ROOT);
