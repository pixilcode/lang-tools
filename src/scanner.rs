use std::rc::Rc;

pub trait Scanner {
    type Token: Token;
    fn from_scanner(scanner: &Self) -> Self;
    fn scan_token(self) -> Self;
    fn is_finished(&self) -> bool;
    fn current_token(&self) -> Rc<Self::Token>;
    fn next_token(&self) -> Rc<Self::Token>;
}

pub trait Token {
    type TokenType: PartialEq;
    fn t_type(&self) -> Self::TokenType;
}