use cfg_if::cfg_if;
pub mod app;
pub mod app_error;
pub mod game_logic;
pub mod game_settings;
pub mod pages;
pub mod utils;

cfg_if! { if #[cfg(feature = "hydrate")] {
    use leptos::*;
    use wasm_bindgen::prelude::wasm_bindgen;
    use crate::app::*;

    // 定义一个暴露给JavaScript的函数hydrate
    #[wasm_bindgen]
    pub fn hydrate() {
        // 使用log crate初始化日志记录
        _ = console_log::init_with_level(log::Level::Debug);
        // 设置一次性panic钩子，在发生panic时记录错误
        console_error_panic_hook::set_once();

        // 将Leptos应用挂载到HTML文档的body部分
        leptos::mount_to_body(App);
    }
}}
