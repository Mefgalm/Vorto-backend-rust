use std::collections::LinkedList;

use std::{rc::Rc};

use crate::error::{VortoError, VortoErrorCode, VortoResult};
use html5ever::tendril::TendrilSink;
use html5ever::{
    parse_document,
    tree_builder::{TreeBuilderOpts},
    ParseOpts,
};
use markup5ever_rcdom::{Node, NodeData, RcDom};
use regex::Regex;
use urlencoding::encode;

fn vocs_url(word: &str) -> String {
    format!(
        "https://ru.wiktionary.org/w/index.php?title={}&printable=yes",
        encode(word)
    )
}

struct Attr<'r> {
    name: &'r str,
    value: &'r str,
}

impl<'r> Attr<'r> {
    pub fn new(name: &'r str, value: &'r str) -> Self {
        Self { name, value }
    }
}

fn find_nodes(node: &Rc<Node>, find_name: &str, attributes: &Vec<Attr>) -> Vec<Rc<Node>> {
    let mut nodes = vec![];
    let mut stack = LinkedList::new();
    stack.push_back(node.clone());

    while !stack.is_empty() {
        let n = stack.pop_front().unwrap();
        if let NodeData::Element {
            name,
            attrs,
            template_contents: _,
            mathml_annotation_xml_integration_point: _,
        } = &n.data
        {
            let attrs_borrow = attrs.borrow();
            let is_match_attrs = attributes.is_empty()
                || attributes.iter().any(|a| {
                    attrs_borrow.iter().any(|na| {
                        na.name.local.to_string() == a.name && na.value.to_string() == a.value
                    })
                });
            if find_name == name.local.to_string() && is_match_attrs {
                nodes.push(n.clone());
            }
        }
        for c in n.children.try_borrow().unwrap().iter() {
            stack.push_back(c.clone());
        }
    }
    nodes
}

fn find_node(node: &Rc<Node>, find_name: &str, attributes: &Vec<Attr>) -> Rc<Node> {
    let nodes = find_nodes(node, find_name, attributes);

    if let Some(node) = nodes.first() {
        node.clone()
    } else {
        let attrs_str = attributes
            .iter()
            .map(|a| format!("(name: {}, value: {})", a.name, a.value))
            .collect::<Vec<_>>()
            .join(", ");

        error!("Node with name '{}' and attrs: '{}' not found", find_name, attrs_str);
        panic!("Node with name '{}' and attrs: '{}' not found", find_name, attrs_str);
    }
}

enum State {
    LangBlock,
    Definitions,
    Ol,
}

fn get_text(node_data: &NodeData) -> String {
    if let NodeData::Text { contents } = node_data {
        contents.borrow().to_string()
    } else {
        error!("Node is not a Text");
        panic!("Node is not a Text");
    }
}

fn get_all_text(node: &Rc<Node>) -> String {
    let mut stack = LinkedList::new();
    let mut text = String::new();
    stack.push_back(node.clone());

    while !stack.is_empty() {
        let n = stack.pop_front().unwrap();

        text.push_str(&get_text(&n.data));

        for c in n.children.borrow().iter().rev() {
            stack.push_front(c.clone());
        }
    }
    text
}

fn get_definition_with_vocs(node: &Rc<Node>) -> (Vec<String>, String) {
    let mut vocs = vec![];    
    let a_s = find_nodes(node, "a", &vec![]);

    for a in a_s {
        if let VortoResult::Ok(span) = get_node_single_text(&a, "span", &vec![]) {
            vocs.push(span);
        }
    }
    (vocs, get_all_text(&node))
}

fn get_node_single_text(
    node: &Rc<Node>,
    find_name: &str,
    attributes: &Vec<Attr>,
) -> VortoResult<String> {
    let fnode = find_node(&node, find_name, attributes);
    let children = fnode.children.borrow();
    VortoResult::Ok(get_text(&children.first()?.data))
}

fn replace_u(def: &str) -> String {
    def.replace("\u{a0}", " ")
}

fn remove_multiple_spaces(def: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r" +").unwrap();
    }

    RE.replace_all(def, " ").into_owned()
}

fn remove_samples(def: &str) -> String {
    if let Some(position) = def.find('◆') {
        def[..position].to_owned()
    } else {
        def.to_owned()
    }
}

fn remove_noise(def: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(\[. \d+\])|(\[\d+\])").unwrap();
    }
    RE.replace_all(def, " ").trim().to_owned()
}

fn remove_vocs(vocs: &Vec<String>, def: &str) -> String {
    let mut new_def = String::from(def);
    for voc in vocs {
        new_def = new_def.replace(voc, "");
    }
    new_def.trim_start_matches(", ").to_owned()
}

fn pretty_definition(vocs: &Vec<String>, def: &str) -> String {
    remove_multiple_spaces(&remove_noise(&remove_samples(&remove_vocs(
        vocs,
        &replace_u(def),
    ))))
}

fn get_element_name(node: &Rc<Node>) -> Option<String> {
    if let NodeData::Element {
        name,
        attrs: _,
        template_contents: _,
        mathml_annotation_xml_integration_point: _,
    } = &node.data
    {
        Some(name.local.to_string())
    } else {
        None
    }
}

fn get_definitions(root: &Rc<Node>) -> VortoResult<Vec<(Vec<String>, String)>> {
    let mw = find_node(&root, "div", &vec![Attr::new("class", "mw-parser-output")]);
    let mut state = State::LangBlock;
    let mut definitions = vec![];

    for node in mw.children.borrow().iter() {
        match state {
            State::LangBlock => {
                if let VortoResult::Ok(text) =
                    get_node_single_text(&node, "span", &vec![Attr::new("class", "mw-headline")])
                {
                    if text == "Русский" {
                        state = State::Definitions;
                    }
                }
            }
            State::Definitions => {
                if let Some(elem_name) = get_element_name(&node) {
                    if elem_name == "h1" {
                        break;
                    }
                }

                if let VortoResult::Ok(text) =
                    get_node_single_text(&node, "span", &vec![Attr::new("class", "mw-headline")])
                {
                    if text == "Значение" {
                        state = State::Ol;
                    }
                }
            }
            State::Ol => {
                let ol_node = find_node(&node, "ol", &vec![]);
                for li in find_nodes(&ol_node, "li", &vec![]) {
                    let (vocs, def) = get_definition_with_vocs(&li);
                    let pretty_def = pretty_definition(&vocs, &def);
                    if !String::is_empty(&pretty_def) {
                        definitions.push((vocs, pretty_def));
                    }
                    
                }
                state = State::Definitions;
            }
        }
    }

    VortoResult::Ok(definitions)
}

pub async fn parse(word: &str) -> VortoResult<Vec<(Vec<String>, String)>> {
    let text = reqwest::get(vocs_url(word)).await?.text().await?;
    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let dom = parse_document(RcDom::default(), opts)
        .from_utf8()
        .read_from(&mut text.as_bytes())?;

    VortoResult::Ok(get_definitions(&dom.document)?)
}