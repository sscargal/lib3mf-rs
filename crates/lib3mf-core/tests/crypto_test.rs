use lib3mf_core::parser::crypto_parser::parse_signature;
use lib3mf_core::parser::xml_parser::XmlParser;
use std::io::Cursor;

#[test]
fn test_parse_signature() {
    let xml = r#"
        <Signature xmlns="http://www.w3.org/2000/09/xmldsig#">
            <SignedInfo>
                <CanonicalizationMethod Algorithm="http://www.w3.org/TR/2001/REC-xml-c14n-20010315"/>
                <SignatureMethod Algorithm="http://www.w3.org/2001/04/xmldsig-more#rsa-sha256"/>
                <Reference URI="/3D/3dmodel.model">
                    <Transforms>
                        <Transform Algorithm="http://www.w3.org/2000/09/xmldsig#enveloped-signature"/>
                    </Transforms>
                    <DigestMethod Algorithm="http://www.w3.org/2001/04/xmlenc#sha256"/>
                    <DigestValue>base64digest</DigestValue>
                </Reference>
            </SignedInfo>
            <SignatureValue>base64sig</SignatureValue>
            <KeyInfo>
                <KeyName>1234-5678-UUID</KeyName>
                <KeyValue>
                    <RSAKeyValue>
                        <Modulus>base64mod</Modulus>
                        <Exponent>AQAB</Exponent>
                    </RSAKeyValue>
                </KeyValue>
            </KeyInfo>
        </Signature>
    "#;

    let mut parser = XmlParser::new(Cursor::new(xml));
    // Skip to Signature element or parse directly?
    // standard XmlParser does not auto-advance to specific root unless configured.
    // parse_signature takes parser.
    // We need to advance to start of Signature.
    let _ = parser.read_next_event().unwrap(); // Start <Signature>
    
    let signature = parse_signature(&mut parser).expect("Failed to parse signature");

    assert_eq!(signature.signed_info.canonicalization_method.algorithm, "http://www.w3.org/TR/2001/REC-xml-c14n-20010315");
    assert_eq!(signature.signed_info.signature_method.algorithm, "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256");
    assert_eq!(signature.signed_info.references.len(), 1);
    
    let ref0 = &signature.signed_info.references[0];
    assert_eq!(ref0.uri, "/3D/3dmodel.model");
    assert_eq!(ref0.digest_method.algorithm, "http://www.w3.org/2001/04/xmlenc#sha256");
    assert_eq!(ref0.digest_value.value, "base64digest");
    
    // Check transforms
    assert!(ref0.transforms.is_some());
    assert_eq!(ref0.transforms.as_ref().unwrap().len(), 1);
    assert_eq!(ref0.transforms.as_ref().unwrap()[0].algorithm, "http://www.w3.org/2000/09/xmldsig#enveloped-signature");

    assert_eq!(signature.signature_value.value, "base64sig");
    
    // Check KeyInfo
    let ki = signature.key_info.as_ref().unwrap();
    assert_eq!(ki.key_name.as_deref(), Some("1234-5678-UUID"));
    let rsa = ki.key_value.as_ref().unwrap().rsa_key_value.as_ref().unwrap();
    assert_eq!(rsa.modulus, "base64mod");
    assert_eq!(rsa.exponent, "AQAB");
}
