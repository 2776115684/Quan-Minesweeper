use std::{ops::RangeInclusive, rc::Rc};

use gloo_timers::future::TimeoutFuture;
use leptos::*;
use leptos_router::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlFormElement;

use crate::{
    game_settings::{apply_setting, fetch_setting, Difficulty, Size, Username},
    utils::to_title,
};

const USERNAME_BOUNDS: RangeInclusive<usize> = 3..=10; // 用户名长度范围
const DICE_SVG: &str = include_str!("../../svgs/dice.svg"); // 骰子SVG图标

// 验证用户名字符是否合法
fn valid_chars(username: &str) -> bool {
    username
        .chars()
        .all(|c| c.is_ascii_alphabetic() || c == '_')
}

// 渲染主页
#[component]
pub fn HomePage() -> impl IntoView {
    // 从上下文中获取用户名的读取和写入信号
    let (username, set_username) = (
        expect_context::<ReadSignal<Username>>(),
        expect_context::<WriteSignal<Username>>(),
    );

    // 创建难度和大小的信号，初始值从设置中获取，如果没有设置则使用默认值
    let (difficulty, set_difficulty) =
        create_signal(fetch_setting::<Difficulty>("difficulty").unwrap_or_default());
    let (size, set_size) = create_signal(fetch_setting::<Size>("size").unwrap_or_default());
    let (form_action, set_form_action) = create_signal("/");

    // 创建HTML元素的引用，用于后续访问DOM元素
    let username_ref = create_node_ref::<html::Input>();
    let error_ref = create_node_ref::<html::Span>();
    let difficulty_ref = create_node_ref::<html::Select>();
    let size_ref = create_node_ref::<html::Select>();

    let username_error_action = create_action(move |&()| async move {
        let username_input = username_ref.get().expect("noderef assigned");
        let username_input = username_input.prop(
            "style",
            "
            border-color: red;
        ",
        );
        let error_span = error_ref.get().expect("noderef assigned");
        let error_span = error_span.prop(
            "style",
            "
            visibility: visible;
            opacity: 1;
            transition: opacity .2s linear;
        ",
        );
        TimeoutFuture::new(500).await;
        let _ = username_input.prop("style", "");
        TimeoutFuture::new(2000).await;
        let _ = error_span.prop(
            "style",
            "
            visibility: hidden;
            opacity: 0;
            transition: visibility 0s .2s, opacity .2s linear;
        ",
        );
    });

    // 用户名输入事件处理函数
    let on_username_input = move |ev| {
        let new_name = event_target_value(&ev);
        // 验证新用户名的长度和字符是否合法
        if new_name.len() <= *USERNAME_BOUNDS.end() && valid_chars(&new_name) {
            set_username(Username::new(new_name));
        } else {
            set_username(username());
            username_error_action.dispatch(());
        }
    };

    // 表单提交事件处理函数
    let on_settings_submit = move |ev: ev::SubmitEvent| {
        let Username { name, stable } = username();
        // 验证用户名的长度和字符是否合法
        if USERNAME_BOUNDS.contains(&name.len()) && valid_chars(&name) {
            if stable {
                apply_setting("username", &name);
            }
        } else {
            ev.prevent_default();
            username_error_action.dispatch(());
            return;
        }

        // 获取并验证难度选择
        let difficulty_select = difficulty_ref.get().expect("noderef assigned");
        if let Ok(selected_difficulty) = difficulty_select.value().parse() {
            if difficulty() != selected_difficulty {
                apply_setting("difficulty", &selected_difficulty);
                set_difficulty(selected_difficulty);
            }
        } else {
            ev.prevent_default();
            return;
        }

        // 获取并验证大小选择
        let size_select = size_ref.get().expect("noderef assigned");
        if let Ok(selected_size) = size_select.value().parse() {
            if size() != selected_size {
                apply_setting("size", &selected_size);
                set_size(selected_size);
            }
        } else {
            ev.prevent_default();
            return;
        }
        ev.target()
            .unwrap()
            .dyn_into::<HtmlFormElement>()
            .unwrap()
            .set_action(form_action());
    };

    // 生成视图
    view! {
        // 表单元素, 包含设置输入和提交按钮
        <Form
            method="GET"
            action="/"
            on:submit=on_settings_submit
            on_form_data=Rc::new(move |form_data| {
                form_data.delete("username"); //don't need this in the query
            })
        >
            // 设置面板
            <div class="panel">
                <div class="panel-label">"Settings"</div>
                <table class="panel-table">
                    // 用户名行
                    <tr class="panel-row">
                        <td class="panel-row-label">
                            <label for="username">"Name:"</label>
                        </td>
                        <td>
                            // 用户名输入框
                            <input
                                type="text"
                                name="username"
                                prop:value=move || username().name
                                size="12"
                                node_ref=username_ref
                                on:input=on_username_input
                            />
                            // 随机用户名按钮
                            <span
                                class="random-name"
                                on:click=move |_| set_username(Username::random())
                                inner_html=DICE_SVG
                            />
                            // 用户名错误提示容器
                            <div class="username-error-container">
                                <span class="username-error" node_ref=error_ref>
                                    "Name must be 3-10 alphanumeric characters and underscores"
                                </span>
                            </div>
                        </td>
                    </tr>

                    // 难度选择行
                    <tr class="panel-row">
                        <td class="panel-row-label">
                            <label for="difficulty">"Difficulty:"</label>
                        </td>
                        <td>
                            // 难度选择框
                            <select name="difficulty" node_ref=difficulty_ref>
                            {
                                // 生成难度选项
                                [
                                    Difficulty::Easy,
                                    Difficulty::Normal,
                                    Difficulty::Hard,
                                ].iter().map(|curr_difficulty| {
                                    view! {
                                        <option
                                            value=curr_difficulty.to_string()
                                            selected=move || difficulty() == *curr_difficulty
                                        >
                                        {to_title(&curr_difficulty)}
                                        </option>
                                    }
                                }).collect_view()
                            }
                            </select>
                        </td>
                    </tr>

                    // 大小选择行
                    <tr class="panel-row">
                        <td class="panel-row-label">
                            <label for="size">"Board Size:"</label>
                        </td>
                        <td>
                            // 大小选择框
                            <select name="size" node_ref=size_ref>
                            {
                                // 生成大小选项
                                [
                                    Size::Small,
                                    Size::Medium,
                                    Size::Large,
                                ].iter().map(|curr_size| {
                                    view! {
                                        <option
                                            value=curr_size.to_string()
                                            selected=move || size() == *curr_size
                                        >
                                        {to_title(&curr_size)}
                                        </option>
                                    }
                                }).collect_view()
                            }
                            </select>
                        </td>
                    </tr>
                </table>
            </div>

            // 提交按钮
            <div class="btns">
                // 新游戏按钮
                <div class="btn">
                    <input
                        type="submit"
                        value="New Game"
                        on:click=move |_| set_form_action("/game")
                    />
                </div>
                // 查看排行榜按钮
                <div class="btn">
                    <input
                        type="submit"
                        value="Scores"
                        on:click=move |_| set_form_action("/scores")
                    />
                </div>
            </div>
        </Form>
    }
}
