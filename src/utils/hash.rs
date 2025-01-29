use std::fmt::Display;

use clap::ValueEnum;
use digest::Digest;
use md5::Md5;
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use sha3::{Sha3_256, Sha3_512};

#[derive(Debug, ValueEnum, Clone, PartialEq)]
pub enum HashAlgorithm {
    Md5,
    Sha1,
    Sha2_256,
    Sha2_512,
    Sha3_256,
    Sha3_512,
}

impl Display for HashAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashAlgorithm::Md5 => write!(f, "MD5"),
            HashAlgorithm::Sha1 => write!(f, "SHA1"),
            HashAlgorithm::Sha2_256 => write!(f, "SHA2-256"),
            HashAlgorithm::Sha2_512 => write!(f, "SHA2-512"),
            HashAlgorithm::Sha3_256 => write!(f, "SHA3-256"),
            HashAlgorithm::Sha3_512 => write!(f, "SHA3-512"),
        }
    }
}

impl HashAlgorithm {
    pub fn compute(&self, buffer: impl AsRef<[u8]>) -> Vec<u8> {
        match self {
            HashAlgorithm::Md5 => Self::compute_hash::<Md5>(buffer),
            HashAlgorithm::Sha1 => Self::compute_hash::<Sha1>(buffer),
            HashAlgorithm::Sha2_256 => Self::compute_hash::<Sha256>(buffer),
            HashAlgorithm::Sha2_512 => Self::compute_hash::<Sha512>(buffer),
            HashAlgorithm::Sha3_256 => Self::compute_hash::<Sha3_256>(buffer),
            HashAlgorithm::Sha3_512 => Self::compute_hash::<Sha3_512>(buffer),
        }
    }

    fn compute_hash<D: Digest>(buffer: impl AsRef<[u8]>) -> Vec<u8> {
        let mut hasher = D::new();
        hasher.update(&buffer);
        hasher.finalize().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex::encode;
    use rstest::rstest;

    #[rstest]
    #[case(
        "Hello, World!",
        "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
    )]
    #[case("", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")]
    #[should_panic(expected = "Invalid hash value")]
    #[case::panic("", "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f")]
    fn test_calculate_hash_sha2_256(#[case] input: &'static str, #[case] expected: &'static str) {
        let algorithm = HashAlgorithm::Sha2_256;
        let hash = encode(algorithm.compute(input.as_bytes()));

        assert_eq!(hash, expected, "Invalid hash value");
    }
}
