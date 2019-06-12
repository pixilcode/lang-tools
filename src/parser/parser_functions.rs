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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn is_at_end_test() {
        assert_eq!(Ok(false), is_at_end().run(TestScanner::new(vec![TestToken::a()])));
        assert_eq!(Ok(true), is_at_end().run(TestScanner::new(vec![])));
    }
    
    #[test]
    fn previous_test() {
        let scanner = TestScanner::new(vec![TestToken::a()]);
        let previous_token = scanner.current_token();
        assert_eq!(Ok(previous_token), previous().run(TestScanner::new(vec![TestToken::a()])));        
    }
    
    #[test]
    fn peek_test() {
        let scanner = TestScanner::new(vec![TestToken::a()]);
        let next_token = scanner.next_token();
        assert_eq!(Ok(next_token), peek().run(TestScanner::new(vec![TestToken::a()])));
    }
    
    #[test]
    fn check_test() {
        assert_eq!(Ok(true), check(TokenType::A).run(TestScanner::new(vec![TestToken::a()])));
        assert_eq!(Ok(false), check(TokenType::A).run(TestScanner::new(vec![TestToken::b()])));
    }
    
    #[test]
    fn advance_test() {
        let next_token = TestScanner::new(vec![TestToken::a()]).scan_token().current_token();
        assert_eq!(Ok(next_token), advance().run(TestScanner::new(vec![TestToken::a()])));
        
        // Ensure that advance actually advances the scanner
        assert!((
            previous() >> |a|
            peek() >> |b|
            advance() >> |_|
            previous() >> move |c| {
                assert_ne!(a, c);
                assert_eq!(b, c);
                Parser::result(())
            }
        ).run(TestScanner::new(vec![TestToken::a()])).is_ok());
    }
    
    #[test]
    fn matches_test() {
        assert_eq!(Ok(true), matches(TokenType::A).run(TestScanner::new(vec![TestToken::a()])));
        
        // Ensure that match advances the scanner
        assert!((
            previous() >> |a|
            peek() >> |b|
            matches(TokenType::A) >> |c|
            previous() >> move |d| {
                assert!(c);
                assert_ne!(a, d);
                assert_eq!(b, d);
                Parser::result(())
            }
        ).run(TestScanner::new(vec![TestToken::a()])).is_ok());
        
        let scanner = TestScanner::new(vec![TestToken::a()]);
        let next_token = scanner.scan_token().current_token();
        assert_eq!(Ok(next_token),
            (matches(TokenType::A) >> |_|
            previous()).run(TestScanner::new(vec![TestToken::a()])));
    }
    
    struct TestScanner {
        tokens: Vec<TestToken>,
        is_at_start: usize
    }
    impl TestScanner {
        fn new(tokens: Vec<TestToken>) -> Self { TestScanner { tokens, is_at_start: 0 } }
    }
    impl Scanner for TestScanner {
        type Token = TestToken;
        
        fn from_scanner(scanner: &Self) -> Self {
            TestScanner {
                tokens: scanner.tokens.clone(),
                is_at_start: scanner.is_at_start
            }
        }
        fn scan_token(mut self) -> Self {
            if self.is_at_start == 0 {
                self.is_at_start = 1;
            } else {
                self.tokens.remove(0);
            }
            self
        }
        fn is_finished(&self) -> bool {
            self.tokens.is_empty()
        }
        fn current_token(&self) -> Rc<Self::Token> {
            match self.tokens.get(0) {
                _ if self.is_at_start == 0 => Rc::new(TestToken(TokenType::None)),
                Some(a) => Rc::new(a.clone()),
                None => Rc::new(TestToken(TokenType::None))
            }
        }
        fn next_token(&self) -> Rc<Self::Token> {
            match self.tokens.get(self.is_at_start) {
                Some(a) => Rc::new(a.clone()),
                None => Rc::new(TestToken(TokenType::None))
            }
        }
    }
    
    #[derive(PartialEq, Clone, Debug)]
    struct TestToken(TokenType);
    impl TestToken {
        fn a() -> Self {
            TestToken(TokenType::A)
        }
        
        fn b() -> Self {
            TestToken(TokenType::B)
        }
    }
    impl Token for TestToken {
        type TokenType = TokenType;
        fn t_type(&self) -> Self::TokenType {
            self.0.clone()
        }
    }
    
    #[derive(PartialEq, Clone, Debug)]
    enum TokenType {
        A,
        B,
        None
    }
    
}