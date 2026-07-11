use axum::response::Html;

const DASHBOARD_HTML: &str = include_str!("assets/index.html");

pub async fn serve_dashboard() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}
