use cfg_if::cfg_if;
use leptos::*;
use leptos_router::A;

use crate::app_error::AppError;

#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;

// 一个用于显示错误的基本函数, 可以在此基础上做更复杂的处理
#[component]
pub fn Error(
    #[prop(optional)] outside_errors: Option<Errors>, // 可选的外部错误
    #[prop(optional)] errors: Option<RwSignal<Errors>>, // 可选的信号错误
) -> impl IntoView {
    // 根据传入的错误参数获取错误信号
    let errors = match outside_errors {
        Some(e) => RwSignal::new(e),
        None => match errors {
            Some(e) => e,
            None => panic!("No Errors found and we expected errors!"),
        },
    };
    // 从信号中获取错误列表
    let errors = errors.get_untracked();

    // Downcast 将错误类型转换为具体的 AppError 类型
    let errors: Vec<AppError> = errors
        .into_iter()
        .filter_map(|(_k, v)| v.downcast_ref::<AppError>().cloned())
        .collect();
    println!("Errors: {errors:#?}");

    // 仅发送第一个错误的响应码, 可以根据具体应用进行定制
    cfg_if! { if #[cfg(feature="ssr")] {
        let response = use_context::<ResponseOptions>();
        if let Some(response) = response {
            response.set_status(errors[0].status_code());
        }
    }}

    // 生成错误视图
    view! {
        <h1>{if errors.len() > 1 {"Errors"} else {"Error"}}</h1>
        <For
            // 返回迭代项的函数
            each= move || {errors.clone().into_iter().enumerate()}
            // 每个项都有一个唯一键
            key=|(index, _error)| *index
            // 将每个项渲染为视图
            children= move |error| {
                let error_string = error.1.to_string();
                let error_code = error.1.status_code();
                view! {
                    <h2>{error_code.to_string()}</h2>
                    <p>{error_string}</p>
                }
            }
        />
        // 返回主页按钮
        <div class="buttons">
            <div class="button-item">
                <A href="/">"Return"</A>
            </div>
        </div>
    }
}
