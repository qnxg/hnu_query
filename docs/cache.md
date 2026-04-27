# 缓存

本库的很多数据都是值得缓存的。如果你使用了类似 Redis 之类的中间件，那么可能需要将需要存储的内容序列化与反序列化。

库中大多数函数的返回内容都实现了 `Serialize` 和 `Deserialize` 这两个 Trait，你可以使用如 `serde_json` 这样的序列化库将返回的内容序列化/反序列化以进行缓存。

对于 Token，我们提供了形如 `from_xxx_unchecked` 的函数，可以让你直接构造 Token。如果你需要缓存 Token，只需要将 `from_xxx_unchecked` 需要的参数序列化/反序列化即可，这些参数都保证可以通过对应的 Token 上的公开的函数来获得。

很多 Token 的 `from_xxx_unchecked` 需要接受一个 `reqwest::header::HeaderMap`，而 `HeaderMap` 并没有实现 `Serialize` 和 `Deserialize`。这意味着如果你想要对 `HeaderMap` 进行序列化与反序列化的话有点麻烦。我们在湖大微生活的后端中，采用了如下的方式进行序列化：

```rust
use std::{collections::HashMap, str::FromStr};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize, ser::SerializeMap};

pub struct SerializableHeaderMap(HeaderMap);

impl SerializableHeaderMap {
    pub fn new(header_map: HeaderMap) -> Self {
        Self(header_map)
    }
    pub fn into_inner(self) -> HeaderMap {
        self.0
    }
}

impl Serialize for SerializableHeaderMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in self.0.iter() {
            let key = key.as_str();
            let value = value.to_str().map_err(|e| {
                serde::ser::Error::custom(format!("{:?}", e))
            })?;
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for SerializableHeaderMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map: HashMap<String, String> =
            HashMap::deserialize(deserializer)?;
        let mut header_map = HeaderMap::new();
        for (key, value) in map {
            let key = HeaderName::from_str(&key).map_err(|e| {
                serde::de::Error::custom(format!("{:?}", e))
            })?;
            let value =
                HeaderValue::from_str(&value).map_err(|e| {
                    serde::de::Error::custom(format!("{:?}", e))
                })?;
            header_map.insert(key, value);
        }
        Ok(SerializableHeaderMap(header_map))
    }
}
```
