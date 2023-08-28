use chrono::prelude::*;
use axum::{
	Router,
    routing::get,
	response::{IntoResponse,Html},
    extract::Query,
    http::header,
};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use std::process::Stdio;

async fn index() -> Html<&'static str> {
        Html(r###"
<html>
<head>
  <script src="https://unpkg.com/htmx.org@1.9.5"></script>
</head>
<body style='background:black;color:white'>
<div hx-get="/clock" hx-trigger="every 1s"></div>
</body>
<html>
"###)
}

async fn clock() -> impl IntoResponse {
    let local: DateTime<Local> = Local::now();
    Html(local.time().format(r#"%H:%M:%S<br/><img src="/clock.jpg?h=%H&m=%M&s=%S"/>"#).to_string())
}

#[derive(Deserialize)]
struct MyTime {
  h: u8,
  m: u8,
  s: u8,
}

async fn clock_img(t: Query<MyTime>) -> impl IntoResponse {
    let mut child = Command::new("ghostscript")
	.args(&["-sDEVICE=jpeg", "-sOutputFile=-", "-q", "-g150x150", "-"])
	.stdin(Stdio::piped())
    .stdout(Stdio::piped())
	.stderr(Stdio::null())
    .spawn().unwrap();
     let mut stdin = child.stdin.take().unwrap();
     stdin.write_all(format!(r#"
%!PS-Adobe-3.0 EPSF-3.0
%%BoundingBox: 0 0 150 150
%%EndComments
/max_length     150 def
/line_size      1.5 def
/marker         5 def
/origin         {{0 dup}} def
/center         {{max_length 2 div}} def
/radius         center def
/hour_segment    {{0.50 radius mul}} def
/minute_segment  {{0.80 radius mul}} def
/second_segment  {{0.90 radius mul}} def
/red            {{1 0 0 setrgbcolor}} def
/green          {{0 1 0 setrgbcolor}} def
/blue           {{0 0 1 setrgbcolor}} def
/black          {{0 0 0 setrgbcolor}} def
/hour_angle {{
    {0} {1} 60 div add 3 sub 30 mul
    neg
}} def
/minute_angle {{
        {1} {2} 60 div add 15 sub 6 mul
        neg
}} def
/second_angle {{
        {2} 15 sub 6 mul
        neg
}} def
center dup translate
black clippath fill
line_size setlinewidth
origin radius 0 360 arc blue stroke
gsave
1 1 12 {{
        pop
        radius marker sub 0 moveto 
        marker 0 rlineto red stroke
        30 rotate
}} for
grestore
origin moveto
hour_segment hour_angle cos mul
hour_segment hour_angle sin mul 
lineto green stroke
origin moveto
minute_segment minute_angle cos mul
minute_segment minute_angle sin mul
lineto green stroke
origin moveto
second_segment second_angle cos mul
second_segment second_angle sin mul
lineto green stroke
origin line_size 2 mul 0 360 arc red fill
showpage
End_of_PostScript_Code
"#, t.h, t.m, t.s).as_bytes()).await.unwrap();
     drop(stdin);
     let out = child.wait_with_output().await.unwrap();
     (
         [(header::CONTENT_TYPE, "image/jpeg")],
        out.stdout
     )
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
	.route("/", get(index))
	.route("/clock", get(clock))
	.route("/clock.jpg", get(clock_img));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
