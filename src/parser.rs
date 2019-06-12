pub mod basic_functions;

use crate::scanner;
use std::ops::Shr;

pub struct Parser<S: scanner::Scanner, T: 'static> {
    f: Box<dyn FnOnce(S) -> (T, S, Vec<String>)>
}

impl<S> Parser<S, S>
where S: 'static + scanner::Scanner {
    pub fn get_scanner() -> Self {
        Parser {
            f: Box::new(|scanner| (S::from_scanner(&scanner), scanner, vec![]))
        }
    }
}

impl<S> Parser<S, ()>
where S: 'static + scanner::Scanner {
    pub fn set_scanner(scanner: S) -> Self {
        Parser {
            f: Box::new(move |_| ((), S::from_scanner(&scanner), vec![]))
        }
    }
}

impl<S> Parser<S, bool>
where S: 'static + scanner::Scanner {
    pub fn or(self, other: Self) -> Self {
        self >> |a|
        other >> move |b|
        Parser::result(a || b)
    }
    
    pub fn if_else<T>(self, t: Parser<S, T>, f: Parser<S, T>) -> Parser<S, T> {
        self >> |is_true|
            if is_true {
                t
            } else {
                f
            }
    }
}

impl<S, T> Parser<S, T>
where S: 'static + scanner::Scanner {
    pub fn result(value: T) -> Self {
        Parser {
            f: Box::new(move |scanner| (value, scanner, vec![]))
        }
    }
    
    pub fn error(value: T, error: String) -> Self {
        Parser {
            f: Box::new(move |scanner| (value, scanner, vec![error]))
        }
    }
    
    pub fn run(self, scanner: S) -> Result<T, Vec<String>> {
        let (value, _, errors) = self.evaluate(scanner);
        if errors.is_empty() {
            Ok(value)
        } else {
            Err(errors)
        }
    }
    
    fn evaluate(self, scanner: S) -> (T, S, Vec<String>) {
        (self.f)(scanner)
    }
}

// Mimicking Haskell's >>= operator
// Later, code macro for this instead?
impl<S, T, U: 'static, V: 'static> Shr<V> for Parser<S, T> 
where V: FnOnce(T) -> Parser<S, U>,
      S: 'static + scanner::Scanner {
    type Output = Parser<S, U>;
    
    fn shr(self, f: V) -> Parser<S, U> {
        Parser {
            f: Box::new(move |scanner| {
                let (value, scanner, mut errors) = self.evaluate(scanner);
                let value = f(value);
                let (v, s, mut other_errors) = value.evaluate(scanner);
                errors.append(&mut other_errors);
                (v, s, errors)
            })
        }
    }
}

fn multi_if<S, T>(mut branches: Vec<(Parser<S, bool>, Parser<S, T>)>, otherwise: Parser<S, T>)
-> Parser<S, T>
where S: 'static + scanner::Scanner {
    match branches.pop() {
        None => otherwise,
        Some((cond, t)) => multi_if(branches, cond.if_else(t, otherwise))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    
    type TestParser<T> = Parser<TestScanner, T>;
    
    #[test]
    fn parser_monad() {
        let parser = TestParser::result(0);
        assert_eq!(Ok(0), parser.run(TestScanner::new("")));
        
        let parser = TestParser::get_scanner();
        assert_eq!(Ok(TestScanner::new("")), parser.run(TestScanner::new("")));
        
        let parser = TestParser::set_scanner(TestScanner::new("a"));
        assert_eq!(((), TestScanner::new("a"), vec![]), parser.evaluate(TestScanner::new("")));
        
        let parser = TestParser::get_scanner() >> |scanner| TestParser::result(scanner.next_token());
        let token = TestScanner::new("a").next_token();
        assert_eq!(Ok(token), parser.run(TestScanner::new("a")));
    }
    
    #[test]
    fn if_else_test() {
        let parser = TestParser::result(true).if_else(
            TestParser::result("success"),
            TestParser::result("fail")
        );
        assert_eq!(Ok("success"), parser.run(TestScanner::new("")));
        
        let parser = TestParser::result(false).if_else(
            TestParser::result("fail"),
            TestParser::result("success")
        );
        assert_eq!(Ok("success"), parser.run(TestScanner::new("")));
    }
    
    #[test]
    fn or_test() {
        let parser = TestParser::result(false).or(TestParser::result(false));
        assert!(!parser.run(TestScanner::new("")).unwrap());
        
        let parser = TestParser::result(true).or(TestParser::result(false));
        assert!(parser.run(TestScanner::new("")).unwrap());
        
        let parser = TestParser::result(false).or(TestParser::result(true));
        assert!(parser.run(TestScanner::new("")).unwrap());
        
        let parser = TestParser::result(true).or(TestParser::result(true));
        assert!(parser.run(TestScanner::new("")).unwrap());
    }
    
    #[test]
    fn multi_if_test() {
        let parser = multi_if(vec![
            (TestParser::result(false), TestParser::result("fail 1")),
            (TestParser::result(true), TestParser::result("success"))
        ], TestParser::result("fail 2"));
        
        assert_eq!(Ok("success"), parser.run(TestScanner::new("")));
    }
    
    #[test]
    fn error() {
        let parser = TestParser::error((), "success".to_string());
        assert_eq!(Err(vec!["success".to_string()]), parser.run(TestScanner::new("")));
        
        let parser = TestParser::error((), "success".to_string()) >> |_|
                     TestParser::result("failed");
        assert_eq!(Err(vec!["success".to_string()]), parser.run(TestScanner::new("")));
        
        let parser = TestParser::error((), "success 1".to_string()) >> |_|
                     TestParser::result("ignored") >> |_|
                     TestParser::error((), "success 2".to_string());
        assert_eq!(Err(vec!["success 1".to_string(), "success 2".to_string()]),
                   parser.run(TestScanner::new("")));
    }
    
    #[derive(Debug, PartialEq)]
    struct TestScanner {
        code: String
    }
    
    impl TestScanner {
        fn new(code: &str) -> Self {
            TestScanner {
                code: code.to_string()
            }
        }
        
        fn next_token(&self) -> String {
            self.code.to_owned()
        }
    }
    
    impl scanner::Scanner for TestScanner {
        
        type Token = TestToken; // Unimportant
        
        fn from_scanner(other: &Self) -> Self {
            TestScanner {
                code: other.code.to_owned()
            }
        }
        
        // Unused in tests
        fn scan_token(self) -> Self {
            self
        }
        
        fn is_finished(&self) -> bool {
            false
        }
        
        fn current_token(&self) -> Rc<Self::Token> {
            Rc::new(TestToken {})
        }
        
        fn next_token(&self) -> Rc<Self::Token> {
            Rc::new(TestToken {})
        }
    }
    
    struct TestToken {}
    impl TestToken {}
    impl scanner::Token for TestToken {
        type TokenType = ();
        fn t_type(&self) -> Self::TokenType {}
    }
}