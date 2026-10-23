use std::path::Path;

use darklua_core::rules::bundle::BundleRequireMode;
use darklua_core::rules::BundleSourceRange;
use darklua_core::{
    process, BundleConfiguration, Configuration, GeneratorParameters, Options, Resources,
};

const ENTRY: &str = "src/entry.luau";
const OUTPUT: &str = "out/entry.luau";

const FILES: &[(&str, &str)] = &[
    (
        ENTRY,
        "local big = require(\"./big\")
local keys = require(\"./keys\")
local function main()
	return big.x() .. keys.k() .. \"entry_line4\"
end
return main
",
    ),
    (
        "src/big.luau",
        "local A = {}
function A.x() return \"big_line2\" end
export type Big = {
	f1: number,
	f2: number,
	f3: number,
	f4: number,
	f5: number,
	f6: number,
}
return A
",
    ),
    (
        "src/keys.luau",
        "export type Keys = {
	[\"a.b\"]: string,
	[\"c.d\"]: string,
}
local B = {}
function B.k()
	return \"keys_line7\"
end
return B
",
    ),
];

fn bundle() -> (String, Vec<BundleSourceRange>) {
    let resources = Resources::from_memory();
    for (path, content) in FILES {
        resources.write(path, content).unwrap();
    }
    let options = Options::new(ENTRY)
        .with_output(OUTPUT)
        .with_generator_override(GeneratorParameters::RetainLines)
        .with_configuration(Configuration::empty().with_bundle_configuration(
            BundleConfiguration::new(BundleRequireMode::Path(Default::default())),
        ));
    let tree = process(&resources, options).unwrap();
    let ranges: Vec<BundleSourceRange> = tree
        .iter_bundle_source_ranges()
        .flat_map(|(_, ranges)| ranges.to_vec())
        .collect();
    tree.result().unwrap();
    (resources.get(OUTPUT).unwrap(), ranges)
}

#[test]
fn every_marker_line_sits_at_its_mapped_bundle_line() {
    let (bundle, ranges) = bundle();
    let bundle_lines: Vec<&str> = bundle.lines().collect();

    assert_eq!(ranges.len(), FILES.len());

    for range in &ranges {
        let (_, source) = FILES
            .iter()
            .find(|(path, _)| Path::new(path) == range.path)
            .unwrap_or_else(|| panic!("unexpected path {}", range.path.display()));

        for (index, line) in source.lines().enumerate() {
            if !line.contains("_line") {
                continue;
            }
            let bundle_line = range.start_line + index;
            let actual = bundle_lines[bundle_line - 1];
            assert!(
                actual.contains(line.trim()),
                "{} line {} should sit at bundle line {}, found `{}`",
                range.path.display(),
                index + 1,
                bundle_line,
                actual
            );
        }
    }
}

#[test]
fn ranges_are_ordered_and_do_not_overlap() {
    let (_, ranges) = bundle();

    for pair in ranges.windows(2) {
        assert!(pair[0].end_line < pair[1].start_line, "{:?}", pair);
    }
    assert_eq!(ranges.last().unwrap().path, Path::new(ENTRY));
}
