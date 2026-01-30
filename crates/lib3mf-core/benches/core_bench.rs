use criterion::{Criterion, criterion_group, criterion_main};
use lib3mf_core::parser::model_parser::parse_model;
use std::io::Cursor;

fn bench_parse_benchy(c: &mut Criterion) {
    let _data = include_bytes!("../../../models/Benchy.3mf");
    // Note: Benchy.3mf is a ZIP, but parse_model expects the unzipped .model content.
    // However, for benchmarking, we can just benchmark the parsing of a large enough unzipped slice
    // or keep it simple for now.
    // In actual usage, we'd unzip first.

    // Let's assume we have a raw model file for pure parser benchmarking.
    // For now, we'll benchmark with a mock large XML or just the root part.
    let root_model = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="100" y="0" z="0" />
                    <vertex x="100" y="100" z="0" />
                    <vertex x="0" y="100" z="0" />
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2" />
                    <triangle v2="0" v3="3" v1="2" />
                </triangles>
            </mesh>
        </object>
    </resources>
    <build>
        <item objectid="1" />
    </build>
</model>"#;

    c.bench_function("parse_root_model", |b: &mut criterion::Bencher| {
        b.iter(|| {
            let _ = parse_model(Cursor::new(root_model));
        })
    });
}

criterion_group!(benches, bench_parse_benchy);
criterion_main!(benches);
