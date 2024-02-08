use std::{collections::HashMap, fs, io::Cursor};

use askama::*;
use typst2pdf::typst2pdf;

#[derive(Template)]
#[template(path = "main.typ", escape = "none")]
struct Doc {
    input: String,
}

fn main() {
    let content = Doc {
        input: "HOWYDDDDD".to_string(),
    };

    let mut files = HashMap::<String, &[u8]>::new();
    let content = content.to_string();
    files.insert("main.typ".to_string(), content.as_bytes());

    files.insert(
        "mLogo.jpeg".to_string(),
        include_bytes!("../templates/mLogo.jpeg"),
    );

    let fonts: Vec<&[u8]> = vec![include_bytes!("../fonts/InterVariable.ttf")];

    let pdf = typst2pdf(files, fonts);
    // Create world with content.
    fs::write("./output.pdf", pdf).expect("Error writing PDF.");
    println!("Created pdf: `./output.pdf`");
}
