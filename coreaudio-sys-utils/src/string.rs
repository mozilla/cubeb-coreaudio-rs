use coreaudio_sys::*;

pub fn cfstringref_from_static_string(string: &'static str) -> CFStringRef {
    // References:
    // https://developer.apple.com/documentation/corefoundation/1543597-cfstringcreatewithbytesnocopy?language=objc
    // https://github.com/opensource-apple/CF/blob/3cc41a76b1491f50813e28a4ec09954ffa359e6f/CFString.c#L1605
    // https://github.com/servo/core-foundation-rs/blob/2aac8fb85b5b114673280e273c04219c0c360e54/core-foundation/src/string.rs#L125
    // https://github.com/servo/core-foundation-rs/blob/2aac8fb85b5b114673280e273c04219c0c360e54/io-surface/src/lib.rs#L48
    // Set deallocator to kCFAllocatorNull to prevent the the memory of the
    // parameter `string` from being released by CFRelease.
    // We manage the string memory by ourselves.
    unsafe {
        CFStringCreateWithBytesNoCopy(
            kCFAllocatorDefault,
            string.as_ptr(),
            string.len() as CFIndex,
            kCFStringEncodingUTF8,
            false as Boolean,
            kCFAllocatorNull,
        )
    }
}

pub fn cfstringref_from_string(string: &str) -> CFStringRef {
    // References:
    // https://developer.apple.com/documentation/corefoundation/1543419-cfstringcreatewithbytes?language=objc
    // https://github.com/opensource-apple/CF/blob/3cc41a76b1491f50813e28a4ec09954ffa359e6f/CFString.c#L1597
    // https://github.com/servo/core-foundation-rs/blob/2aac8fb85b5b114673280e273c04219c0c360e54/core-foundation/src/string.rs#L111
    // https://github.com/servo/core-foundation-rs/blob/2aac8fb85b5b114673280e273c04219c0c360e54/io-surface/src/lib.rs#L48
    unsafe {
        CFStringCreateWithBytes(
            kCFAllocatorDefault,
            string.as_ptr(),
            string.len() as CFIndex,
            kCFStringEncodingUTF8,
            false as Boolean,
        )
    }
}

#[test]
fn test_create_static_cfstring_ref() {
    use super::*;

    let cfstrref = cfstringref_from_static_string(PRIVATE_AGGREGATE_DEVICE_NAME);
    let cstring = audiounit_strref_to_cstr_utf8(cfstrref);
    unsafe {
        CFRelease(cfstrref as *const c_void);
    }

    assert_eq!(
        PRIVATE_AGGREGATE_DEVICE_NAME,
        cstring.into_string().unwrap()
    );

    // TODO: Find a way to check the string's inner pointer is same.
}

#[test]
fn test_create_cfstring_ref() {
    use super::*;

    let test_string = "Rustaceans 🦀";
    let cfstrref = cfstringref_from_string(test_string);
    let cstring = audiounit_strref_to_cstr_utf8(cfstrref);
    unsafe {
        CFRelease(cfstrref as *const c_void);
    }

    assert_eq!(test_string, cstring.to_string_lossy());

    // TODO: Find a way to check the string's inner pointer is different.
}
