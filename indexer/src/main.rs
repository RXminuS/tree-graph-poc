use anyhow::Result;
use apache_age::{tokio::AgeClient, AgType, NoTls};
use convert_case::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::iter::FromIterator;
use tokio_postgres::GenericClient;
use tree_sitter::{Language, Node, Parser, Tree};
use tree_sitter_graph::{
    ast::File, functions::Functions, graph::Graph, ExecutionConfig, Identifier, NoCancellation,
    Variables,
};
use tree_sitter_traversal::{traverse, traverse_tree, Order};

const CODE: &str = r#"
    function double(x: number, n: number = 2) {
        return x * n;
    }

    double(a)
"#;

const DSL: &str = r#"
(identifier) @id
{
  node @id.node
}
"#;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WSTNode {
    pub id: usize,
    pub text: String,
    pub start_column: usize,
    pub start: usize,
    pub start_row: usize,
    pub start_byte: usize,
    pub end_column: usize,
    pub end_row: usize,
    pub end_byte: usize,
    //TODO: pub preorder: usize,
    pub named: bool,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WSTEdge {
    pub parent_id: usize,
    pub child_id: usize,
    pub field: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let (mut db, _) = apache_age::tokio::Client::connect_age(
        "host=localhost user=admin port=5430 dbname=treegraph",
        NoTls,
    )
    .await
    .unwrap();

    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_typescript::language_typescript())
        .expect("Error loading TypeScript grammar");

    let mut parser = Parser::new();
    let lang = tree_sitter_typescript::language_typescript();
    parser.set_language(lang).unwrap();
    let tree = parser.parse(CODE, None).unwrap();
    let file = File::from_str(tree_sitter_typescript::language_typescript(), DSL)
        .expect("Cannot parse file");
    let functions = Functions::stdlib();
    let mut globals = Variables::new();
    globals
        .add(Identifier::from("filename"), "test.ts".into())
        .unwrap();
    // .map_err(|_| ExecutionError::DuplicateVariable("filename".into()))?;
    let mut config = ExecutionConfig::new(&functions, &globals);
    let graph: Graph = file
        .execute(&tree, CODE, &mut config, &NoCancellation)
        .unwrap();

    db.drop_graph("index").await?;
    db.create_graph("index").await?;

    for (preorder, node) in traverse_tree(&tree, Order::Pre).enumerate() {
        let id = node.id();
        dbg!("TRAVERSAL", id);
        //this is a bit weird but I didn't want to re-implement the tree traversal. So essentially for every node we assume it was already created when we visited the parent
        if let Some(_) = node.parent() {
            let parent = node;
            let mut cursor = node.walk();
            let mut has_next = cursor.goto_first_child();
            while has_next {
                let field_name = cursor.field_name();
                let child = cursor.node();
                insert_node(&mut db, &child).await?;
                insert_edges(&mut db, &parent, &child, field_name).await?;
                has_next = cursor.goto_next_sibling();
            }
        } else {
            insert_node(&mut db, &node).await?;
        }
    }

    println!("{}", graph.pretty_print());

    Ok(())
    // let result = graph.pretty_print().to_string();
    // let tree: Tree = parser.parse(CODE, None).unwrap();

    // // walk the tree and print all attributes

    // // let file = std::fs::File::create("ast.dot").unwrap();
    // // parsed.print_dot_graph(&file);
    // // println!("{:#?}", parsed);
    // let preorder: Vec<Node<'_>> = traverse(tree.walk(), Order::Pre).collect::<Vec<_>>();
    // let postorder: Vec<Node<'_>> = traverse_tree(&tree, Order::Post).collect::<Vec<_>>();
    // for node in preorder {
    //     println!(
    //         "{} {}",
    //         node.kind(),
    //         node.utf8_text(CODE.as_bytes()).unwrap()
    //     )
    //     // println!("{:#?}", node);
    // }

    // let graph = tree_sitter_graph::graph::
    // print!("preorder: {:?}\n", preorder);
}

async fn insert_node(
    db: &mut apache_age::tokio::Client,
    node: &Node<'_>,
) -> Result<u64, tokio_postgres::Error> {
    let start_point = node.start_position();#[cfg(test)]
                                            mod tests {
                                                use super::*;

                                                #[test]
                                                fn test_index_file() {
                                                    let file_path = "test_file.txt";
                                                    let file_contents = "This is a test file.";
                                                    std::fs::write(file_path, file_contents).unwrap();

                                                    let result = index_file(file_path);

                                                    assert!(result.is_ok());
                                                    let index_result = result.unwrap();
                                                    assert_eq!(index_result.file_path, file_path);
                                                    assert_eq!(index_result.file_contents, file_contents);

                                                    std::fs::remove_file(file_path).unwrap();
                                                }

                                                #[test]
                                                fn test_index_file_nonexistent() {
                                                    let file_path = "nonexistent_file.txt";

                                                    let result = index_file(file_path);

                                                    assert!(result.is_err());
                                                }
                                            }

    let end_point = node.end_position();
    dbg!("CREATION", node.id());
    db.execute_cypher(
        "index",
        "CREATE(n: WSTNode { id: $id, text: $text, start_column: $start_column, start_row: $start_row, start_byte: $start_byte, end_column: $end_column, end_row: $end_row, end_byte: $end_byte, preorder: $preorder, named: $named, type: $type })",
        Some(AgType::<WSTNode>(WSTNode {
            id: node.id(),
            text: node.utf8_text(CODE.as_bytes()).expect("Should be able to read bytes").to_owned(),
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            start_row: start_point.row,
            end_row: end_point.row,
            start_column: start_point.column,
            end_column: end_point.column,
            //TODO: preorder: preorder,
            named: node.is_named(),
            r#type: node.kind().to_string(),
        })),
    ).await
}

async fn insert_edges(
    db: &mut apache_age::tokio::Client,
    parent: &Node<'_>,
    child: &Node<'_>,
    field: Option<&str>,
) -> Result<(), tokio_postgres::Error> {
    let stmt = if let Some(field) = field {
        format!(
            r#"
        MATCH (p: WSTNode {{ id: $parent_id }}), (c: WSTNode {{ id: $child_id }})
        CREATE (p)-[:CHILD]->(c),
               (p)-[:{rel}]->(c)
        "#,
            rel = field.to_case(Case::UpperSnake)
        )
    } else {
        r#"
        MATCH (p: WSTNode { id: $parent_id }), (c: WSTNode { id: $child_id })
        CREATE (p)-[:CHILD]->(c)
        "#
        .to_owned()
    };
    if let Some(field) = field {
        if field == "parameters" {
            dbg!(parent.id(), child.id());
        }
    }
    db.execute_cypher(
        "index",
        &stmt,
        Some(AgType::<WSTEdge>(WSTEdge {
            parent_id: parent.id(),
            child_id: child.id(),
            field: field.map(|s| s.to_owned()),
        })),
    )
    .await?;

    Ok(())
}
