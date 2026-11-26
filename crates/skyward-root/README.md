# skyward-root

写 Root 用的静态数据：KernelSU 的 8 个 KMI、每个 KMI 该进 `boot` 还是
`init_boot`、以及「按手机的 Android 版本与内核版本挑哪个 KMI」。

`ksud`、对应 KMI 的 `.ko`、两个管理器的 APK 都不在包里，写 Root 时才从
GitHub 取，所以这个 crate 没有任何依赖，也不进 ramdisk 的体积。

## 用法

```rust
let kmi = skyward_root::default(info.android.as_deref(), &info.kernel);
let target = kmi.map(|k| k.target.name()).unwrap_or("init_boot");
```

`default()` 的取舍顺序：Android 版本与内核都对得上 → 同一内核下不比
手机新的最新一版 → 该内核的任意一版。都不中时返回 `None`，由页面让用户自己选。
