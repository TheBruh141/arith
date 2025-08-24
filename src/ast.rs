pub enum ASTNode {
    Number(f64),
    Variable(String),
    Addition { left: Box<ASTNode>, right: Box<ASTNode> },
    Subtraction { left: Box<ASTNode>, right: Box<ASTNode> },
    Multiplication { left: Box<ASTNode>, right: Box<ASTNode> },
    Division { left: Box<ASTNode>, right: Box<ASTNode> },
    // Maybe extend later:
    UnaryMinus(Box<ASTNode>),
    // FunctionCall { name: String, args: Vec<Node> },
}
