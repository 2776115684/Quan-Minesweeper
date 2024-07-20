use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use cfg_if::cfg_if;
use rand::seq::SliceRandom;
use serde::de::IntoDeserializer;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// 主题枚举
#[derive(Copy, Clone, Default, Debug)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

impl Theme {
    // 切换主题
    pub fn toggle(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }
}

// 实现Display trait用于格式化输出
impl Display for Theme {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Dark => "dark",
                Self::Light => "light",
            }
        )
    }
}

// 实现FromStr trait用于从字符串解析
impl FromStr for Theme {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dark" => Ok(Self::Dark),
            "light" => Ok(Self::Light),
            _ => Err(()),
        }
    }
}

// 用户名结构体
#[derive(Clone, Debug)]
pub struct Username {
    pub name: String,
    pub stable: bool,
}

// 随机生成用户名的函数
fn random_name() -> &'static str {
    // 在项目的根目录下有用于随机生成用户名的名为 names 的JSON文件
    include!("../names.json")
        .choose(&mut rand::thread_rng())
        .expect("array is nonempty")
}

impl Username {
    // 创建新的用户名
    pub fn new(name: String) -> Self {
        Self { name, stable: true }
    }

    // 创建随机用户名
    pub fn random() -> Self {
        Self {
            name: random_name().into(),
            stable: true,
        }
    }
}

// 实现从Option<String>转换为Username
impl From<Option<String>> for Username {
    fn from(value: Option<String>) -> Self {
        if let Some(name) = value {
            Self { name, stable: true }
        } else {
            Self {
                name: random_name().into(),
                stable: false,
            }
        }
    }
}

// 解析难度错误类型
#[derive(Error, Debug)]
pub struct ParseDifficultyError;

impl Display for ParseDifficultyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Error parsing game difficulty")
    }
}

// 游戏难度枚举
#[derive(PartialEq, Copy, Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    #[default]
    Easy,
    Normal,
    Hard,
}

// 实现从字符串解析Difficulty
impl FromStr for Difficulty {
    type Err = serde::de::value::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::deserialize(s.into_deserializer())
    }
}

// 实现Display trait用于格式化输出难度Difficulty
impl Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.serialize(f)
    }
}

// 解析大小size错误类型
#[derive(Error, Debug)]
pub struct ParseSizeError;

impl Display for ParseSizeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Error parsing gameboard size")
    }
}

// 扫雷游戏板大小枚举
#[derive(PartialEq, Copy, Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Size {
    #[default]
    Small,
    Medium,
    Large,
}

// 实现从字符串解析Size
impl FromStr for Size {
    type Err = serde::de::value::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::deserialize(s.into_deserializer())
    }
}

// 实现Display trait用于格式化输出大小Size
impl Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.serialize(f)
    }
}

// 使用cfg_if宏，根据不同的编译环境选择不同的代码路径
cfg_if! {
    if #[cfg(feature = "ssr")] {

        // 服务器端渲染时获取设置
        pub fn fetch_setting<T: FromStr + Default>(setting: &str) -> Option<T> {
            leptos::use_context().and_then(|leptos_axum::RequestParts { headers, ..}| {
                let jar = axum_extra::extract::CookieJar::from_headers(&headers);
                jar.get(setting).and_then(|cookie| cookie.value().parse().ok())
            })
        }

        // 服务器端渲染时应用设置
        pub fn apply_setting<T: ToString>(_setting: &str, _value: &T) {
            unimplemented!()
        }

    } else if #[cfg(target_arch = "wasm32")] {

        // WebAssembly环境下获取设置
        pub fn fetch_setting<T: FromStr>(setting: &str) -> Option<T> {
            Some(wasm_cookies::get(setting)?.ok()?.parse().ok()?)
        }

        // WebAssembly环境下应用设置
        pub fn apply_setting<T: ToString>(setting: &str, value: &T) {
            wasm_cookies::set(
                setting,
                &value.to_string(),
                &wasm_cookies::CookieOptions::default()
                    .expires_after(chrono::Duration::weeks(999).to_std().expect("convert to std duration")));
        }

    } else {

        // 其他环境下的存根函数, 实际情况下不调用
        pub fn fetch_setting<T: FromStr + Default>(_setting: &str) -> Option<T> {
            Default::default()
        }

        pub fn apply_setting<T: ToString>(_setting: &str, _value: &T) {
            unimplemented!()
        }
    }
}
