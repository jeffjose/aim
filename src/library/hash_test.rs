use hash::petname;
use hash::sha256;
use hash::sha256_short;

use super::*;

#[test]
fn test_sha256() {
    // Test empty string
    assert_eq!(
        sha256(""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );

    // Test basic string
    assert_eq!(
        sha256("hello"),
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );

    // Test with spaces
    assert_eq!(
        sha256("hello world"),
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );

    // Test with special characters
    assert_eq!(
        sha256("!@#$%^&*()"),
        "95ce789c5c9d18490972709838ca3a9719094bca3ac16332cfec0652b0236141"
    );
}

#[test]
fn test_sha256_short() {
    // Test empty string (first 12 chars)
    assert_eq!(sha256_short(""), "e3b0c44298fc");

    // Test basic string
    assert_eq!(sha256_short("hello"), "2cf24dba5fb0");

    // Test with spaces
    assert_eq!(sha256_short("hello world"), "b94d27b9934d");

    // Verify length is always 12
    assert_eq!(sha256_short("very long string").len(), 12);
}

#[test]
fn test_petname() {
    // Test deterministic output for same input
    let name1 = petname("test-input");
    let name2 = petname("test-input");
    assert_eq!(name1, name2);

    // Test different inputs produce different names
    let name3 = petname("different-input");
    assert_ne!(name1, name3);

    // Test format (two words separated by hyphen)
    let name = petname("test");
    assert!(name.contains('-'));
    assert_eq!(name.matches('-').count(), 1);
    assert!(!name.contains(' '));

    // Test empty string input
    let empty_name = petname("");
    assert!(empty_name.contains('-'));
    assert_ne!(empty_name, "-");
}

#[test]
fn test_petname_consistency() {
    // Test that specific inputs always generate the same petnames
    let test_cases = [
        ("test1", "joint-mynah"),
        ("test2", "loving-chow"),
        ("hello", "intrigued-trout"),
        ("world", "prophetic-brocket"),
    ];

    for (input, expected) in test_cases {
        assert_eq!(petname(input), expected);
    }
}

#[test]
fn test_hash_and_petname_combination() {
    // Test that hashing and then using as petname seed is consistent
    let input = "test string";
    let hash = sha256_short(input);
    let name1 = petname(&hash);
    let name2 = petname(&hash);

    assert_eq!(name1, name2);
}
