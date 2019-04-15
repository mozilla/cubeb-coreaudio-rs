// A compile-time static string mapped to kAudioAggregateDeviceNameKey
// https://github.com/phracker/MacOSX-SDKs/blob/9fc3ed0ad0345950ac25c28695b0427846eea966/MacOSX10.12.sdk/System/Library/Frameworks/CoreAudio.framework/Versions/A/Headers/AudioHardware.h#L1513
pub const AGGREGATE_DEVICE_NAME_KEY: &str = "name";

// A compile-time static string mapped to kAudioAggregateDeviceUIDKey
// https://github.com/phracker/MacOSX-SDKs/blob/9fc3ed0ad0345950ac25c28695b0427846eea966/MacOSX10.12.sdk/System/Library/Frameworks/CoreAudio.framework/Versions/A/Headers/AudioHardware.h#L1505
pub const AGGREGATE_DEVICE_UID_KEY: &str = "uid";

// A compile-time static string mapped to kAudioAggregateDeviceIsPrivateKey
// https://github.com/phracker/MacOSX-SDKs/blob/9fc3ed0ad0345950ac25c28695b0427846eea966/MacOSX10.12.sdk/System/Library/Frameworks/CoreAudio.framework/Versions/A/Headers/AudioHardware.h#L1553
pub const AGGREGATE_DEVICE_PRIVATE_KEY: &str = "private";

// A compile-time static string mapped to kAudioAggregateDeviceIsStackedKey
// https://github.com/phracker/MacOSX-SDKs/blob/9fc3ed0ad0345950ac25c28695b0427846eea966/MacOSX10.12.sdk/System/Library/Frameworks/CoreAudio.framework/Versions/A/Headers/AudioHardware.h#L1562
pub const AGGREGATE_DEVICE_STACKED_KEY: &str = "stacked";

// A compile-time static string mapped to  kAudioAggregateDeviceSubDeviceListKey
// https://github.com/phracker/MacOSX-SDKs/blob/9fc3ed0ad0345950ac25c28695b0427846eea966/MacOSX10.12.sdk/System/Library/Frameworks/CoreAudio.framework/Versions/A/Headers/AudioHardware.h#L1522
pub const AGGREGATE_DEVICE_SUB_DEVICE_LIST_KEY: &str = "subdevices";

// A compile-time static string mapped to kAudioSubDeviceUIDKey
// https://github.com/phracker/MacOSX-SDKs/blob/9fc3ed0ad0345950ac25c28695b0427846eea966/MacOSX10.12.sdk/System/Library/Frameworks/CoreAudio.framework/Versions/A/Headers/AudioHardware.h#L1645
pub const SUB_DEVICE_UID_KEY: &str = "uid";
