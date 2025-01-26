use std::collections::HashMap;

use build_html::{HtmlElement, HtmlTag};
use parse_paragraph::parse_paragraph;
use rust_norg::{NorgAST, NorgASTFlat};
use yaml_rust2::YamlLoader;

use crate::{
    html_entry::{html_entry_flatten, HtmlEntry, HtmlEntryFlat},
    parse_paragraph,
};

pub(crate) fn parse_ast(
    class_prefixes: &[&str],
    ast: NorgAST,
    anchors: &HashMap<String, String>,
) -> HtmlEntry {
    match ast {
        rust_norg::NorgAST::Paragraph(vec) => HtmlEntry::Main(parse_paragraph(
            class_prefixes,
            vec.as_slice(),
            HtmlElement::new(HtmlTag::ParagraphText),
            anchors,
        )),
        rust_norg::NorgAST::NestableDetachedModifier {
            modifier_type,
            level,
            extensions,
            text,
            content,
        } => {
            // dont know why we do this https://github.com/NTBBloodbath/norgolith/blob/master/src/converter.rs#L160
            let mod_text = if let NorgASTFlat::Paragraph(s) = *text.clone() {
                s
            } else {
                unreachable!();
            };
            match modifier_type {
                rust_norg::NestableDetachedModifier::Quote => todo!(),
                rust_norg::NestableDetachedModifier::UnorderedList
                | rust_norg::NestableDetachedModifier::OrderedList => {
                    let main = parse_paragraph(
                        class_prefixes,
                        &mod_text,
                        HtmlElement::new(HtmlTag::ListElement),
                        anchors,
                    );
                    let main = html_entry_flatten(
                        content
                            .into_iter()
                            .map(|x| parse_ast(class_prefixes, x, anchors))
                            .collect(),
                    )
                    .into_iter()
                    .fold(main, |html, x| match x {
                        HtmlEntryFlat::Main(html_element) => html.with_child(html_element.into()),
                        _ => html,
                    });
                    match modifier_type {
                        rust_norg::NestableDetachedModifier::Quote => unreachable!(),
                        rust_norg::NestableDetachedModifier::UnorderedList => {
                            HtmlEntry::UnorderedList(main)
                        }
                        rust_norg::NestableDetachedModifier::OrderedList => {
                            HtmlEntry::OrderedList(main)
                        }
                    }
                }
            }
        }
        rust_norg::NorgAST::RangeableDetachedModifier {
            modifier_type,
            title,
            extensions,
            content,
        } => todo!(),
        rust_norg::NorgAST::Heading {
            level,
            title,
            extensions,
            content,
        } => {
            let classes = class_prefixes
                .iter()
                .map(|x| format!("{x}-header-{level}"))
                .chain(class_prefixes.iter().map(|x| x.to_string()))
                .collect::<Vec<_>>();
            let tag = match level {
                1 => HtmlTag::Heading1,
                2 => HtmlTag::Heading2,
                3 => HtmlTag::Heading3,
                4 => HtmlTag::Heading4,
                5 => HtmlTag::Heading5,
                6 => HtmlTag::Heading6,
                _ => HtmlTag::ParagraphText,
            };
            let classes = classes.iter().map(|x| x.as_str()).collect::<Vec<_>>();
            let header_element = HtmlElement::new(tag).with_attribute("class", classes.join(" "));
            let header_element = parse_paragraph(
                classes.as_slice(),
                title.as_slice(),
                header_element,
                anchors,
            );
            let inner = html_entry_flatten(
                content
                    .into_iter()
                    .map(|x| parse_ast(class_prefixes, x, anchors))
                    .collect(),
            )
            .into_iter()
            .fold(
                HtmlElement::new(HtmlTag::Div).with_child(header_element.into()),
                |html, x| match x {
                    HtmlEntryFlat::Main(html_element) => html.with_child(html_element.into()),
                    _ => html,
                },
            );
            HtmlEntry::Main(inner)
        }
        rust_norg::NorgAST::CarryoverTag {
            tag_type,
            name,
            parameters,
            next_object,
        } => todo!(),
        rust_norg::NorgAST::VerbatimRangedTag {
            name,
            parameters,
            content,
        } => {
            match name
                .iter()
                .map(|x| x.as_str())
                .collect::<Vec<_>>()
                .as_slice()
            {
                ["document", "meta"] => YamlLoader::load_from_str(content.as_str())
                    .ok()
                    .and_then(|x| x.into_iter().next())
                    .map(HtmlEntry::Metadata)
                    .unwrap_or(HtmlEntry::Empty),
                _ => unimplemented!("other metadata types"),
            }
        }
        rust_norg::NorgAST::RangedTag {
            name,
            parameters,
            content,
        } => todo!(),
        rust_norg::NorgAST::InfirmTag { name, parameters } => todo!(),
        rust_norg::NorgAST::DelimitingModifier(delimiting_modifier) => todo!(),
    }
}
