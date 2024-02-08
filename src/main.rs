use std::{collections::HashMap, fs};

use askama::*;
use typst2pdf::rendering::typst2pdf;

#[derive(Template)]
#[template(path = "main.typ", escape = "none")]
struct Doc {
    input: String,
}

fn main() {
    let content = Doc {
        input: "HOWYDDDDD".to_string(),
    };

    let helper_filename = "helpers.typ";
    let helper = r#"#let variable = "I WIN" "#;

    dbg!(&content.to_string());

    let mut files = HashMap::<String, String>::new();
    files.insert("main.typ".to_string(), content.to_string());
    files.insert(helper_filename.to_string(), helper.to_string());

    let fonts: Vec<&[u8]> = vec![include_bytes!("../fonts/InterVariable.ttf")];

    let pdf = typst2pdf(files, fonts);
    // Create world with content.
    fs::write("./output.pdf", pdf).expect("Error writing PDF.");
    println!("Created pdf: `./output.pdf`");
}
