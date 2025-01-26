use std::ops::Deref;

use rust_norg::{NorgAST, NorgASTFlat, ParagraphSegment};

pub(crate) fn anchor_identifier(segment: &[ParagraphSegment]) -> String {
    segment
        .iter()
        .map(|segment| match segment {
            ParagraphSegment::Token(token) => match token {
                rust_norg::ParagraphSegmentToken::Text(x) => x.to_string(),
                rust_norg::ParagraphSegmentToken::Whitespace => " ".to_string(),
                rust_norg::ParagraphSegmentToken::Special(c) => c.to_string(),
                rust_norg::ParagraphSegmentToken::Escape(c) => c.to_string(),
            },
            _ => unimplemented!("dont support weird content in links yet"),
        })
        .collect::<String>()
}

pub(crate) fn anchor_disovery(ast: &[&NorgAST]) -> Vec<(String, String)> {
    fn inner(segments: &[ParagraphSegment]) -> Vec<(String, String)> {
        segments
            .iter()
            .filter_map(|x| match x {
                ParagraphSegment::AnchorDefinition { content, target } => {
                    let key = anchor_identifier(content);
                    let link = match target.deref() {
                        ParagraphSegment::Link { filepath, .. } => filepath,
                        _ => unimplemented!("dont support weird target in links yet"),
                    };
                    Some((key, link.to_owned()?))
                }
                _ => None,
            })
            .collect()
    }
    fn inner_flat(ast: &[&NorgASTFlat]) -> Vec<(String, String)> {
        ast.iter()
            .flat_map(|x| match x {
                NorgASTFlat::Paragraph(vec) => inner(vec),
                NorgASTFlat::NestableDetachedModifier { content, .. } => {
                    inner_flat(&[content.deref()])
                }
                NorgASTFlat::RangeableDetachedModifier { title, content, .. } => inner(title)
                    .into_iter()
                    .chain(inner_flat(content.iter().collect::<Vec<_>>().as_slice()))
                    .collect(),
                NorgASTFlat::Heading { title, .. } => inner(title),
                NorgASTFlat::CarryoverTag { .. } => Vec::new(),
                NorgASTFlat::VerbatimRangedTag { .. } => Vec::new(),
                NorgASTFlat::RangedTag { .. } => Vec::new(),
                NorgASTFlat::InfirmTag { .. } => Vec::new(),
                NorgASTFlat::DelimitingModifier(_) => Vec::new(),
            })
            .collect()
    }
    ast.iter()
        .flat_map(|x| match x {
            NorgAST::Paragraph(vec) => inner(vec),
            NorgAST::NestableDetachedModifier { text, content, .. } => {
                anchor_disovery(content.iter().collect::<Vec<_>>().as_slice())
                    .into_iter()
                    .chain(inner_flat(&[text.deref()]))
                    .collect()
            }
            NorgAST::RangeableDetachedModifier { title, content, .. } => inner(title)
                .into_iter()
                .chain(inner_flat(content.iter().collect::<Vec<_>>().as_slice()))
                .collect(),
            NorgAST::Heading { title, content, .. } => {
                anchor_disovery(content.iter().collect::<Vec<_>>().as_slice())
                    .into_iter()
                    .chain(inner(title))
                    .collect()
            }
            NorgAST::CarryoverTag { .. } => Vec::new(),
            NorgAST::VerbatimRangedTag { .. } => Vec::new(),
            NorgAST::RangedTag { content, .. } => {
                inner_flat(content.iter().collect::<Vec<_>>().as_slice())
            }
            NorgAST::InfirmTag { .. } => Vec::new(),
            NorgAST::DelimitingModifier(_) => Vec::new(),
        })
        .collect()
}
