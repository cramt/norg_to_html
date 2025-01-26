use build_html::{HtmlElement, HtmlTag};
use yaml_rust2::Yaml;

#[derive(Debug)]
pub(crate) enum HtmlEntry {
    Metadata(Yaml),
    Main(HtmlElement),
    UnorderedList(HtmlElement),
    OrderedList(HtmlElement),
    Empty,
}

#[derive(Debug)]
pub(crate) enum HtmlEntryFlat {
    Metadata(Yaml),
    Main(HtmlElement),
}

pub(crate) fn html_entry_flatten(mut entries: Vec<HtmlEntry>) -> Vec<HtmlEntryFlat> {
    entries.push(HtmlEntry::Empty);
    let min_size = entries
        .iter()
        .filter(|x| match x {
            HtmlEntry::Metadata(_) => true,
            HtmlEntry::Main(_) => true,
            HtmlEntry::UnorderedList(_) => false,
            HtmlEntry::OrderedList(_) => false,
            HtmlEntry::Empty => false,
        })
        .count();
    entries
        .into_iter()
        .fold(
            (
                Vec::with_capacity(min_size),
                Option::<HtmlElement>::None,
                Option::<HtmlElement>::None,
            ),
            |(mut v, unordered, ordered), x| {
                if let Some(unordered) = unordered {
                    match x {
                        HtmlEntry::UnorderedList(html_element) => {
                            return (v, Some(unordered.with_child(html_element.into())), None)
                        }
                        _ => {
                            v.push(HtmlEntryFlat::Main(unordered));
                        }
                    }
                } else if let HtmlEntry::UnorderedList(html_element) = x {
                    return (
                        v,
                        Some(
                            HtmlElement::new(HtmlTag::UnorderedList)
                                .with_child(html_element.into()),
                        ),
                        None,
                    );
                }
                if let Some(ordered) = ordered {
                    match x {
                        HtmlEntry::OrderedList(html_element) => {
                            return (v, Some(ordered.with_child(html_element.into())), None)
                        }
                        _ => {
                            v.push(HtmlEntryFlat::Main(ordered));
                        }
                    }
                } else if let HtmlEntry::OrderedList(html_element) = x {
                    return (
                        v,
                        Some(
                            HtmlElement::new(HtmlTag::OrderedList).with_child(html_element.into()),
                        ),
                        None,
                    );
                }
                match x {
                    HtmlEntry::Metadata(yaml) => v.push(HtmlEntryFlat::Metadata(yaml)),
                    HtmlEntry::Main(html_element) => v.push(HtmlEntryFlat::Main(html_element)),
                    _ => {}
                }
                (v, None, None)
            },
        )
        .0
}
