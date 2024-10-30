use std::collections::VecDeque;

pub fn one_of<Token, Tree>(
    tokens: &VecDeque<Token>,
    fns: Vec<fn(&VecDeque<Token>) -> Option<(VecDeque<Token>, Tree)>>,
) -> Option<(VecDeque<Token>, Tree)> {
    for f in fns {
        let res = f(tokens);
        if res.is_some() {
            return res;
        }
    }
    None
}

#[macro_export]
macro_rules! one_of {
    ($tokens:ident, $( $x:ident ),* ) => {
        {
            use crate::compiler_tools::parser::one_of;
            one_of(&$tokens, vec![$( $x ),*])
        }
    };
}
