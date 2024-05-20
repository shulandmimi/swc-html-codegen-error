use std::{path::PathBuf, sync::Arc};
use swc_common::{
    errors::HANDLER, input::SourceFileInput, sync::Lrc, FileName, Globals, SourceFile, SourceMap,
    GLOBALS,
};
use swc_ecma_transforms::helpers::{Helpers, HELPERS};
use swc_error_reporters::handler::try_with_handler;
use anyhow::Result;
use swc_html_ast::Document;
use swc_html_codegen::{
    writer::basic::{BasicHtmlWriter, BasicHtmlWriterConfig},
    CodeGenerator, CodegenConfig, Emit,
};
use swc_html_minifier::{minify_document, option::MinifyOptions};
use swc_html_parser::{
    lexer::Lexer,
    parser::{Parser, ParserConfig},
};
pub struct Source {
    pub path: PathBuf,
    pub content: Lrc<String>,
}

pub fn create_swc_source_map(source: Source) -> (Arc<SourceMap>, Lrc<SourceFile>) {
    let cm = Arc::new(SourceMap::default());
    let sf = cm.new_source_file_from(FileName::Real(source.path), source.content);

    (cm, sf)
}

pub fn try_with<F>(cm: Arc<SourceMap>, globals: &Globals, op: F) -> Result<()>
where
    F: FnOnce(),
{
    GLOBALS.set(globals, || {
        try_with_handler(cm, Default::default(), |handler| {
            HELPERS.set(&Helpers::new(true), || HANDLER.set(handler, op));
            Ok(())
        })
    })
}

fn html_codegen(doc: &Document) -> String {
    let mut html_code = String::new();
    let html_writer = BasicHtmlWriter::new(&mut html_code, None, BasicHtmlWriterConfig::default());
    let mut html_gen = CodeGenerator::new(
        html_writer,
        CodegenConfig {
            minify: true,
            ..Default::default()
        },
    );

    html_gen.emit(doc).unwrap();

    html_code
}

fn main() {
    let (_cm, source_file) = create_swc_source_map(Source {
        path: PathBuf::from("test.js"),
        content: Lrc::new(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="X-UA-Compatible" content="IE=edge">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Document</title>
</head>
<body>
  <div id="root"></div>
  <script src="./src/index.tsx"></script>
</body>
</html>
        "#
            .to_string(),
        ),
    });

    let html_lexer = Lexer::new(SourceFileInput::from(&*source_file));
    let mut parser = Parser::new(
        html_lexer,
        ParserConfig {
            ..Default::default()
        },
    );

    let parse_result = parser.parse_document().unwrap();
    let mut minify_parse_result = parse_result.clone();

    minify_document(&mut minify_parse_result, &MinifyOptions::default());

    println!("html_code:\n{}", html_codegen(&parse_result));
    println!(
        "\nminify html_code:\n{}",
        html_codegen(&minify_parse_result)
    );
}
