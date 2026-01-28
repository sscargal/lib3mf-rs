use lib3mf_core::model::{CapMode, ClippingMode, Geometry};
use lib3mf_core::parser::parse_model;
use std::io::Cursor;

#[test]
fn test_parse_beam_lattice() -> anyhow::Result<()> {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="10" y="0" z="0" />
                    <vertex x="0" y="10" z="0" />
                    <vertex x="0" y="0" z="10" />
                </vertices>
                <beamlattice minlength="0.1" precision="0.01" clippingmode="inside">
                    <beams>
                        <beam v1="0" v2="1" r1="1.0" r2="1.0" cap="hemisphere" />
                        <beam v1="0" v2="2" r1="1.5" />
                        <beam v1="0" v2="3" r1="0.5" cap="butt" />
                    </beams>
                    <beamsets>
                        <beamset name="Set1" identifier="BS1">
                            <ref index="0" />
                            <ref index="1" />
                        </beamset>
                    </beamsets>
                </beamlattice>
            </mesh>
        </object>
    </resources>
    <build>
        <item objectid="1" />
    </build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;
    let obj = model.resources.get_object(lib3mf_core::model::ResourceId(1)).expect("Object missing");
    
    if let Geometry::Mesh(mesh) = &obj.geometry {
        assert_eq!(mesh.vertices.len(), 4);
        
        let lattice = mesh.beam_lattice.as_ref().expect("Beam lattice missing");
        
        // Check lattice attributes
        assert_eq!(lattice.min_length, 0.1);
        assert_eq!(lattice.precision, 0.01);
        assert_eq!(lattice.clipping_mode, ClippingMode::Inside);
        
        // Check Beams
        assert_eq!(lattice.beams.len(), 3);
        
        // Beam 0
        let b0 = &lattice.beams[0];
        assert_eq!(b0.v1, 0); 
        assert_eq!(b0.v2, 1);
        assert_eq!(b0.r1, 1.0);
        assert_eq!(b0.r2, 1.0);
        assert_eq!(b0.cap_mode, CapMode::Hemisphere);
        
        // Beam 1
        let b1 = &lattice.beams[1];
        assert_eq!(b1.v1, 0); 
        assert_eq!(b1.v2, 2);
        assert_eq!(b1.r1, 1.5);
        // r2 default logic is "same as r1" if missing? My parser impl: `unwrap_or(r1)`.
        assert_eq!(b1.r2, 1.5); 
        assert_eq!(b1.cap_mode, CapMode::Sphere); // Default
        
        // Beam 2
        let b2 = &lattice.beams[2];
        assert_eq!(b2.cap_mode, CapMode::Butt);
        
        // Check BeamSets
        assert_eq!(lattice.beam_sets.len(), 1);
        let bs = &lattice.beam_sets[0];
        assert_eq!(bs.name, Some("Set1".to_string()));
        assert_eq!(bs.identifier, Some("BS1".to_string()));
        assert_eq!(bs.refs.len(), 2);
        assert_eq!(bs.refs[0], 0);
        assert_eq!(bs.refs[1], 1);
        
    } else {
        panic!("Geometry is not a mesh");
    }

    Ok(())
}
