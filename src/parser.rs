use pest::Parser;
#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct CParser;

pub fn parse(source_code: String) {
    let pairs = CParser::parse(Rule::program, &source_code)
        .unwrap()
        .next()
        .unwrap();
    todo!("parser tree -> abstract syntax tree");
}

#[cfg(test)]
mod test {
    use pest::iterators::Pair;
    use pest::Parser;
    use std::fs::File;
    use std::io::Read;

    fn dfs(tab: usize, pair: Pair<'_, super::Rule>) {
        for _ in 0..tab {
            print!("  ");
        }
        println!("{:?}", pair.as_rule());
        for i in pair.into_inner() {
            dfs(tab + 1, i);
        }
    }
    #[test]
    fn basic() {
        let mut f = File::open("test/test.c").unwrap();
        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();
        let pairs = super::CParser::parse(super::Rule::program, &buf)
            .unwrap()
            .next()
            .unwrap();
        // println!("{:?}", pairs.as_span());

        dfs(0, pairs);
    }
}
