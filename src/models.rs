#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: String,
    pub language: String,
    pub size: i64,
    pub checksum: String,
}

#[derive(Debug, Clone)]
pub struct FunctionNode {
    pub file_path: String,
    pub name: String,
    pub line: usize,
    pub end_line: usize,
    pub signature: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ClassNode {
    pub file_path: String,
    pub name: String,
    pub line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeType {
    Contains,
    Calls,
    Imports,
    InheritsFrom,
}

impl EdgeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeType::Contains => "Contains",
            EdgeType::Calls => "Calls",
            EdgeType::Imports => "Imports",
            EdgeType::InheritsFrom => "InheritsFrom",
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeData {
    File(FileNode),
    Function(FunctionNode),
    Class(ClassNode),
    Stub(String), // Placeholder for forward references before parsing
}

impl NodeData {
    pub fn id(&self) -> String {
        match self {
            NodeData::File(f) => f.path.clone(),
            NodeData::Function(f) => format!("{}:{}", f.file_path, f.name),
            NodeData::Class(c) => format!("{}:{}", c.file_path, c.name),
            NodeData::Stub(id) => id.clone(),
        }
    }
}
