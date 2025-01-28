pub mod anchor_disovery;
pub mod html_entry;
pub mod parse_ast;
pub mod parse_paragraph;
use build_html::Html;
use itertools::Either;
use itertools::Itertools;
use serde_yaml2::wrapper::YamlNodeWrapper;
use std::{collections::HashMap, ffi::OsStr, path::Path};

use anchor_disovery::anchor_disovery;
use async_walkdir::WalkDir;
use futures::{future::join_all, stream::StreamExt};
use html_entry::html_entry_flatten;
use html_entry::HtmlEntryFlat;
use parse_ast::parse_ast;
use rust_norg::parse_tree;
use tokio::fs::{create_dir_all, read_to_string};
use serde::{Serialize, Deserialize};

async fn read_norg_dir(dir: &str) -> HashMap<String, String> {
    WalkDir::new(dir)
        .filter_map(|x| futures::future::ready(x.ok()))
        .filter_map(|x| async move {
            if x.path().extension() == Some(OsStr::new("norg")) {
                Some((
                    x.path()
                        .as_os_str()
                        .to_str()?
                        .replace(dir, "")
                        .replace(".norg", ""),
                    read_to_string(x.path()).await.ok()?,
                ))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect()
}

#[derive(Debug, Serialize)]
pub struct HtmlOutput {
    pub metadata: Option<YamlNodeWrapper>,
    pub html: String,
}

pub fn build_html(norg: &str) -> HtmlOutput {
    let ast = parse_tree(norg).unwrap();
    let anchors = anchor_disovery(ast.iter().collect::<Vec<_>>().as_slice())
        .into_iter()
        .map(|(k, v)| (k, v.replace("$", "")))
        .collect::<HashMap<_, _>>();
    let result_entries = html_entry_flatten(
        ast.into_iter()
            .map(|x| parse_ast(&["norg"], x, &anchors))
            .collect::<Vec<_>>(),
    );
    let (metadatas, html): (Vec<_>, Vec<_>) =
        result_entries.into_iter().partition_map(|x| match x {
            HtmlEntryFlat::Metadata(yaml) => Either::Left(yaml),
            HtmlEntryFlat::Main(html_element) => Either::Right(html_element),
        });
    HtmlOutput {
        metadata: metadatas.into_iter().next().map(YamlNodeWrapper::new),
        html: html.into_iter().map(|x| x.to_html_string()).collect(),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test() {
    let norg_dir = read_norg_dir("../the-seventh-realm").await;
    let result = norg_dir
        .into_iter()
        .map(|(file, body)| (file, build_html(body.as_str())))
        .collect::<HashMap<_, _>>();
    join_all(result.into_iter().map(|(file, output)| async move {
        let path = format!("dist/{file}.html");
        let path = Path::new(path.as_str());
        if let Some(parent) = path.parent() {
            create_dir_all(parent).await.unwrap();
        }
        tokio::fs::write(path, output.html).await.unwrap()
    }))
    .await;
}
