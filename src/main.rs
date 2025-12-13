use arith::execute;

fn main() {
    // Example: (\x : Int -> x + 10) 5
    let src = r#"
        let f = .\ x : Int -> x * 2 in
        let y = 10 in
        f y
    "#;
    execute(src);
}