#[derive(Debug)]
pub enum Ast {
    Use {
        from: String,
        name: Option<String>,
        items: Vec<String>
    },
    Doc(String),
    Typ, // todo
    Def, // todo
    Let, // todo
}