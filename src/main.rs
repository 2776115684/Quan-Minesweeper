cfg_if::cfg_if! {
    // 条件编译宏，根据feature "ssr"选择不同的代码路径
    if #[cfg(feature = "ssr")] {
        use axum::{
            body::{boxed, Body, BoxBody},
            extract::{Path, State, RawQuery, FromRef},
            http::{Request, Response, StatusCode, Uri, header::HeaderMap},
            response::IntoResponse,
            response::Response as AxumResponse,
            routing::get,
            Router,
        };
        use leptos::logging::*;
        use leptos::*;
        use leptos_axum::{generate_route_list, LeptosRoutes};
        use sqlx::SqlitePool;
        use tower::ServiceExt;
        use tower_http::services::ServeDir;

        use quan_minesweeper::app::App;

        // 定义应用状态结构体，包含Leptos选项和数据库连接池
        #[derive(FromRef, Debug, Clone)]
        struct AppState {
            leptos_options: LeptosOptions,
            db_pool: SqlitePool,
        }

        // 主函数，启动异步执行环境
        #[tokio::main]
        async fn main() {
            // 初始化日志记录
            simple_logger::init_with_level(log::Level::Info).expect("logging initializes");

            // 获取Leptos配置选项
            let leptos_options = get_configuration(None)
                .await
                .expect("configuration exists")
                .leptos_options;
            let addr = leptos_options.site_addr;
            let routes = generate_route_list(App);
            let db_url = dotenvy::var("DATABASE_URL").expect(".env exists");
            let db_pool = SqlitePool::connect(&db_url)
                .await
                .expect("sqlite ready for connections");

            // 运行数据库迁移
            sqlx::migrate!("./migrations")
                .run(&db_pool)
                .await
                .expect("database migrated");

            // 创建应用状态
            let state = {
                let db_pool = db_pool.clone();
                AppState {
                    leptos_options,
                    db_pool,
                }
            };

            // 设置路由和处理器
            let app = Router::new()
                .route(
                    "/api/*fn_name",
                    get(server_fn_handler).post(server_fn_handler),
                )
                .leptos_routes_with_context(&state, routes, move || {
                    provide_context(db_pool.clone());
                }, App)
                .fallback(file_and_error_handler)
                .with_state(state);

            log!("listening on http://{}", &addr);
            axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .await
                .expect("axum server binds to addr");
        }

        // 处理服务器端函数调用的路由处理器
        async fn server_fn_handler(
            State(db_pool): State<SqlitePool>,
            path: Path<String>,
            headers: HeaderMap,
            raw_query: RawQuery,
            request: Request<Body>,
        ) -> impl IntoResponse {
            leptos_axum::handle_server_fns_with_context(
                path,
                headers,
                raw_query,
                move || {
                    provide_context(db_pool.clone());
                },
                request,
            )
            .await
        }

        // 处理静态文件和错误响应的路由处理器
        async fn file_and_error_handler(
            uri: Uri,
            State(options): State<LeptosOptions>,
            req: Request<Body>,
        ) -> AxumResponse {
            let root = options.site_root.clone();
            let res = get_static_file(uri.clone(), &root).await.unwrap();

            if res.status() == StatusCode::OK {
                res.into_response()
            } else {
                let handler =
                    leptos_axum::render_app_to_stream(options.to_owned(), move || view! { <App />});
                handler(req).await.into_response()
            }
        }

        // 获取静态文件的辅助函数
        async fn get_static_file(uri: Uri, root: &str) -> Result<Response<BoxBody>, (StatusCode, String)> {
            let req = Request::builder()
                .uri(uri.clone())
                .body(Body::empty())
                .unwrap();
            match ServeDir::new(root).oneshot(req).await {
                Ok(res) => Ok(res.map(boxed)),
                Err(err) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Something went wrong: {err}"),
                )),
            }
        }
    } else {
        // 非ssr模式下的主函数
        pub fn main() {}
    }
}
