use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use crate::{
    app_error::AppError,
    game_settings::{Difficulty, Size},
    pages::Error,
    utils::{to_time, to_title},
};

// 排行榜只显示前10名
const MAX_SCORES: usize = 10;

// 得分结构体
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Score {
    username: String,
    time_in_seconds: i64,
}

// 获取得分
#[server(GetScores)]
async fn get_scores(difficulty: Difficulty, size: Size) -> Result<Vec<Score>, ServerFnError> {
    let pool = expect_context::<sqlx::SqlitePool>(); // 获取数据库连接池上下文
    let (difficulty, size) = (difficulty.to_string(), size.to_string());

    // 查询数据库
    sqlx::query_as!(
        Score,
        "
            SELECT username, time_in_seconds
            FROM scores
            WHERE difficulty=?
                AND size=?
            ORDER BY time_in_seconds
            LIMIT ?
        ",
        difficulty,
        size,
        MAX_SCORES as i64
    )
    .fetch_all(&pool)
    .await
    .map_err(Into::into)
}

// 提交得分
#[server(PostScore)]
pub async fn post_score(
    username: String,
    time_in_seconds: i64,
    difficulty: Difficulty,
    size: Size,
) -> Result<(), ServerFnError> {
    let pool = expect_context::<sqlx::SqlitePool>(); // 获取数据库连接池上下文
    let (difficulty, size) = (difficulty.to_string(), size.to_string());

    // 向数据库中插入数据
    sqlx::query_as!(
        Score,
        "
            INSERT INTO scores(username, time_in_seconds, difficulty, size)
            VALUES (?, ?, ?, ?)
        ",
        username,
        time_in_seconds,
        difficulty,
        size,
    )
    .execute(&pool)
    .await
    .map(|_| ())
    .map_err(Into::into)
}

// 显示排行榜的组件
#[component]
pub fn Scores() -> impl IntoView {
    let (difficulty, set_difficulty) = create_query_signal::<Difficulty>("difficulty");
    let (size, set_size) = create_query_signal::<Size>("size");
    provide_context((difficulty, size));
    provide_context((set_difficulty, set_size));

    match (difficulty.get_untracked(), size.get_untracked()) {
        (Some(difficulty), Some(size)) => view! {
            <ScoreFilters difficulty size /> // 过滤器组件(可根据难度/尺寸过滤排行榜)

            <Scoreboard /> // 排行榜组件

            <div class="btns">
                <div class="btn">
                    <A href="/">
                        "Return"
                    </A>
                </div>
            </div>
        }
        .into_view(),

        _ => {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);

            view! {
                <Error outside_errors />
            }
            .into_view()
        }
    }
}

// 过滤器组件
#[component]
fn ScoreFilters(difficulty: Difficulty, size: Size) -> impl IntoView {
    let (set_difficulty, set_size) =
        expect_context::<(SignalSetter<Option<Difficulty>>, SignalSetter<Option<Size>>)>();

    view! {
        <div class="panel">
            <div class="panel-label">
                "Filters"
            </div>
            <table class="panel-table">
                <tr class="panel-row">
                    <td>
                        <select on:change=move |ev| {
                            set_difficulty(Some(event_target_value(&ev).parse().expect("value is a difficulty")));
                        }>
                        {
                            [
                                Difficulty::Easy,
                                Difficulty::Normal,
                                Difficulty::Hard,
                            ].iter().map(|curr_difficulty| {
                                view! {
                                    <option
                                        value=curr_difficulty.to_string()
                                        selected=move || difficulty == *curr_difficulty
                                        on:click=move |_| set_difficulty(Some(*curr_difficulty))
                                    >
                                    {to_title(&curr_difficulty)}
                                    </option>
                                }
                            }).collect_view()
                        }
                        </select>
                    </td>
                    <td>
                        <select on:change=move |ev| {
                            set_size(Some(event_target_value(&ev).parse().expect("value is a size")));
                        }>
                        {
                            [
                                Size::Small,
                                Size::Medium,
                                Size::Large,
                            ].iter().map(|curr_size| {
                                view! {
                                    <option
                                        value=curr_size.to_string()
                                        selected=move || size == *curr_size
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
    }
}

// 排行榜组件
#[component]
fn Scoreboard() -> impl IntoView {
    let (difficulty, size) = expect_context::<(Memo<Option<Difficulty>>, Memo<Option<Size>>)>();
    let filters = move || (difficulty().unwrap_or_default(), size().unwrap_or_default());
    let score_getter = create_resource(filters, |(difficulty, size)| async move {
        get_scores(difficulty, size).await.unwrap_or_default()
    });

    view! {
        <div>
            <table class="scoreboard">
                <tr class="header">
                    <th class="n">
                        "#"
                    </th>
                    <th class="name">
                        "Name"
                    </th>
                    <th class="time">
                        "Time"
                    </th>
                </tr>
                <Transition fallback=move || view! { <ScoreRows scores=vec![] /> }>
                    {move || view! { <ScoreRows scores=score_getter().unwrap_or_default() /> }}
                </Transition>
            </table>
        </div>
    }
}

// 排行榜行组件(用于显示具体的分数记录: 包括名词 用户名 耗时)
#[component]
fn ScoreRows(mut scores: Vec<Score>) -> impl IntoView {
    scores.resize_with(MAX_SCORES, Default::default);

    scores
        .into_iter()
        .zip(1..=MAX_SCORES)
        .map(
            |(
                Score {
                    username,
                    time_in_seconds,
                },
                n,
            )| {
                view! {
                    <tr class={ if n % 2 == 0 { "even" } else { "odd" }}>
                        <td class="n">
                            { n.to_string() }
                        </td>
                        <td class="name">
                            {username}
                        </td>
                        <td class="time">
                            { (time_in_seconds > 0).then(|| to_time(time_in_seconds)) }
                        </td>
                    </tr>
                }
            },
        )
        .collect_view()
}
