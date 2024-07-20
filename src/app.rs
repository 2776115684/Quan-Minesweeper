use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::app_error::AppError;
use crate::game_settings::{apply_setting, fetch_setting, Theme, Username};
use crate::pages::{Error, Game, HomePage, Scores};

// 定义两个常量，分别包含浅色和深色模式的SVG图标
const LIGHTBULB_SVG: &str = include_str!("../svgs/lightbulb.svg"); // 浅色模式图标
const MOON_SVG: &str = include_str!("../svgs/moon.svg"); // 深色模式图标

// 定义App组件
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context(); // 提供Meta上下文

    // 获取主题设置，如果未设置则使用默认值
    let theme_setting = fetch_setting::<Theme>("theme");
    let (theme, set_theme) = create_signal(theme_setting.unwrap_or_default());
    if theme_setting.is_none() {
        // 如果未设置主题，根据系统偏好设置主题
        Effect::new(move |_| {
            if let Ok(Some(mql)) = leptos::window().match_media("(prefers-color-scheme: dark)") {
                if mql.matches() {
                    set_theme(Theme::Dark);
                }
            }
        });
    }

    // 获取用户名并设置信号
    let (username, set_username) = create_signal(Username::from(fetch_setting("username")));
    provide_context(username); // 提供用户名上下文
    provide_context(set_username); // 提供设置用户名上下文

    // 返回视图
    view! {
        // 引用样式表
        <Stylesheet id="leptos" href="/pkg/tailwind.css" />
        <Stylesheet id="leptos" href="/pkg/Quan-Minesweeper.css" />

        // 设置网页标题
        <Title text="Quan-Minesweeper" />

        <Html class=move || theme().to_string() />

         // 配置路由和路由处理器
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <Error outside_errors />
            }
            .into_view()
        }>
            <main>
                <div class="text-4xl my-5 mx-auto font-bold">"Quan-Minesweeper"</div>
                // 主题切换按钮
                <button
                    class="theme-toggle"

                    on:click=move |_| {
                        let new_theme = theme().toggle();
                        set_theme(new_theme);
                        apply_setting("theme", &new_theme);
                    }

                    inner_html=move || {
                        match theme() {
                            Theme::Light => MOON_SVG,
                            Theme::Dark => LIGHTBULB_SVG,
                        }
                    }
                />
                // 配置路由
                <Routes>
                    <Route path="" view=HomePage />
                    <Route path="game" view=Game />
                    <Route path="scores" view=Scores />
                </Routes>
            </main>
        </Router>
    }
}
