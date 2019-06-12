use crate::parser::Parser;
use crate::scanner::{Scanner, Token};

use std::rc::Rc;

pub fn is_at_end<S>() -> Parser<S, bool>
where S: 'static + Scanner {
    Parser::get_scanner() >> |scanner: S|
    Parser::result(scanner.is_finished())
}

pub fn previous<S>() -> Parser<S, Rc<S::Token>>
where S: 'static + Scanner {
    Parser::get_scanner() >> |scanner: S|
    Parser::result(scanner.current_token())
}

pub fn peek<S>() -> Parser<S, Rc<S::Token>>
where S: 'static + Scanner {
    Parser::get_scanner() >> |scanner: S|
    Parser::result(scanner.next_token())
}

pub fn advance<S>() -> Parser<S, Rc<S::Token>>
where S: 'static + Scanner {
    Parser::get_scanner() >> |scanner: S|
    Parser::set_scanner(scanner.scan_token()) >> |_|
    previous()
}

pub fn check<S>(t_type: <S::Token as Token>::TokenType) -> Parser<S, bool>
where S: 'static + Scanner{
    is_at_end().if_else(
        Parser::result(false),
        peek() >> move |token: Rc<S::Token>|
            Parser::result(<S as Scanner>::Token::t_type(&*token) == t_type)
    )
}

pub fn matches<S>(t_type: <S::Token as Token>::TokenType) -> Parser<S, bool>
where S: 'static + Scanner {
    check(t_type).if_else(
        advance() >> |_|
            Parser::result(true),
        Parser::result(false)
    )
}