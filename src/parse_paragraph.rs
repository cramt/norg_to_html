use std::collections::HashMap;

use build_html::{HtmlElement, HtmlTag};
use rust_norg::ParagraphSegment;

use crate::anchor_disovery::anchor_identifier;

pub(crate) fn parse_paragraph(
    class_prefixes: &[&str],
    segments: &[ParagraphSegment],
    container_element: HtmlElement,
    anchors: &HashMap<String, String>,
) -> HtmlElement {
    segments.iter().fold(container_element, |mut html, x| {
        match x {
            ParagraphSegment::Token(token) => match token {
                rust_norg::ParagraphSegmentToken::Text(s) => html.add_child(s.into()),
                rust_norg::ParagraphSegmentToken::Whitespace => html.add_child(" ".into()),
                rust_norg::ParagraphSegmentToken::Special(c) => {
                    html.add_child(c.to_string().into())
                }
                rust_norg::ParagraphSegmentToken::Escape(c) => html.add_child(c.to_string().into()),
            },
            ParagraphSegment::AttachedModifierOpener(_) => todo!(),
            ParagraphSegment::AttachedModifierOpenerFail(_) => todo!(),
            ParagraphSegment::AttachedModifierCloserCandidate(_) => todo!(),
            ParagraphSegment::AttachedModifierCloser(_) => todo!(),
            ParagraphSegment::AttachedModifierCandidate {
                modifier_type,
                content,
                closer,
            } => todo!(),
            ParagraphSegment::AttachedModifier {
                modifier_type,
                content,
            } => {
                let class_postfix = match modifier_type {
                    '*' => "bold",
                    '/' => "italics",
                    '_' => "underline",
                    '-' => "strikethrough",
                    '^' => "superscript",
                    ',' => "subscript",
                    '!' => "spoiler",
                    '%' => "comment",
                    // TODO: do math conversion with like https://github.com/ronkok/Temml or other
                    // mathml things
                    _ => unimplemented!("other paragraph modifiers"),
                };
                let classes = class_prefixes
                    .iter()
                    .map(|x| format!("{x}-{class_postfix}"))
                    .chain(class_prefixes.iter().map(|x| x.to_string()))
                    .collect::<Vec<_>>();
                html.add_child(
                    parse_paragraph(
                        classes
                            .iter()
                            .map(|x| x.as_str())
                            .collect::<Vec<_>>()
                            .as_slice(),
                        content,
                        HtmlElement::new(HtmlTag::Span).with_attribute("class", classes.join(" ")),
                        anchors,
                    )
                    .into(),
                );
            }
            ParagraphSegment::Link {
                filepath,
                targets,
                description,
            } => todo!(),
            ParagraphSegment::AnchorDefinition { content, .. }
            | ParagraphSegment::Anchor { content, .. } => {
                let mut element = HtmlElement::new(HtmlTag::Link);

                if let Some(target) = anchors.get(&anchor_identifier(content)) {
                    element.add_attribute("href", target);
                }

                html.add_child(parse_paragraph(class_prefixes, content, element, anchors).into());
            }
            ParagraphSegment::InlineLinkTarget(vec) => todo!(),
            ParagraphSegment::InlineVerbatim(vec) => todo!(),
        }
        html
    })
}
