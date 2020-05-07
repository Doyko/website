use std::sync::Arc;

use handlebars::Handlebars;
use serde::Serialize;
use serde_json::json;
use std::fs;
use warp::Filter;

struct WithTemplate<T: Serialize> {
    name: String,
    value: T,
}

fn render<T: Serialize>(template: WithTemplate<T>, hbs: Arc<Handlebars>) -> impl warp::Reply {
    let render = hbs.render(&template.name, &template.value).unwrap();
    warp::reply::html(render)
}

#[tokio::main]
async fn main() {
    let mut hb = Handlebars::new();

    // templates
    let template = fs::read_to_string("./views/template.hbs").unwrap();
    hb.register_template_string("template", template).unwrap();

    let paths = fs::read_dir("./views/pages").unwrap();
    let pages: Vec<String> = paths
        .map(|file| {
            let path = file.unwrap().path();
            let t = fs::read_to_string(&path).unwrap();
            let s = path.file_stem().unwrap().to_str().unwrap().to_string();
            hb.register_template_string(&s, t).unwrap();
            s
        })
        .collect();

    let hb = Arc::new(hb);

    let handlebars = move |with_template| render(with_template, hb.clone());

    println!("pages : {:?}", pages);

    // get //

    // home
    let root = warp::get()
        .and(warp::path::end())
        .map(|| WithTemplate {
            name: String::from("home"),
            value: json!(
            {
                "title" : "home",
                "parent" : "template"
            }),
        })
        .map(handlebars.clone());

    // files
    let files = warp::path("resources").and(warp::fs::dir("views/resources/"));

    // views
    let views = warp::get()
        .and(warp::path::param())
        .map(move |name: String| {
            if pages.contains(&name) {
                return WithTemplate {
                    name: name.clone(),
                    value: json!(
                    {
                        "title" : name,
                        "parent" : "template"
                    }),
                };
            }
            WithTemplate {
                name: String::from("error_404"),
                value: json!(
                {
                    "title" : "error_404",
                    "parent" : "template"
                }),
            }
        })
        .map(handlebars);

    let routes = root.or(files).or(views);

    warp::serve(routes).run(([127, 0, 0, 1], 1337)).await;
}
