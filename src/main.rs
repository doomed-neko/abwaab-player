use reqwest as http;
use serde_json::Value;
use std::{
    borrow::Cow,
    collections::HashMap,
    env::{args, Args},
    process::exit,
    string::String,
};
use tao::{
    event::{StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use url::Url;
use wry::{
    http::{HeaderMap, HeaderValue},
    WebViewBuilder,
};

const ABWAAB_URL: &str = "https://gw.abgateway.com/content/lesson";

struct Abwaab {
    id: u32,
    item_type: String,
    program_id: u32,
    mobile_user: bool,
    access_token: String,
}

impl Abwaab {
    fn new(info: HashMap<String, String>) -> Self {
        let id = info.get("lesson_id").unwrap().to_owned();
        let id: u32 = id.trim().parse().unwrap();
        let item_type = "lesson".to_owned();
        let program_id = info.get("program_id").unwrap().to_owned();
        let program_id = program_id.trim().parse().unwrap();
        let mobile_user = true;
        let access_token = info.get("x_access_token").unwrap().to_owned();
        Abwaab {
            id,
            item_type,
            program_id,
            mobile_user,
            access_token,
        }
    }
    fn iterator(&self) -> (Vec<(&str, String)>, String) {
        let a = vec![
            ("id", self.id.to_string()),
            ("item_type", self.item_type.to_owned()),
            ("program_id", self.program_id.to_string()),
            ("mobile_user", self.mobile_user.to_string()),
        ];
        (a, self.access_token.to_owned())
    }
}

fn get_params(url: Url) -> HashMap<String, String> {
    if url.scheme() != "abwaab-player" {
        std::process::exit(-1);
    }
    let params: Vec<(Cow<'_, str>, Cow<'_, str>)> = url.query_pairs().collect();
    let mut param: HashMap<String, String> = HashMap::new();
    for (k, v) in params {
        param.insert(k.to_string(), v.to_string());
    }
    param
}

fn parse_args(args: Args) -> Url {
    let a = args.last().unwrap();
    let url: Url;
    if let Ok(u) = Url::parse(&a) {
        url = u.to_owned();
    } else {
        exit(-5)
    }
    url
}
async fn get_vid_data(obj: Abwaab) -> Value {
    let it = obj.iterator();
    let url = Url::parse_with_params(ABWAAB_URL, it.0).unwrap();
    let req = http::Client::new();
    let e = req
        .get(url)
        .header("X-Access-Token", it.1)
        .header("Platform", "web")
        .send()
        .await
        .unwrap();
    let e: Value = match e.json().await {
        Ok(t) => t,
        Err(e) => {
            println!("{e}");
            exit(-6)
        }
    };
    let status = e.get("status").unwrap().to_owned();
    if let Value::String(v) = status {
        if v == *"200" {
            let url = e.get("data").unwrap().to_owned();
            if let Value::Array(t) = url {
                let url = t.first().unwrap();
                return url.to_owned();
            }
        }
    }
    std::process::exit(-2)
}

async fn ui() -> wry::Result<()> {
    let url: Url = parse_args(args());
    let params = get_params(url);
    let obj = Abwaab::new(params);
    let vid_data = get_vid_data(obj);
    let vid_url = vid_data.await.get("lesson_url").unwrap().to_owned();
    if let Value::String(url) = vid_url {
        let event_loop = EventLoop::new();
        let win = WindowBuilder::new()
            .with_title("abwaab-player")
            .build(&event_loop)
            .unwrap();
        let mut head = HeaderMap::new();
        head.insert(
            "Referer",
            HeaderValue::from_str("https://www.abwaabiraq.com/").unwrap(),
        );
        let _web = WebViewBuilder::new().with_url(url).with_headers(head);
        #[cfg(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "ios",
            target_os = "android"
        ))]
        let _webview = _web.build(&win)?;
        #[cfg(not(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "ios",
            target_os = "android"
        )))]
        let _webview = {
            use tao::platform::unix::WindowExtUnix;
            use wry::WebViewBuilderExtUnix;
            let vbox = win.default_vbox().unwrap();
            _web.build_gtk(vbox)?
        };

        event_loop.run(move |ev, _, ctrl| {
            *ctrl = ControlFlow::Wait;

            match ev {
                tao::event::Event::NewEvents(StartCause::Init) => println!("Running"),
                tao::event::Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *ctrl = ControlFlow::Exit;
                }
                _ => (),
            }
        });
    }
    Ok(())
}

#[tokio::main()]
async fn main() -> wry::Result<()> {
    ui().await
}
