use der_parser::*;
use nom::{IResult,Err,ErrorKind};

#[allow(non_snake_case)]

#[derive(Debug)]
pub struct X509Certificate<'a> {
    tbsCertificate: DerObject<'a>,
    signatureAlgorithm: DerObject<'a>,
    signatureValue: DerObject<'a>,
}

impl<'a> X509Certificate<'a> {
    pub fn new(mut v: Vec<DerObject>) -> X509Certificate {
        X509Certificate{
            // XXX note, reverse order
            signatureValue:     v.pop().unwrap(),
            signatureAlgorithm: v.pop().unwrap(),
            tbsCertificate:     v.pop().unwrap(),
        }
    }
}


#[inline]
fn parse_directory_string(i:&[u8]) -> IResult<&[u8],DerObject> {
    alt_complete!(i,
                  parse_der_utf8string |
                  parse_der_printablestring |
                  parse_der_ia5string |
                  parse_der_t61string |
                  parse_der_bmpstring)
}

#[inline]
fn parse_attr_type_and_value(i:&[u8]) -> IResult<&[u8],DerObject> {
    parse_der_sequence_defined!(i,
                                parse_der_oid,
                                parse_directory_string
                               )
}

#[inline]
fn parse_rdn(i:&[u8]) -> IResult<&[u8],DerObject> {
    parse_der_set_of!(i, parse_attr_type_and_value)
}

#[inline]
fn parse_name(i:&[u8]) -> IResult<&[u8],DerObject> {
    parse_der_sequence_of!(i, parse_rdn)
}

#[inline]
pub fn parse_version(i:&[u8]) -> IResult<&[u8],DerObject> {
    parse_der_explicit(i, 0, parse_der_integer)
}

#[inline]
fn parse_choice_of_time(i:&[u8]) -> IResult<&[u8],DerObject> {
    alt_complete!(i, parse_der_utctime | parse_der_generalizedtime)
}

#[inline]
fn parse_validity(i:&[u8]) -> IResult<&[u8],DerObject> {
    parse_der_sequence_defined!(i,
                                parse_choice_of_time,
                                parse_choice_of_time
                               )
}

#[inline]
fn parse_subject_public_key_info(i:&[u8]) -> IResult<&[u8],DerObject> {
    parse_der_sequence_defined!(i,
                                parse_algorithm_identifier,
                                parse_der_bitstring
                               )
}

#[inline]
fn der_read_bitstring_content(i:&[u8], _tag:u8, len: usize) -> IResult<&[u8],DerObjectContent,u32> {
    der_read_element_content_as(i, DerTag::BitString as u8, len)
}

#[inline]
fn parse_issuer_unique_id(i:&[u8]) -> IResult<&[u8],DerObject> {
    parse_der_implicit(i, 1, der_read_bitstring_content)
}

#[inline]
fn parse_subject_unique_id(i:&[u8]) -> IResult<&[u8],DerObject> {
    parse_der_implicit(i, 2, der_read_bitstring_content)
}

#[inline]
fn der_read_opt_bool(i:&[u8]) -> IResult<&[u8],DerObject,u32> {
    parse_der_optional!(i, parse_der_bool)
}

#[inline]
fn parse_extension(i:&[u8]) -> IResult<&[u8],DerObject> {
    parse_der_sequence_defined!(
        i,
        parse_der_oid,
        der_read_opt_bool,
        parse_der_octetstring
    )
}

#[inline]
fn parse_extension_sequence(i:&[u8]) -> IResult<&[u8],DerObject> {
    parse_der_sequence_of!(i, parse_extension)
}

#[inline]
fn parse_extensions(i:&[u8]) -> IResult<&[u8],DerObject> {
    parse_der_explicit(i, 3, parse_extension_sequence)
}


pub fn parse_tbs_certificate(i:&[u8]) -> IResult<&[u8],DerObject> {
    parse_der_sequence_defined!(i,
        parse_version,
        parse_der_integer, // serialNumber
        parse_algorithm_identifier,
        parse_name, // issuer
        parse_validity,
        parse_name, // subject
        parse_subject_public_key_info,
        parse_issuer_unique_id,
        parse_subject_unique_id,
        parse_extensions,
    )
}

#[inline]
fn der_read_opt_der(i:&[u8]) -> IResult<&[u8],DerObject,u32> {
    parse_der_optional!(i, parse_der)
}

#[inline]
pub fn parse_algorithm_identifier(i:&[u8]) -> IResult<&[u8],DerObject> {
    parse_der_sequence_defined!(i, parse_der_oid, der_read_opt_der)
}

#[inline]
pub fn parse_signature_value(i:&[u8]) -> IResult<&[u8],DerObject> {
    parse_der_bitstring(i)
}

// XXX validate X509 structure
pub fn x509_parser(i:&[u8]) -> IResult<&[u8],X509Certificate> {
    map!(i,
         parse_der_defined!(
             0x10,
             parse_tbs_certificate,
             parse_algorithm_identifier,
             parse_der_bitstring
         ),
         |(_hdr,o)| X509Certificate::new(o)
    )
}






#[cfg(test)]
mod tests {
    //use super::*;
    //use der::*;
    use x509::x509_parser;
    use nom::IResult;

static IGCA_DER: &'static [u8] = &[
  0x30, 0x82, 0x04, 0x02, 0x30, 0x82, 0x02, 0xea, 0xa0, 0x03, 0x02, 0x01,
  0x02, 0x02, 0x05, 0x39, 0x11, 0x45, 0x10, 0x94, 0x30, 0x0d, 0x06, 0x09,
  0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x05, 0x05, 0x00, 0x30,
  0x81, 0x85, 0x31, 0x0b, 0x30, 0x09, 0x06, 0x03, 0x55, 0x04, 0x06, 0x13,
  0x02, 0x46, 0x52, 0x31, 0x0f, 0x30, 0x0d, 0x06, 0x03, 0x55, 0x04, 0x08,
  0x13, 0x06, 0x46, 0x72, 0x61, 0x6e, 0x63, 0x65, 0x31, 0x0e, 0x30, 0x0c,
  0x06, 0x03, 0x55, 0x04, 0x07, 0x13, 0x05, 0x50, 0x61, 0x72, 0x69, 0x73,
  0x31, 0x10, 0x30, 0x0e, 0x06, 0x03, 0x55, 0x04, 0x0a, 0x13, 0x07, 0x50,
  0x4d, 0x2f, 0x53, 0x47, 0x44, 0x4e, 0x31, 0x0e, 0x30, 0x0c, 0x06, 0x03,
  0x55, 0x04, 0x0b, 0x13, 0x05, 0x44, 0x43, 0x53, 0x53, 0x49, 0x31, 0x0e,
  0x30, 0x0c, 0x06, 0x03, 0x55, 0x04, 0x03, 0x13, 0x05, 0x49, 0x47, 0x43,
  0x2f, 0x41, 0x31, 0x23, 0x30, 0x21, 0x06, 0x09, 0x2a, 0x86, 0x48, 0x86,
  0xf7, 0x0d, 0x01, 0x09, 0x01, 0x16, 0x14, 0x69, 0x67, 0x63, 0x61, 0x40,
  0x73, 0x67, 0x64, 0x6e, 0x2e, 0x70, 0x6d, 0x2e, 0x67, 0x6f, 0x75, 0x76,
  0x2e, 0x66, 0x72, 0x30, 0x1e, 0x17, 0x0d, 0x30, 0x32, 0x31, 0x32, 0x31,
  0x33, 0x31, 0x34, 0x32, 0x39, 0x32, 0x33, 0x5a, 0x17, 0x0d, 0x32, 0x30,
  0x31, 0x30, 0x31, 0x37, 0x31, 0x34, 0x32, 0x39, 0x32, 0x32, 0x5a, 0x30,
  0x81, 0x85, 0x31, 0x0b, 0x30, 0x09, 0x06, 0x03, 0x55, 0x04, 0x06, 0x13,
  0x02, 0x46, 0x52, 0x31, 0x0f, 0x30, 0x0d, 0x06, 0x03, 0x55, 0x04, 0x08,
  0x13, 0x06, 0x46, 0x72, 0x61, 0x6e, 0x63, 0x65, 0x31, 0x0e, 0x30, 0x0c,
  0x06, 0x03, 0x55, 0x04, 0x07, 0x13, 0x05, 0x50, 0x61, 0x72, 0x69, 0x73,
  0x31, 0x10, 0x30, 0x0e, 0x06, 0x03, 0x55, 0x04, 0x0a, 0x13, 0x07, 0x50,
  0x4d, 0x2f, 0x53, 0x47, 0x44, 0x4e, 0x31, 0x0e, 0x30, 0x0c, 0x06, 0x03,
  0x55, 0x04, 0x0b, 0x13, 0x05, 0x44, 0x43, 0x53, 0x53, 0x49, 0x31, 0x0e,
  0x30, 0x0c, 0x06, 0x03, 0x55, 0x04, 0x03, 0x13, 0x05, 0x49, 0x47, 0x43,
  0x2f, 0x41, 0x31, 0x23, 0x30, 0x21, 0x06, 0x09, 0x2a, 0x86, 0x48, 0x86,
  0xf7, 0x0d, 0x01, 0x09, 0x01, 0x16, 0x14, 0x69, 0x67, 0x63, 0x61, 0x40,
  0x73, 0x67, 0x64, 0x6e, 0x2e, 0x70, 0x6d, 0x2e, 0x67, 0x6f, 0x75, 0x76,
  0x2e, 0x66, 0x72, 0x30, 0x82, 0x01, 0x22, 0x30, 0x0d, 0x06, 0x09, 0x2a,
  0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x01, 0x05, 0x00, 0x03, 0x82,
  0x01, 0x0f, 0x00, 0x30, 0x82, 0x01, 0x0a, 0x02, 0x82, 0x01, 0x01, 0x00,
  0xb2, 0x1f, 0xd1, 0xd0, 0x62, 0xc5, 0x33, 0x3b, 0xc0, 0x04, 0x86, 0x88,
  0xb3, 0xdc, 0xf8, 0x88, 0xf7, 0xfd, 0xdf, 0x43, 0xdf, 0x7a, 0x8d, 0x9a,
  0x49, 0x5c, 0xf6, 0x4e, 0xaa, 0xcc, 0x1c, 0xb9, 0xa1, 0xeb, 0x27, 0x89,
  0xf2, 0x46, 0xe9, 0x3b, 0x4a, 0x71, 0xd5, 0x1d, 0x8e, 0x2d, 0xcf, 0xe6,
  0xad, 0xab, 0x63, 0x50, 0xc7, 0x54, 0x0b, 0x6e, 0x12, 0xc9, 0x90, 0x36,
  0xc6, 0xd8, 0x2f, 0xda, 0x91, 0xaa, 0x68, 0xc5, 0x72, 0xfe, 0x17, 0x0a,
  0xb2, 0x17, 0x7e, 0x79, 0xb5, 0x32, 0x88, 0x70, 0xca, 0x70, 0xc0, 0x96,
  0x4a, 0x8e, 0xe4, 0x55, 0xcd, 0x1d, 0x27, 0x94, 0xbf, 0xce, 0x72, 0x2a,
  0xec, 0x5c, 0xf9, 0x73, 0x20, 0xfe, 0xbd, 0xf7, 0x2e, 0x89, 0x67, 0xb8,
  0xbb, 0x47, 0x73, 0x12, 0xf7, 0xd1, 0x35, 0x69, 0x3a, 0xf2, 0x0a, 0xb9,
  0xae, 0xff, 0x46, 0x42, 0x46, 0xa2, 0xbf, 0xa1, 0x85, 0x1a, 0xf9, 0xbf,
  0xe4, 0xff, 0x49, 0x85, 0xf7, 0xa3, 0x70, 0x86, 0x32, 0x1c, 0x5d, 0x9f,
  0x60, 0xf7, 0xa9, 0xad, 0xa5, 0xff, 0xcf, 0xd1, 0x34, 0xf9, 0x7d, 0x5b,
  0x17, 0xc6, 0xdc, 0xd6, 0x0e, 0x28, 0x6b, 0xc2, 0xdd, 0xf1, 0xf5, 0x33,
  0x68, 0x9d, 0x4e, 0xfc, 0x87, 0x7c, 0x36, 0x12, 0xd6, 0xa3, 0x80, 0xe8,
  0x43, 0x0d, 0x55, 0x61, 0x94, 0xea, 0x64, 0x37, 0x47, 0xea, 0x77, 0xca,
  0xd0, 0xb2, 0x58, 0x05, 0xc3, 0x5d, 0x7e, 0xb1, 0xa8, 0x46, 0x90, 0x31,
  0x56, 0xce, 0x70, 0x2a, 0x96, 0xb2, 0x30, 0xb8, 0x77, 0xe6, 0x79, 0xc0,
  0xbd, 0x29, 0x3b, 0xfd, 0x94, 0x77, 0x4c, 0xbd, 0x20, 0xcd, 0x41, 0x25,
  0xe0, 0x2e, 0xc7, 0x1b, 0xbb, 0xee, 0xa4, 0x04, 0x41, 0xd2, 0x5d, 0xad,
  0x12, 0x6a, 0x8a, 0x9b, 0x47, 0xfb, 0xc9, 0xdd, 0x46, 0x40, 0xe1, 0x9d,
  0x3c, 0x33, 0xd0, 0xb5, 0x02, 0x03, 0x01, 0x00, 0x01, 0xa3, 0x77, 0x30,
  0x75, 0x30, 0x0f, 0x06, 0x03, 0x55, 0x1d, 0x13, 0x01, 0x01, 0xff, 0x04,
  0x05, 0x30, 0x03, 0x01, 0x01, 0xff, 0x30, 0x0b, 0x06, 0x03, 0x55, 0x1d,
  0x0f, 0x04, 0x04, 0x03, 0x02, 0x01, 0x46, 0x30, 0x15, 0x06, 0x03, 0x55,
  0x1d, 0x20, 0x04, 0x0e, 0x30, 0x0c, 0x30, 0x0a, 0x06, 0x08, 0x2a, 0x81,
  0x7a, 0x01, 0x79, 0x01, 0x01, 0x01, 0x30, 0x1d, 0x06, 0x03, 0x55, 0x1d,
  0x0e, 0x04, 0x16, 0x04, 0x14, 0xa3, 0x05, 0x2f, 0x18, 0x60, 0x50, 0xc2,
  0x89, 0x0a, 0xdd, 0x2b, 0x21, 0x4f, 0xff, 0x8e, 0x4e, 0xa8, 0x30, 0x31,
  0x36, 0x30, 0x1f, 0x06, 0x03, 0x55, 0x1d, 0x23, 0x04, 0x18, 0x30, 0x16,
  0x80, 0x14, 0xa3, 0x05, 0x2f, 0x18, 0x60, 0x50, 0xc2, 0x89, 0x0a, 0xdd,
  0x2b, 0x21, 0x4f, 0xff, 0x8e, 0x4e, 0xa8, 0x30, 0x31, 0x36, 0x30, 0x0d,
  0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x05, 0x05,
  0x00, 0x03, 0x82, 0x01, 0x01, 0x00, 0x05, 0xdc, 0x26, 0xd8, 0xfa, 0x77,
  0x15, 0x44, 0x68, 0xfc, 0x2f, 0x66, 0x3a, 0x74, 0xe0, 0x5d, 0xe4, 0x29,
  0xff, 0x06, 0x07, 0x13, 0x84, 0x4a, 0xab, 0xcf, 0x6d, 0xa0, 0x1f, 0x51,
  0x94, 0xf8, 0x49, 0xcb, 0x74, 0x36, 0x14, 0xbc, 0x15, 0xdd, 0xdb, 0x89,
  0x2f, 0xdd, 0x8f, 0xa0, 0x5d, 0x7c, 0xf5, 0x12, 0xeb, 0x9f, 0x9e, 0x38,
  0xa4, 0x47, 0xcc, 0xb3, 0x96, 0xd9, 0xbe, 0x9c, 0x25, 0xab, 0x03, 0x7e,
  0x33, 0x0f, 0x95, 0x81, 0x0d, 0xfd, 0x16, 0xe0, 0x88, 0xbe, 0x37, 0xf0,
  0x6c, 0x5d, 0xd0, 0x31, 0x9b, 0x32, 0x2b, 0x5d, 0x17, 0x65, 0x93, 0x98,
  0x60, 0xbc, 0x6e, 0x8f, 0xb1, 0xa8, 0x3c, 0x1e, 0xd9, 0x1c, 0xf3, 0xa9,
  0x26, 0x42, 0xf9, 0x64, 0x1d, 0xc2, 0xe7, 0x92, 0xf6, 0xf4, 0x1e, 0x5a,
  0xaa, 0x19, 0x52, 0x5d, 0xaf, 0xe8, 0xa2, 0xf7, 0x60, 0xa0, 0xf6, 0x8d,
  0xf0, 0x89, 0xf5, 0x6e, 0xe0, 0x0a, 0x05, 0x01, 0x95, 0xc9, 0x8b, 0x20,
  0x0a, 0xba, 0x5a, 0xfc, 0x9a, 0x2c, 0x3c, 0xbd, 0xc3, 0xb7, 0xc9, 0x5d,
  0x78, 0x25, 0x05, 0x3f, 0x56, 0x14, 0x9b, 0x0c, 0xda, 0xfb, 0x3a, 0x48,
  0xfe, 0x97, 0x69, 0x5e, 0xca, 0x10, 0x86, 0xf7, 0x4e, 0x96, 0x04, 0x08,
  0x4d, 0xec, 0xb0, 0xbe, 0x5d, 0xdc, 0x3b, 0x8e, 0x4f, 0xc1, 0xfd, 0x9a,
  0x36, 0x34, 0x9a, 0x4c, 0x54, 0x7e, 0x17, 0x03, 0x48, 0x95, 0x08, 0x11,
  0x1c, 0x07, 0x6f, 0x85, 0x08, 0x7e, 0x5d, 0x4d, 0xc4, 0x9d, 0xdb, 0xfb,
  0xae, 0xce, 0xb2, 0xd1, 0xb3, 0xb8, 0x83, 0x6c, 0x1d, 0xb2, 0xb3, 0x79,
  0xf1, 0xd8, 0x70, 0x99, 0x7e, 0xf0, 0x13, 0x02, 0xce, 0x5e, 0xdd, 0x51,
  0xd3, 0xdf, 0x36, 0x81, 0xa1, 0x1b, 0x78, 0x2f, 0x71, 0xb3, 0xf1, 0x59,
  0x4c, 0x46, 0x18, 0x28, 0xab, 0x85, 0xd2, 0x60, 0x56, 0x5a
];

#[test]
fn test_x509_parser() {
    let empty = &b""[..];
    //assert_eq!(x509_parser(IGCA_DER), IResult::Done(empty, (1)));
    let res = x509_parser(IGCA_DER);
    match res {
        IResult::Done(e, cert) => {
            assert_eq!(e,empty);
            println!("tbsCertificate: {:?}", cert.tbsCertificate.as_pretty(0,2));
            println!("signatureAlgorithm: {:?}", cert.signatureAlgorithm.as_pretty(0,2));
        },
        _ => panic!("x509 parsing failed: {:?}", res),
    }
}

}
