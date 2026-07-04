use crate::closure::*;
use crate::function_core::*;
use crate::gc::{Gc, NO_GC};
use crate::lexer::*;
use crate::token::*;
use crate::value::*;
use std::cell:RefCell;
use std::rc::Rc;

pub(crate) const MAX_LOCALS: usize = 256;
pub(crate) const MAX_UPVALUES: usize = 256;
pub(crate) const MAX_CONSTANTS: usize = 65536;
pub(crate) const MAX_VARIABLES: usize = 65536;
pub(crate) const MAX_FIELDS: usize = 256;
pub(crate) const MAX_METHODS: usize = 65536;

#[derive(Debug, Clone, PartialEq)]
pub enum InterpretErrorType {
    ParseError,
    CompileTimeError,
    RuntimeError,
    OtherError,
}

#[derive(Clone)]
enum BindingPower {
    NoBp,
    Assignment, // = += -= *= /= &= |= ^= <<= >>=

    Ternary,    // ? :
    LOr,        // ||
    LAnd,       // &&
    Equality,   // == !=
    Comparison, // < > <= >=
    Bitwise,    // & ^ |
    Shift,      // << >>
    Term,       // + -
    Factor,     // * /
    Unary,      // ! ~ + -
    Exponent,   // **
    Call,       // . () []
    Primary,
}

impl From<u8> for BindingPower {
    fn from(val: u8) -> Self {
        match val {
            0 => BindingPower::NoBp,
            1 => BindingPower::Assignment, // = += -= *= /* &= |= ^= <<= >>=
            2 => BindingPower::Ternary,    // ? :
            3 => BindingPower::LOr,        // ||
            4 => BindingPower::LAnd,       // &&
            5 => BindingPower::Equality,   // == !=
            6 => BindingPower::Comparison, // < > <= >=
            7 => BindingPower::Bitwise,    // & ^ |
            8 => BindingPower::Shift,      // << >>
            9 => BindingPower::Term,       // + -
            10 => BindingPower::Factor,    // * /
            11 => BindingPower::Unary,     // ! ~ + -
            12 => BindingPower::Exponent,  // **
            13 => BindingPower::Call,      // . (| ( ) [ ]
            14 => BindingPower::Primary,
            _ => panic!("Invalid binding power value {}", val),
        }
    }
}

impl From<BindingPower> for u8 {
    fn from(val: BindingPower) -> u8 {
        match val {
            BindingPower::NoBp => 0,
            BindingPower::Assignment => 1, // = += -= *= /* &= |= ^= <<= >>=
            BindingPower::Ternary => 2,    // ? :
            BindingPower::LOr => 3,        // ||
            BindingPower::LAnd => 4,       // &&
            BindingPower::Equality => 5,   // == !=
            BindingPower::Comparison => 6, // < > <= >=
            BindingPower::Bitwise => 7,    // & ^ |
            BindingPower::Shift => 8,      // << >>
            BindingPower::Term => 9,       // + -
            BindingPower::Factor => 10,    // * /
            BindingPower::Unary => 11,     // ! ~ + -
            BindingPower::Exponent => 12,  // **
            BindingPower::Call => 13,      // . () []
            BindingPower::Primary => 14,
        }
    }
}

struct LexerData {
    lexer: Option<Lexer>,
    curr: Token,
    prev: Token,
}

pub struct Parser <'a> {
    compiler: Compiler,
    vm: VM,
    gc: Gc,
    lexer_stack: Vec<LexerData>,
    error_count: usize,
    in_panic_mode: bool,
    in_panic_lock_mode: bool,

}

impl<'a> Parser {
    fn new(debug: bool, compile: bool) -> Self {
        let mut s = Self {
            compiler: Compiler::new(),
            vm: VM::new(),
            gc: Gc::new(),
            lexer_stack: vec![],
            in_panic_mode: false,
            in_panic_lock_mode: false,
            error_count: 0,
            /*table: vec![
                { NoPrec, false, false }, // Err
                { NoPrec, false, false }, // false
                { NoPrec, false, false }, // Done
                { NoPrec, false, false }, // Ident
                { NoPrec, false, false }, // Int
                { NoPrec, false,match bp {
                            Some(x)
                        } false }, // Uint
                { NoPrec, false, false }, // Float
                { NoPrec, false, false }, // String
                { NoPrec, false, false }, // LetKw
                { NoPrec, false, false }, // MacroKw
                { NoPrec, false, false }, // FunKw
                { NoPrec, false, false }, // AnonClosure
                { NoPrec, false, false }, // AnonFnKw
                { NoPrec, false, false }, // IfKw
                { NoPrec, false, false }, // ElseKw
                { NoPrec, false, false }, // WhileKw
                { NoPrec, false, false }, // DoKw
                { NoPrec, false, false }, // ForKw
                { NoPrec, false, false }, // ClassKw
                { NoPrec, false, false }, // ThisKw
                { NoPrec, false,match bp {
                            Some(x)
                        } false }, // SuperKw
                { NoPrec, false, false }, // StaticKw
                { NoPrec, false, false }, // ContinueKw
                { NoPrec, false, false }, // BreakKw
                { NoPrec, false, false }, // ReturnKw
                { NoPrec, false, false }, // NewKw
                { NoPrec, false, false }, // NullKw
                { NoPrec, false, false }, // TrueKw
                { NoPrec, false, false }, // FalseKw
                { NoPrec, false, match bp {
                            Some(x)
                        }false }, // ConstKw
                { NoPrec, false, false }, // NsKw
                { NoPrec, false, false }, // ImportKw
                { NoPrec, false, false }, // LParen
                { NoPrec, false, false }, // RParen
                { NoPrec, false, false }, // LBrace
                { NoPrec, false, false }, // RBrace
                { NoPrec, false, false }, // LBracket
                { NoPrec, false, false }, // RBracket
                { NoPrec, false, false }, // Question
                { NoPrec, false, false }, // Semicolon
                { NoPrec, false, false }, // Colon
                { NoPrec, false, false }, // DoubleColon
                { NoPrec, false, false }, // Comma
                { NoPrec, false, false }, // Dot
                { NoPrec, false, false }, // Field
                { NoPrec, false, false }, // StaticField
                { NoPrec, false, false }, // LAnd
                { NoPrec, false, false }, // LOr
                { NoPrec, false, false }, // LNot
                { NoPrec, false, false }, // Equal
                { NoPrec, false, false }, // NEqual
                { NoPrec, false, false }, // Less
                { NoPrec, false, false }, // LessEqual
                { NoPrec, false, false }, // Greater
                { NoPrec, false, false }, // GreaterEqual
                { NoPrec, false, false }, // Add
                { NoPrec, false, false }, // Subtract
                { NoPrec, false, false }, // Multiply
                { NoPrec, false, false }, // Divide
                { NoPrec, false, false }, // Modulo
                { NoPrec, false, false }, // Power
                { NoPrec, false, false }, // And
                { NoPrec, false, false }, // Or
                { NoPrec, false, false }, // XOr
                { NoPrec, false, false }, // Not
                { NoPrec, false, false }, // LShift
                { NoPrec, false, false }, // RShift
                { NoPrec, false, false }, // Assign
                { NoPrec, false, false }, // AddAssign
                { NoPrec, false, false }, // SubtractAssign
                { NoPrec, false, false }, // MultiplyAssign
                { NoPrec, false, false }, // DivideAssign
                { NoPrec, false, false }, // ModuloAssign
                { NoPrec, false, false }, // AndAssign
                { NoPrec, false, false }, // OrAssign
                { NoPrec, false, false }, // XorAssign
                { NoPrec, false, false }, // LShiftAssign
                { NoPrec, false, false }, // RShiftAssign
                { NoPrec, false, false }, // IndexMethod
                { NoPrec, false, false }, // IndexAssignMethod
            ],*/
        };
        s.lexer_stack = vec![];
        s.error_count = 0;
        s.lexer_stack.push(LexerData {
            lexer: None,
            curr: Token {
                kind: TokenKind::None,
                ty: TokenType::None,
                lexeme: "".to_string(),
                line: 1,
                col: 1,
                value: Value::Nil,
            },
            prev: Token {
                kind: TokenKind::None,
                ty: TokenType::None,
                lexeme: "".to_string(),
                line: 1,
                col: 1,
                value: Value::Nil,
            },
        });
        s
    }

    pub fn parse_text(&self, text: &mut String) -> Result<Value, InterpretErrorType> {
        self.lexer_stack = vec![];
        self.error_count = 0;
        self.in_panic_mode = false;
        self.in_panic_lock_mode = false;

        let lexer = Lexer::new_from_text(text); 

        self.lexer_stack.clear();
        self.lexer_stack.push(LexerData {
            lexer: Some(lexer),
            curr: Token {
                kind: TokenKind::None,
                ty: TokenType::None,
                lexeme: "".to_string(),
                line: 1,
                col: 1,
                value: Value::Nil,
            },
            prev: Token {
                kind: TokenKind::None,
                ty: TokenType::None,
                lexeme: "".to_string(),
                line: 1,
                col: 1,
                value: Value::Nil,
            },
        });
        self.advance(self.gc);
        if !self.parse(self.gc) {
            Err(InterpretErrorType::ParseError)
        } else {
            vm.run(&mut root, gc, debug, compile)
        }
    }
    pub fn parse_file(&self, fname: String) -> Result<Value, InterpretErrorType> {
        self.lexer_stack = vec![];
        self.error_count = 0;
        self.in_panic_mode = false;
        self.in_panic_lock_mode = false;

        let lexer = Lexer::new_from_file(fname);

        if lexer.is_none() {
            return Err(InterpretErrorType::ParseError)
        }

        self.lexer_stack.push(LexerData {
            lexer: lexer,
            curr: Token {
                kind: TokenKind::None,
                ty: TokenType::None,
                lexeme: "".to_string(),
                line: 1,
                col: 1,
                value: Value::Nil,
            },
            prev: Token {
                kind: TokenKind::None,
                ty: TokenType::None,
                lexeme: "".to_string(),
                line: 1,
                col: 1,
                value: Value::Nil,
            },
        });
        self.advance(self.gc);
        if !self.parse(self.gc) {
            Err(InterpretErrorType::ParseError)
        } else {
            vm.run(gc, debug, compile)
        }
    }


    fn advance(&mut self, gc: &Gc) {
        self.lexer_stack.last_mut().unwrap().prev =
            self.lexer_stack.last_mut().unwrap().curr.clone();
        loop {
            self.lexer_stack.last_mut().unwrap().curr =
                *self.lexer_stack.last_mut().unwrap().scan(gc);
            if self.current().kind != TokenKind::Err {
                break;
            }
            let curr = self.current();
            self.error_at_curr(curr.lexeme)
        }
    }

    fn check(&mut self, tk: TokenKind) -> bool {
        self.current().kind == tk
    }

    fn match_token(&mut self, tk: TokenKind, gc: &Gc) -> bool {
        if self.current().kind != tk {
            return false;
        }
        self.advance(gc);
        return true;
    }

    fn consume(&mut self, tk: TokenKind, err_message: String, gc: &Gc) -> bool {
        if self.current().kind == tk {
            self.advance(gc);
            return true;
        }
        self.error_at_curr(err_message);

        false
    }

    fn error_at(&mut self, token: Token, err_message: String) {
        if self.in_panic_mode || self.in_panic_lock_mode {
            return;
        }

        eprint!(
            "{}:{}:{}: error",
            *self.lexer_stack.last_mut().unwrap().file_name(),
            token.line,
            token.col
        );

        if token.kind == TokenKind::Done {
            eprint!(" at end of file");
        } else if token.kind == TokenKind::Err {
        } else {
            eprint!(" at '{}'", token.lexeme);
        }

        eprintln!(": {}", err_message);
        eprintln!("");
        self.error_count += 1;

        if self.error_count >= 20 {
            if !self.in_panic_mode {
                eprintln!("error: bailing out; too many errors");
                self.in_panic_lock_mode = true;
            }
        }
        self.in_panic_mode = true;
    }

    fn error_at_prev(&mut self, err_message: String) {
        let prev = self.previous();
        self.error_at(prev, err_message);
    }

    fn error_at_curr(&mut self, err_message: String) {
        let curr = self.current();
        self.error_at(curr, err_message);
    }

    fn current(&mut self) -> Token {
        self.lexer_stack.last_mut().unwrap().curr.clone()
    }

    fn previous(&mut self) -> Token {
        self.lexer_stack.last_mut().unwrap().prev.clone()
    }

    // Entry point of parser
    fn parse(&self, gc: &Gc) -> bool {
        while self.current().kind != TokenKind::Done {
            let _ = self.current().kind;
            let left = self.parse_stmt(gc);
            println!("left = {:#?}", left);
            if !left {
                continue;
            }
        }

        if self.error_count > 0 {
            if self.error_count > 1 {
                println!("\nFound {} errors.", self.error_count);
            } else {
                println!("\nFound 1 error.");
            }
            false
        } else {
            true
        }
    }

    // Pratt Parser methods

    // Synchronizes the error system
    fn synchronize_error(&mut self, gc: &Gc) {
        self.in_panic_mode = false;

        while self.current().kind != TokenKind::Done {
            if self.previous().kind == TokenKind::Semicolon {
                return;
            }
            match self.current().kind {
                TokenKind::LetKw |
                TokenKind::FunKw |
                TokenKind::AnonClosure |
                //TokenKind::NsKw |
                TokenKind::IfKw |
                TokenKind::WhileKw |
                TokenKind::ForKw |
                TokenKind::ReturnKw => {
                    return;
                },
                _ => {},
            }

            self.advance(gc);
        }
    }

    // Calls the corresponding statement function
    fn call_stmt_fn(&mut self, tk: TokenKind, gc: &Gc) -> bool {
        match tk {
            TokenKind::LetKw => self.parse_let_stmt(gc),
            TokenKind::ReturnKw => self.parse_return_stmt(gc),
            TokenKind::LBrace => self.parse_block(gc),
            TokenKind::IfKw => self.parse_if_stmt(gc),
            _ => false,
        }
    }

    // Calls the corresponding prefix function
    fn call_prefix_fn(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        match tk {
            TokenKind::Int => self.parse_int(gc),
            TokenKind::Float => self.parse_float(gc),
            TokenKind::String => self.parse_string(gc),
            TokenKind::Ident => self.parse_var(gc),
            TokenKind::TrueKw => self.parse_true(gc),
            TokenKind::FalseKw => self.parse_false(gc),
            TokenKind::NilKw => self.parse_nil(gc),
            TokenKind::Subtract => self.parse_negative(gc),
            TokenKind::Add => self.parse_positive(gc),
            TokenKind::AnonClosure => self.parse_anon_closure(gc),
            TokenKind::FunKw => self.parse_named_closure(gc),
            TokenKind::LNot => self.parse_lnot(gc),
            TokenKind::Not => self.parse_not(gc),
            _ => false,
        }
    }

    // Calls the corresponding infix function
    fn call_infix_fn(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        match tk {
            TokenKind::Add => self.parse_addition(gc),
            TokenKind::Subtract => self.parse_subtraction(gc),
            TokenKind::Multiply => self.parse_multiplication(gc),
            TokenKind::Divide => self.parse_division(gc),
            TokenKind::Modulo => self.parse_modulo(gc),
            TokenKind::Power => self.parse_power(gc),
            TokenKind::LOr => self.parse_lor(gc),
            TokenKind::LAnd => self.parse_land(gc),
            TokenKind::Or => self.parse_or(gc),
            TokenKind::And => self.parse_and(gc),
            TokenKind::XOr => self.parse_xor(gc),
            TokenKind::LShift => self.parse_lshift(gc),
            TokenKind::RShift => self.parse_rshift(gc),
            TokenKind::Assign => self.parse_assignment(gc),
            TokenKind::AddAssign => self.parse_compound_addition(gc),
            TokenKind::SubtractAssign => self.parse_compound_subtraction(gc),
            TokenKind::MultiplyAssign => self.parse_compound_multiplication(gc),
            TokenKind::DivideAssign => self.parse_compound_division(gc),
            TokenKind::ModuloAssign => self.parse_compound_modulo(gc),
            TokenKind::PowerAssign => self.parse_compound_power(gc),
            TokenKind::AndAssign => self.parse_compound_and(gc),
            TokenKind::OrAssign => self.parse_compound_or(gc),
            TokenKind::XOrAssign => self.parse_compound_xor(gc),
            TokenKind::LShiftAssign => self.parse_compound_lshift(gc),
            TokenKind::RShiftAssign => self.parse_compound_rshift(gc),
            TokenKind::LParen => self.parse_function_call(gc),
            _ => false,
        }
    }

    fn stmt_bp(&mut self, tk: TokenKind) -> Option<BindingPower> {
        match tk {
            TokenKind::LetKw => Some(BindingPower::NoBp),
            TokenKind::ReturnKw => Some(BindingPower::NoBp),
            TokenKind::LBrace => Some(BindingPower::NoBp),
            _ => None,
        }
    }

    fn prefix_bp(&mut self, tk: TokenKind) -> Option<BindingPower> {
        match tk {
            TokenKind::Int
            | TokenKind::Float
            | TokenKind::String
            | TokenKind::Ident
            | TokenKind::TrueKw
            | TokenKind::FalseKw
            | TokenKind::NilKw => Some(BindingPower::Primary),
            TokenKind::Add | TokenKind::Subtract | TokenKind::LNot | TokenKind::Not => {
                Some(BindingPower::Unary)
            }
            TokenKind::AnonClosure => Some(BindingPower::Call),
            TokenKind::FunKw => Some(BindingPower::Call),
            _ => None,
        }
    }

    fn infix_bp(&mut self, tk: TokenKind) -> Option<BindingPower> {
        match tk {
            TokenKind::Add | TokenKind::Subtract => Some(BindingPower::Term),
            TokenKind::Multiply | TokenKind::Divide | TokenKind::Modulo | TokenKind::Power => {
                Some(BindingPower::Factor)
            }
            TokenKind::LAnd => Some(BindingPower::LAnd),
            TokenKind::LOr => Some(BindingPower::LOr),
            TokenKind::Or | TokenKind::And | TokenKind::XOr => Some(BindingPower::Bitwise),
            TokenKind::LShift | TokenKind::RShift => Some(BindingPower::Shift),
            TokenKind::Assign
            | TokenKind::AddAssign
            | TokenKind::SubtractAssign
            | TokenKind::MultiplyAssign
            | TokenKind::DivideAssign
            | TokenKind::ModuloAssign
            | TokenKind::PowerAssign
            | TokenKind::OrAssign
            | TokenKind::AndAssign
            | TokenKind::XOrAssign
            | TokenKind::LShiftAssign
            | TokenKind::RShiftAssign => Some(BindingPower::Assignment),
            TokenKind::LParen | TokenKind::LBrace | TokenKind::LBracket => Some(BindingPower::Call),
            _ => None,
        }
    }

    fn parse_stmt(&mut self, gc: &Gc) -> bool {
        let mut node = false;
        let tk = self.current().kind;
        let stmt = self.stmt_bp(tk.clone());
        if stmt.is_some() {
            node = self.call_stmt_fn(tk.clone(), gc);
            if self.in_panic_mode {
                self.advance(gc);
                self.synchronize_error(gc);
            }
            return node;
        }

        node = self.parse_expr_stmt(gc);
        if self.in_panic_mode {
            self.advance(gc);
            self.synchronize_error(gc);
        }

        node
    }

    fn parse_expr(&mut self, gc: &Gc, bp: BindingPower) -> bool {
        let mut tk = self.current().kind;
        let prefix = self.prefix_bp(tk);
        if prefix.is_none() {
            self.error_at_curr("expression expected".to_string());
            return false;
        }

        let mut l = self.call_prefix_fn(gc);

        tk = self.current().kind;
        let mut infix = self.infix_bp(tk);
        if infix.is_none() {
            return l;
        }
        while <BindingPower as Into<u8>>::into(bp.clone())
            <= <BindingPower as Into<u8>>::into(infix.unwrap())
        {
            tk = self.current().kind;
            infix = self.infix_bp(tk);
            if infix.is_none() {
                return l;
            }
            l = self.call_infix_fn(l, gc);
        }

        l
    }

    fn parse_let_stmt(&mut self, gc: &Gc) -> bool {
        self.advance(gc);
        self.consume(
            TokenKind::Ident,
            "identifier expected after 'let'".to_string(),
            gc,
        );
        let ident = self.previous().lexeme;
        //self.advance(gc);
        self.consume(
            TokenKind::Assign,
            "assignment expected after identifier".to_string(),
            gc,
        );

        let e = self.parse_expr(false, gc, BindingPower::Assignment);
        if e.is_none() {
            return false;
        }

        if !self.consume(
            TokenKind::Semicolon,
            "';' expected after let statement".to_string(),
            gc,
        ) {
            false
        } else {
            true
        }
    }

    fn parse_const_stmt(&mut self, _gc: &Gc) -> bool {
        false
    }

    fn parse_class_decl(&mut self, _gc: &Gc) -> bool {
        false
    }

    fn parse_macro_def(&mut self) -> bool{
        false
    }

    fn parse_ns_decl(&mut self, _gc: &Gc) -> bool {
        false
    }

    fn parse_import_cmd(&mut self) -> bool{
        false
    }

    // control flow statements

    // IF Statement

    fn parse_if_stmt(&mut self, gc: &Gc) -> bool{
        self.advance(gc);
        let if_expr = self.parse_expr(gc, BindingPower::Assignment);
        if if_expr {
            self.advance(gc);
            let else_expr = self.parse_expr(gc, BindingPower::Assignment);
            return else_expr;
        }
        false
    }

    // prefix operators

    fn parse_anon_closure(&mut self, gc: &Gc) -> bool {
        self.advance(gc);
        let mut params: Vec<String> = vec![];

        loop {
            if !self.check(TokenKind::RParen) {
                if !self.consume(TokenKind::Ident, "parameter name expected".to_string(), gc) {
                    return false;
                }
                let param = Box::new(Ast::Variable {
                    name: self.previous().lexeme.clone(),
                    arity: false,
                });
                params.push(param);

                self.match_token(TokenKind::Comma, gc);
            } else {
                break;
            }
        }
        if !self.consume(
            TokenKind::RParen,
            "`)` expected after parameters".to_string(),
            gc,
        ) {
            return false;
        }

        self.consume(
            TokenKind::LBrace,
            "`{` expected at start of closure body".to_string(),
            gc,
        );

        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Done) {
            let mut l = false;
            l = self.parse_stmt(gc);
            if let v = l {
                code.push(v);
            } else {
            }
        }

        if !self.consume(
            TokenKind::RBrace,
            "`}` expected at end of closure body".to_string(),
            gc,
        )
        {
            false
        } else {
            true
        }
    }

    fn parse_named_closure(&mut self, gc: &Gc) -> bool {
        self.advance(gc);
        let mut params: Vec<String> = vec![];

        if !self.consume(
            TokenKind::Ident,
            "function name expected after 'fun'".to_string(),
            gc,
        ) {
            self.advance(gc);
        }

        let fun_name = self.previous().lexeme.clone();

        if !self.consume(
            TokenKind::LParen,
            "'(' expected after function name".to_string(),
            gc,
        ) {
            self.advance(gc);
        }
        loop {
            if !self.check(TokenKind::RParen) {
                if params.len() >= 16 {
                    self.error_at_prev("too many parameters".to_string());
                }
                if !self.consume(TokenKind::Ident, "parameter name expected".to_string(), gc) {
                    self.advance(gc);
                }

                
            }
            if !self.match_token(TokenKind::Comma, gc) {
                break;
            }
        }

        if !self.consume(
            TokenKind::RParen,
            "')' expected after parameters".to_string(),
            gc,
        ) {
            self.advance(gc);
        }

        if !self.consume(
            TokenKind::LBrace,
            "'{' expected at start of function body".to_string(),
            gc,
        ) {
            self.advance(gc);
        }

        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Done) {
            let mut l = false;
            l = self.parse_stmt(gc);
            if let v = l {
                code.push(v);
            } else {
            }
        }

        if !self.consume(
            TokenKind::RBrace,
            "'}' expected at end of function body".to_string(),
            gc,
        ) {
            self.advance(gc);
        }

        let ret = if self.error_count > 0 {
            false
        } else {
            true
        } 
    }

    fn parse_return_stmt(&mut self, gc: &Gc) -> bool {
        self.advance(gc);
        if !self.match_token(TokenKind::Semicolon, gc) {
            let ret_expr = self.parse_expr(gc, BindingPower::Assignment);
            if !ret_expr {
                return false;
            }
        }

        self.advance(gc);

        true
    }

    fn parse_break_stmt(&mut self) -> bool{
        false
    }

    fn parse_continue_stmt(&mut self) -> bool {
        false
    }

    fn parse_block(&mut self, gc: &Gc) -> bool{
        self.advance(gc);

        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Done) {
            self.parse_stmt(gc); 
        }

        if !self.consume(
            TokenKind::RBrace,
            "'}' expected at end of block".to_string(),
            gc,
        ) {
            self.advance(gc);
        }

        if self.error_count > 0 {
            false
        } else {
            true
        }
    }

    fn parse_expr_stmt(&mut self, gc: &Gc) -> bool {
        // generate expression nodes
        let node = self.parse_expr(gc, BindingPower::Assignment);

        if node.is_none() {
            return false;
        }

        if !self.consume(
            TokenKind::Semicolon,
            "';' expected after expression".to_string(),
            gc,
        ) {
            false
        } else {
            true
        }
    }

    fn parse_int(&mut self, gc: &Gc) -> bool {
        let v = self.current().value;
        self.advance(gc);
        true
    }

    fn parse_float(&mut self, gc: &Gc) -> bool {
        let v = self.current().value;
        self.advance(gc);
        true
    }

    fn parse_var(&mut self, gc: &Gc) -> bool {
        let name = self.current().lexeme;
        self.advance(gc);
        true
    }

    fn parse_true(&mut self, gc: &Gc) -> bool {
        self.advance(gc);
        true
    }

    fn parse_false(&mut self, gc: &Gc) -> bool {
        self.advance(gc);
        true
    }

    fn parse_nil(&mut self, gc: &Gc) -> bool {
        let v = self.current().value;
        self.advance(gc);
        true
    }

    fn parse_string(&mut self, gc: &Gc) -> bool {
        let v = self.current().value;
        self.advance(gc);
        true
    }

    fn parse_argument_list(&mut self, gc: &Gc) -> Option<Vec<String>> {
        let mut arg_list: Vec<String>> = vec![];
        if !self.check(TokenKind::RParen) {
            while !self.check(TokenKind::RParen) && !self.check(TokenKind::Done) {
                println!("curr = {:#?}", self.current());
                let arg = self.parse_expr(gc, BindingPower::Assignment);

                arg_list.push(a);

                if !self.match_token(TokenKind::Comma, gc) {
                    break;
                }
            }
        }

        if !self.consume(
            TokenKind::RParen,
            "')' expected after arguments".to_string(),
            gc,
        ) {
            self.advance(gc);
            None
        }
        Some(arg_list.clone());
    }

    fn parse_function_call(&mut self, gc: &Gc) -> bool {
        self.advance(gc);
        let arg_list = self.parse_argument_list(left.clone(), gc);

        if arg_list.is_none() {
            false;
        } else {
            true
        }
    }

    fn parse_positive(&mut self, gc: &Gc) -> bool {
        self.advance(gc);
        self.parse_expr(gc, BindingPower::Assignment)
    }

    fn parse_negative(&mut self, gc: &Gc) -> bool {
        self.advance(gc);

        self.parse_expr(gc, BindingPower::Assignment)
    }

    fn parse_lnot(&mut self, gc: &Gc) -> bool {
        self.advance(gc);

        self.parse_expr(gc, BindingPower::Assignment);
    }

    fn parse_not(&mut self, gc: &Gc) -> bool {
        self.advance(gc);

        self.parse_expr(gc, BindingPower::Unary) 
    }

    fn parse_addition(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false
        } else {
            true
        }
    }

    fn parse_subtraction(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false
        } else {
            true
        }
    }

    fn parse_multiplication(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false
        } else {
            true
        }
    }

    fn parse_division(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false
        } else {
            true
        }
    }

    fn parse_modulo(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false
        } else {
            true
        }
    }

    fn parse_power(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);

        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false
        } else {
            true
        }
    }

    fn parse_land(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        if bp.is_some() {
            self.advance(gc);

            let right = self.parse_expr(
                gc,
                BindingPower::from(<BindingPower as Into<u8>>::into(bp.clone().unwrap()) + 1),
            );
            if !right() {
                return false;
            }
            if bp.is_some() {
            }
        }
        false
    }

    fn parse_lor(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        let mut right = false;
        if bp.is_none() {
            return false;
        }

        if bp.is_some() {
            self.advance(gc);

            right = self.parse_expr(
                gc,
                BindingPower::from(<BindingPower as Into<u8>>::into(bp.clone().unwrap()) + 1),
            );
        }

        if !right() {
            return false;
        }

        if bp.is_some() {
            self.advance(gc);

            let right = self.parse_expr(
                gc,
                BindingPower::from(<BindingPower as Into<u8>>::into(bp.clone().unwrap()) + 1),
            );
            if !right() {
                return false;
            }
            if bp.is_some() {
            }
        }
        false
    }

    fn parse_and(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false
        }
    }

    fn parse_or(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false
        }
    }

    fn parse_xor(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            return false;
        }
    }

    fn parse_lshift(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false
        }
    }

    fn parse_rshift(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false
        }
    }

    fn parse_compound_addition(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false;
        }
    }

    fn parse_compound_subtraction(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false
        }
    }

    fn parse_compound_multiplication(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false
        }
    }

    fn parse_compound_division(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false
        }
    }

    fn parse_compound_modulo(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false
        }
    }

    fn parse_compound_power(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false
        }
    }

    fn parse_compound_and(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false
        }
    }

    fn parse_compound_or(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false
        }
    }

    fn parse_compound_xor(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false
        }
    }

    fn parse_compound_lshift(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false
        }
    }

    fn parse_compound_rshift(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if !right() {
            false
        }
    }

    fn parse_assignment(&mut self, gc: &Gc) -> bool {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return false;
        }

        self.advance(gc);


        let right = self.parse_expr(
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap() + 1)),
        );
        if !right() {
            return false;
        }
    }

    // End Pratt Parser methods
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gc::{Gc, NO_GC};
    use crate::parser::interpret_string;

    #[test]
    fn test_add_ns_var() {
        let mut deps = prep_for_test();

        let mut code = "let var = 10;";
        let mut result = interpret_string(
            &mut code.to_string(),
            &mut deps.0,
            &mut deps.1,
            false,
            false,
            false,
        );
        assert_eq!(result, Ok(Value::Nil));

        code = "var;";
        result = interpret_string(
            &mut code.to_string(),
            &mut deps.0,
            &mut deps.1,
            false,
            false,
            false,
        );
        assert_eq!(result, Ok(Value::Int(10)));
    }

    fn prep_for_test() -> (VM, Gc) {
        (VM::new(), Gc::new())
    }
}




struct Compiler {
    symbols: SymbolTables,

}

impl Compiler {
    fn new() -> Self {
        SymbolTables::new()
    }

    #[inline]
    fn register_mod(


    #[inline]
    fn get_mod(
        &self, 
        name: String
    ) -> Result<Module, String> {
        self.symbols.get_mod(self, name)
    }

    #[inline]
    fn declare_fun(
        &self,
        mut name: String,
        params: Vec<VarEntry>
        parent: Option<Rc<FunEnv>>,
        fun_type: FunctionType,
    ) -> Result<usize, String>
        
        name += "(" + params.len().to_string() + ")";

        if params.len() >= 16 {
            return format!("Too many parameters for function '{}'", name);
        }

        self.look_up_var(name);
}

struct SymbolTables {
    mods: Modules,
    calls: Vec<CallEnv>
}

impl SymbolTables {
    fn new() -> Self {
        let mut self = Self {
            mods: Modules::new(),
            calls: vec![],
        }

        let _ = self.declare_fun(
            " main".to_string(),
            vec![],
            None,
            FunctionType::Main,
        )
        self
    }

    fn declare_mod(
        &self,
        name: String
    ) -> Option<usize> {
        self.mod.declare_mod(
            name,
        )
    }
    fn get_mod(
        &self, 
        name: String
    ) -> Result<Module, String> {
        let res = self.mods.get(name);
        if res.is_some() {
            Ok(res.unwrap())
        } else {
            Err(format!("Module '{}' doesn't exist", name))
        }
    }

    fn declare_fun(
        name: String, 
        params: Vec<VarEntry>, 
        parent: Option<Rc<FunEnv>>,
        fun_type: FunctionType
    ) -> usize {
        calls.push(FunEnv::new(
            name,
            params,
            parent,
            fun_type,
        ))
        calls.len()
    }

    fn declare_local(
        name: String,
        local_scope: usize,
        dist_from_top: usize,
    ) -> VarEntry {
        VarEntry::new(
            name,
            VarType::Function
            local_scope,

}

pub struct Modules {
    mod_table: HashMap<String, Module>,
}

impl Modules {
    fn new(&self) -> Self {
        self = Self {
            mod_table: HashMap::<String, usize>::new(),
            mods: Vec::new(),
        }
        self.mod_table.insert("global", 0);
        self.mods.push(Module::new());
    }

    fn declare_mod(
        &self, 
        name: String
    ) -> Result<(), String> {
        if self.mod_table.len() == MAX_MODULES {
            Err(format!("Module limit exceeded'", name))
        } else {
            let id = self.mod_table.len();
            self.mod_table.insert(name, id);
            Ok(())
        }
    }
}

struct Module {
    mod_name: String,
    mod_index: usize,
    mod_parent: Option<Rc<Module>>,
    mod_vars: HashMap<String, VarEntry>,
}

impl Module {
    fn new(
        name: String,
        index: usize,
        parent: Option<Rc<Module>>,
    ) -> {
        Self {
            mod_name: name,
            mod_index: index,
            mod_parent: parent,
            mod_vars: HashMap::<String, VarEntry>::new(),
        }
    }

    fn declare_mod_var(
        
}

struct FunEnv {
    locals: Rc<RefCell<HashMap<String, Option<VarEntry>>>>,
    env_parent: Option<Rc<RefCell<FunEnv>>>,
}

impl FunEnv {
    fn new(parent: Option<Rc<RefCell<FunEnv>>>) {
        Self {
            locals: Rc::new(RefCell::new(HashMap::new())),
            env_parent: parent,
        }
    }


}

#[derive(Clone)]
struct VarEntry {
    name: String,
    arity: Option<usize>,
    var_type: VarType,
    scope_type: ScopeType,
    scope: usize,
    index: usize,
    upvalue_local_index: usize,
    var_mod: Option<Rc<Module>>,
}

impl VarEntry {
    fn new(
        name: String,
        arity: Option<usize>,
        var_type: VarType,
        scope_type: ScopeType,
        scope: usize,
        index: usize,
        upvalue_local_index: usize,
        var_mod: Option<Rc<Module>>,
    ) -> Self {
        Self {
            name,
            arity,
            var_type,
            scope_type,
            scope,
            index,
            upvalue_local_index,
        }
    }
}


struct VarEntry {
    name: String,
    var_type: VarType,
    var_index: usize,
    var_scope: usize,
    value: Value,
    var_mod: Option<Rc<Module>>,
}

impl VarEntry {
    pub fn new(
        name: String, 
        var_type: VarType,
        dist_from_top: usize,
        can_be_optimized: bool,
        value: Option<Value>
    ) -> Self {
        Self {
            name,
            var_type,
            dist_from_top,
            true,
            value,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_type(&self) -> VarType {
        self.var_type.clone()
    }

    pub fn get_value(&self) -> Option<Value> {
        self.value.clone()
    }
}

fn emit_var_load(&mut self, fc: &mut FunCompiler, var: VarInfo) {
    match var.scope_type {
        ScopeType::Ns => {
            fc.emit_word_op(OpCode::LoadVar, var.index as usize);
        }
        ScopeType::Local => {
            if var.index < 9 {
                fc.emit_op(OpCode::from(u8::from(OpCode::LoadLocal0) + var.index as u8));
            } else {
                fc.emit_byte_op(OpCode::LoadLocal, var.index as u8);
            }
        }
        ScopeType::Upvalue => {
            fc.emit_byte_op(OpCode::LoadUpvalue, var.index as u8);
        }
    }
}

fn emit_math_op<'a>(
    &mut self,
    tk: TokenKind,
    l: &Value,
    r: &Value,
    gc: &Gc,
) -> Option<Managed<Closure>> {
    match tk {
        TokenKind::Add {
            self.code_gen(l.clone(), gc);
            self.code_gen(r.clone(), gc);
            last.emit_math_op(MathOp::Add);
            last.closure
        }
        TokenKind::Subtract => {
            self.code_gen(l.clone(), gc);
            self.code_gen(r.clone(), vm, gc);
            last.emit_math_op(MathOp::Subtract);
            last.closure
        }
        TokenKind::Multiply => {
            self.code_gen(l.clone(), vm, gc);
            self.code_gen(r.clone(), vm, gc);
            last.emit_math_op(MathOp::Multiply);
            last.closure
        }
        TokenKind::Divide => {
            self.code_gen(l.clone(), gc);
            self.code_gen(r.clone(), gc);
            last.emit_math_op(MathOp::Divide);
            last.closure
        }
        TokenKind::Modulo => {
            self.code_gen(l.clone(), gc);
            self.code_gen(r.clone(), gc);
            last.emit_math_op(MathOp::Modulo);
            last.closure
        }
        TokenKind::Power => {
            self.code_gen(l.clone(), gc);
            self.code_gen(r.clone(), gc);
            last.emit_math_op(MathOp::Power);
            last.closure
        }
        _ => {
            unreachable!();
        }
    } else {
        panic!("current token kind is invalid");
    }
}

fn get_constant(&mut self, val: &Value) -> usize {
    let len = self.constants.len();

    let pair = self.constants.get_key_value(val);
    if pair.is_none() {
        self.constants.insert(val.clone(), self.constants.len());
        self.constant_arr.push(val.clone());
    } else {
        return *pair.unwrap().1;
    }
    return len;
}

fn emit_op(&mut self, opcode: OpCode) {
    #[cfg(feature = "debug_opcode")]
    println!(
        "gen: {}: {}",
        self.closure.unwrap().get_core().get_name(),
        opcode
    );
    self.code.push(u8::from(opcode));
}

fn emit_byte_op(&mut self, opcode: OpCode, byte: u8) {
    #[cfg(feature = "debug_opcode")]
    println!(
        "gen: {}: {} byte {}",
        self.closure.unwrap().get_core().get_name(),
        opcode,
        byte
    );
    self.code.push(u8::from(opcode));
    self.code.push(byte);
}

fn emit_word_op(&mut self, opcode: OpCode, word: usize) {
    #[cfg(feature = "debug_opcode")]
    println!(
        "gen: {}: {} word {}",
        self.closure.unwrap().get_core().get_name(),
        opcode,
        word
    );
    self.code.push(u8::from(opcode));
    self.code.push(((word >> 8) & 0xff) as u8);
    self.code.push((word & 0xff) as u8);
}

fn emit_three_byte_op(&mut self, opcode: OpCode, three_byte: u32) {
    #[cfg(feature = "debug_opcode")]
    println!(
        "gen: {}: {} three byte {}",
        self.closure.unwrap().get_core().get_name(),
        opcode,
        three_byte
    );
    self.code.push(u8::from(opcode));
    self.code.push(((three_byte >> 16) & 0xff) as u8);
    self.code.push(((three_byte >> 8) & 0xff) as u8);
    self.code.push((three_byte & 0xff) as u8);
}

fn emit_math_op(&mut self, op: MathOp) {
    println!("gen: {} op {}", OpCode::MathOp, op);
    self.code.push(u8::from(OpCode::MathOp));
    self.code.push(u8::from(op));
}





use crate::closure::Closure;
use crate::function_core::{FunctionCore, FunctionType};
use crate::managed::*;
use crate::opcode::*;
use ordered_float::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
struct CallFrame {
    fun: Value,
    closure: Option<Managed<Closure>>,
    core: Option<Managed<FunctionCore>>,
    pc: usize,
    fp: usize,
}

impl CallFrame {
    pub fn new(fun: Value) -> Self {
        let _closure: Option<Managed<Closure>> = None;
        let _core: Option<Managed<FunctionCore>> = None;

        match fun {
            Value::Closure(c) => Self {
                fun,
                closure: Some(c),
                core: Some(c.get_core()),
                pc: 0,
                fp: 0,
            },
            _ => {
                panic!("'{}' got pushed onto the call stack!", fun);
            }
        }
    }
    pub fn get_fun(&self) -> &Value {
        &self.fun
    }

    pub fn get_closure(&self) -> Option<Managed<Closure>> {
        self.closure
    }

    fn get_core(&self) -> Option<Managed<FunctionCore>> {
        self.core
    }

    fn get_pc(&self) -> usize {
        self.pc.clone()
    }

    fn inc_pc(&mut self, amount: usize) -> Option<usize> {
        if self.pc + amount > 200_000 {
            None
        } else {
            self.pc += amount;
            Some(self.pc)
        }
    }

    fn dec_pc(&mut self, amount: usize) -> Option<usize> {
        let mut new_pc = self.pc as isize;
        new_pc -= amount as isize;
        if new_pc < 0 {
            None
        } else {
            self.pc -= amount;
            Some(self.pc)
        }
    }

    #[inline]
    fn get_fp(&self) -> usize {
        self.fp
    }

    #[inline]
    fn set_fp(&mut self, index: usize) -> usize {
        self.fp = index;
        self.fp
    }
}

#[derive(Debug, Clone, PartialEq)]
struct CallStack {
    stack: Vec<CallFrame>,
}

impl CallStack {
    fn new() -> Self {
        Self { stack: vec![] }
    }

    #[inline]
    fn get_fp(&self) -> usize {
        self.stack.last().unwrap().get_fp()
    }

    #[inline]
    fn set_fp(&mut self, index: usize) {
        self.stack.last_mut().unwrap().set_fp(index);
    }

    #[inline]
    fn get_pc(&self) -> usize {
        self.stack.last().unwrap().get_pc()
    }

    fn clear_pc(&mut self) {
        self.stack.last_mut().unwrap().pc = 0;
    }

    #[inline]
    fn inc_pc(&mut self, amount: usize) -> Option<usize> {
        if let Some(last) = self.stack.last_mut() {
            last.inc_pc(amount)
        } else {
            None
        }
    }

    #[inline]
    fn dec_pc(&mut self, amount: usize) -> Option<usize> {
        if let Some(last) = self.stack.last_mut() {
            last.dec_pc(amount)
        } else {
            None
        }
    }

    fn top(&self) -> Option<CallFrame> {
        if let Some(last) = self.stack.last() {
            return Some(last.clone());
        } else {
            None
        }
    }

    fn next_from_top(&self) -> Option<CallFrame> {
        if self.stack.len() > 1 {
            let tmp = self.stack.len() - 2;
            let nxt = self.stack[tmp].clone();
            Some(nxt)
        } else {
            None
        }
    }

    fn push(&mut self, fun: Value) {
        self.stack.push(CallFrame::new(fun))
    }

    fn pop(&mut self) -> Option<CallFrame> {
        self.stack.pop()
    }

    fn clear(&mut self) {
        self.stack.clear();
    }

    #[allow(unused_assignments)]
    pub(crate) fn get_backtrace(&self) -> String {
        let mut string = format!("\nBacktrace:\n");

        let mut iter = self.stack.iter();
        loop {
            let elem = iter.next();
            if elem.is_none() {
                break;
            }

            let mut arity = 0usize;
            let mut name = String::from("");
            let mut fun_ty = FunctionType::NamedClosure;
            let mut fun_ty_name = String::from("");
            let filename = String::from("");
            if let Some(f) = elem {
                match f.fun {
                    Value::Closure(c) => {
                        name = c.get_core().get_name();
                        arity = c.get_core().get_arity();
                        fun_ty = c.get_core().get_type();
                        match fun_ty {
                            FunctionType::TopLevel => {
                                fun_ty_name = String::from("top-level");
                            }
                            FunctionType::NamedClosure => {
                                fun_ty_name = String::from("named closure");
                            }
                            FunctionType::AnonClosure => {
                                fun_ty_name = String::from("anonymous closure");
                            }
                            _ => {
                                unreachable!();
                            }
                        }
                    }
                    _ => {}
                }
                string = string
                    + &format!(
                        "[ {} {}(#{}) at {} line {} ]\n\n",
                        fun_ty_name, name, arity, filename, 0,
                    );
            }
        }
        string
    }
}

pub fn is_none(x: Value) -> bool {
    x == Value::Nil || (x.is_bool() && !x.as_bool())
}

macro_rules! read_byte {
    ($pc:ident, $code:ident, $out:ident) => {
        $pc += 1;
        let $out = $code[$pc - 1];
    };
}

macro_rules! read_word {
    ($pc:ident, $code:ident, $out:ident) => {
        $pc += 2;
        let $out = (($code[$pc - 2] as usize) << 8usize) | $code[$pc - 1] as usize;
    };
}

#[allow(dead_code)]
struct VM {
    vars: Vec<Value>,
    main_fun: Option<Managed<Closure>>, 
    calls: CallStack,
    stack: Vec<Value>,
    sp: usize,
}

#[allow(dead_code)]
impl VM {
    pub fn new() -> Self {
        Self {
            vars: Vec::<Value>::new(),
            main_fun: None,
            calls: CallStack::new(),
            stack: Vec::<Value>::new(),
            sp: 0usize,
        }
    } 

    #[inline]
    fn get_sp(&self) -> usize {
        self.sp
    }

    #[inline]
    fn set_sp(&mut self, index: usize) {
        self.sp = index;
    }

    #[inline]
    fn get_fp(&self) -> usize {
        self.calls.get_fp()
    }

    #[inline]
    fn set_fp(&mut self, index: usize) {
        self.calls.set_fp(index);
    }
    #[inline]
    fn get_pc(&mut self) -> Option<usize> {
        if let Some(frame) = self.calls.top() {
            Some(frame.get_pc())
        } else {
            None
        }
    }
    #[inline]
    fn inc_pc(&mut self, amount: usize) -> Option<usize> {
        if let Some(mut frame) = self.calls.top() {
            frame.inc_pc(amount)
        } else {
            None
        }
    }
    #[inline]
    fn dec_pc(&mut self, amount: usize) -> Option<usize> {
        if let Some(mut frame) = self.calls.top() {
            frame.dec_pc(amount)
        } else {
            None
        }
    }
    fn get_stack_at_index(&mut self, index: usize) -> Option<Value> {
        if let Some(val) = self.stack.get(index) {
            Some((*val).clone())
        } else {
            None
        }
    }

    fn get_stack_top(&mut self) -> Option<Value> {
        if let Some(val) = self.stack.last_mut() {
            Some((*val).clone())
        } else {
            None
        }
    }

    fn stack_push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn stack_pop(&mut self) {
        self.stack.pop();
    }

    fn get_var(&mut self, index: usize) -> Option<Value> {
        if let Some(val) = self.vars.get_mut(index) {
            Some((*val).clone())
        } else {
            None
        }
    }

    fn set_var(&mut self, index: usize, value: Value) {
        if let Some(val) = self.vars.get_mut(index) {
            *val = value;
        }
    }

    fn add_var(&mut self) {
        self.vars.push(Value::Nil);
    }

    fn push_call_frame(&mut self, num_args: usize) {
        let top = self.stack.len();
        let call = &self.stack[top - num_args as usize - 1];
        self.calls.push(call.clone());
        self.calls.set_fp(top - num_args as usize - 1);
        self.calls.clear_pc();
    }

    fn pop_call_frame(&mut self, mut fp: usize) -> usize {
        self.stack.resize(fp + 1, Value::Nil);

        if let Some(new_top) = self.calls.next_from_top() {
            fp = new_top.fp;
        } else if let Some(new_top) = self.calls.top() {
            fp = new_top.fp;
        }
        self.calls.pop();
        fp
    }

    fn check_arity(&self, fun: &Value, num_args: usize) -> Result<usize, String> {
        let top = self.stack.len();
        let mut arity = 0;
        match fun {
            Value::Closure(c) => {
                arity = c.get_arity();
            }
            _ => {
                return Err(format!("not a function type, but {}", fun.value_type()));
            }
        }
        if num_args != arity {
            Err(format!("{} args expected, found {}", arity, num_args))
        } else {
            Ok(num_args)
        }
    }

    fn call_value(&mut self, num_args: usize, debug: bool, gc: &Gc) -> Result<Value, String> {
        let fun_index = self.stack.len() - (num_args as usize) - 1;
        let fun = &self.stack[fun_index];
        let args = &self.stack[fun_index..fun_index + (num_args as usize) + 1];
        match self.check_arity(fun, num_args) {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }
        match fun {
            Value::Closure(_) => self.call_closure(num_args, gc, debug),
            _ => Err(format!("not a function type, but {}", fun.value_type())),
        }
    }

    #[allow(unused_assignments)]
    fn call_closure(&mut self, num_args: usize, gc: &Gc, debug: bool) -> Result<Value, String> {
        let mut pc = 0usize;

        if let Some(counter) = self.get_pc() {
            pc = counter;
        } else {
            panic!("no functions on call stack");
        }
        /*if debug {
            return call_closure_debug(self, u8, gc);
        }*/

        self.push_call_frame(num_args);

        let mut result = Ok(Value::Nil);
        let mut curr_byte = 0u8;
        let _sp = self.get_sp();
        let mut fp = self.get_fp();
        let mut closure: Managed<Closure> = self.stack[fp].as_closure();
        let fc = closure.get_core();
        let code: &[u8] = fc.get_code();

        let _sp = 0usize;
        while pc < fc.len() && result == Ok(Value::Nil) {
            closure = self.stack[fp].as_closure();
            fp = self.calls.get_fp();

            read_byte!(pc, code, byte);
            curr_byte = byte;

            let opcode = OpCode::from(curr_byte);

            #[cfg(feature = "debug_opcode")]
            {
                println!("Opcode {}:", opcode);
            }
            #[cfg(feature = "debug_vars")]
            {
                println!("");
                self.debug_vars();
            }
            #[cfg(feature = "debug_stack")]
            {
                println!("");
                self.debug_stack(fp);
            }

            result = match opcode {
                OpCode::PushN => {
                    read_byte!(pc, code, arg);
                    for _i in 0..arg {
                        self.stack.push(Value::Nil);
                    }

                    Ok(Value::Nil)
                }
                OpCode::Pop => {
                    self.stack_pop();
                    Ok(Value::Nil)
                }
                OpCode::PopN => {
                    read_byte!(pc, code, arg);
                    for _i in 0..arg {
                        self.stack_pop();
                    }

                    Ok(Value::Nil)
                }
                OpCode::Dup => {
                    let last = self.get_stack_top();
                    if let Some(i) = last {
                        self.stack.push(i.clone());
                    } else {
                        panic!("accessed stack from out of bounds");
                    }
                    Ok(Value::Nil)
                }
                OpCode::LoadTrue => {
                    self.stack.push(Value::Bool(true));
                    Ok(Value::Nil)
                }
                OpCode::LoadFalse => {
                    self.stack.push(Value::Bool(false));
                    Ok(Value::Nil)
                }
                OpCode::LoadNil => {
                    self.stack.push(Value::Nil);
                    Ok(Value::Nil)
                }
                OpCode::Load0
                | OpCode::Load1
                | OpCode::Load2
                | OpCode::Load3
                | OpCode::Load4
                | OpCode::Load5
                | OpCode::Load6
                | OpCode::Load7
                | OpCode::Load8
                | OpCode::Load9
                | OpCode::Load10 => {
                    self.stack.push(Value::Int(
                        (u8::from(opcode) - (u8::from(OpCode::Load0))) as i64,
                    ));
                    Ok(Value::Nil)
                }
                OpCode::LoadValue => {
                    read_word!(pc, code, arg);
                    self.stack
                        .push(closure.get_core().get_constants()[arg as usize].clone());
                    Ok(Value::Nil)
                }
                OpCode::Array => Ok(Value::Nil),
                OpCode::Dict => Ok(Value::Nil),
                OpCode::DefVar => {
                    read_word!(pc, code, index);
                    let top = self.get_stack_top();

                    if let Some(val) = top {
                        self.set_var(index as usize, val);
                    } else {
                        panic!("stack is empty");
                    }
                    Ok(Value::Nil)
                }
                OpCode::LoadVar => {
                    read_word!(pc, code, arg);
                    let elem = self.get_var(arg as usize).clone();
                    if let Some(val) = elem {
                        self.stack.push(val);
                    } else {
                        panic!(
                            "var index out of range: {}; limit: {}",
                            arg,
                            self.vars.len()
                        );
                    }

                    Ok(Value::Nil)
                }
                OpCode::StoreVar => {
                    read_word!(pc, code, arg);
                    if let Some(j) = self.get_stack_top() {
                        self.set_var(arg as usize, j.clone());
                    }
                    Ok(Value::Nil)
                }
                OpCode::LoadLocal => Ok(Value::Nil),
                OpCode::LoadLocal0 => Ok(Value::Nil),
                OpCode::LoadLocal1 => Ok(Value::Nil),
                OpCode::LoadLocal2 => Ok(Value::Nil),
                OpCode::LoadLocal3 => Ok(Value::Nil),
                OpCode::LoadLocal4 => Ok(Value::Nil),
                OpCode::LoadLocal5 => Ok(Value::Nil),
                OpCode::LoadLocal6 => Ok(Value::Nil),
                OpCode::LoadLocal7 => Ok(Value::Nil),
                OpCode::LoadLocal8 => Ok(Value::Nil),
                OpCode::StoreLocal => Ok(Value::Nil),
                OpCode::StoreLocal0 => Ok(Value::Nil),
                OpCode::StoreLocal1 => Ok(Value::Nil),
                OpCode::StoreLocal2 => Ok(Value::Nil),
                OpCode::StoreLocal3 => Ok(Value::Nil),
                OpCode::StoreLocal4 => Ok(Value::Nil),
                OpCode::StoreLocal5 => Ok(Value::Nil),
                OpCode::StoreLocal6 => Ok(Value::Nil),
                OpCode::StoreLocal7 => Ok(Value::Nil),
                OpCode::StoreLocal8 => Ok(Value::Nil),
                OpCode::LoadUpvalue => Ok(Value::Nil),
                OpCode::StoreUpvalue => Ok(Value::Nil),
                OpCode::LoadField => Ok(Value::Nil),
                OpCode::StoreField => Ok(Value::Nil),
                OpCode::LoadStatic => Ok(Value::Nil),
                OpCode::StoreStatic => Ok(Value::Nil),
                OpCode::LoadMethod => Ok(Value::Nil),
                OpCode::LoadStaticMethod => Ok(Value::Nil),
                OpCode::LoadSuperMethod => Ok(Value::Nil),
                OpCode::JumpFwd => {
                    read_word!(pc, code, arg);
                    pc += arg as usize;
                    Ok(Value::Nil)
                }
                OpCode::JumpBack => {
                    read_word!(pc, code, arg);
                    pc -= arg as usize;
                    Ok(Value::Nil)
                }
                OpCode::JumpTrue => {
                    read_word!(pc, code, arg);
                    let val = self.get_stack_top();
                    if let Some(i) = val {
                        if !is_none(i.clone()) {
                            pc += arg as usize;
                        }
                    }
                    Ok(Value::Nil)
                }
                OpCode::JumpFalse => {
                    read_word!(pc, code, arg);
                    let val = self.get_stack_top();
                    if let Some(i) = val {
                        if is_none(i.clone()) {
                            pc += arg as usize;
                        }
                    }
                    Ok(Value::Nil)
                }
                OpCode::DefStatic => Ok(Value::Nil),
                OpCode::Method => Ok(Value::Nil),
                OpCode::StaticMethod => Ok(Value::Nil),
                OpCode::Instance => Ok(Value::Nil),
                OpCode::Closure => Ok(Value::Nil),
                OpCode::CloseUpvalue => Ok(Value::Nil),
                OpCode::Call => {
                    read_byte!(pc, code, num_args);
                    println!("num_args = {}", num_args);

                    match self.call_value((num_args as usize) + 1, debug, gc) {
                        Ok(ref res) => {
                            let top = self.stack.len();
                            self.stack[top - (num_args as usize) - 1] = res.clone();
                            Ok(res.clone())
                        }
                        Err(e) => Err(e),
                    }
                }
                OpCode::Return => {
                    if let Some(val) = self.calls.top() {
                        fp = val.fp;
                    } else {
                        panic!("no calls left on the call stack");
                    }

                    let top = self.stack.len();
                    let last = self.stack[top - 1].clone();
                    self.stack[fp] = last.clone();
                    fp = self.pop_call_frame(fp);

                    Ok(last.clone())
                }
                OpCode::Ternary => Ok(Value::Nil),
                OpCode::Neg => {
                    if let Some(i) = self.get_stack_top() {
                        match i {
                            Value::Int(int) => {
                                let val: Value = Value::Int(-int);
                                let mut elem = self.get_stack_top();
                                if let Some(ref mut e) = elem {
                                    *e = val;
                                } else {
                                    panic!("stack is empty");
                                }
                                Ok(Value::Nil)
                            }
                            Value::Float(float) => {
                                let val: Value = Value::Float(-float);
                                let mut elem = self.get_stack_top();
                                if let Some(ref mut e) = elem {
                                    *e = val;
                                } else {
                                    panic!("stack is empty");
                                }
                                Ok(Value::Nil)
                            }
                            _ => Err("cannot negate value".to_string()),
                        }
                    } else {
                        panic!("stack is empty");
                    }
                }
                OpCode::Print => {
                    if let Some(val) = self.get_stack_top() {
                        print!("{}", val);
                    } else {
                        panic!("stack is empty");
                    }
                    Ok(Value::Nil)
                }
                OpCode::Println => {
                    if let Some(val) = self.get_stack_top() {
                        println!("{}", val);
                    } else {
                        panic!("stack is empty");
                    }
                    Ok(Value::Nil)
                }
                OpCode::Input => Ok(Value::Nil),
                OpCode::MathOp => {
                    read_byte!(pc, code, arg);
                    self.execute_math_op(MathOp::from(arg), gc)
                }
                OpCode::MathAssignOp => Ok(Value::Nil),
                OpCode::BitwiseOp => Ok(Value::Nil),
                OpCode::BitwiseAssignOp => Ok(Value::Nil),
                OpCode::Invalid => {
                    panic!("{}", "invalid instruction".to_string());
                }
            };
        }

        match result {
            Ok(ref res) => {
                let top = self.stack.len();
                self.stack[top - (num_args as usize) - 1] = res.clone();
                Ok(res.clone())
            }
            Err(e) => Err(e),
        }
    }

    /*fn call_closure_debug
        (
        &mut self,
        num_args: u8,
        gc: &Gc,
    )
        {
        self.push_call_frame(num_args);

        let mut result = Ok(Value::Nil);
        let mut curr_byte = 0u8;
        let _sp = self.get_sp();
        let mut fp = self.get_fp();
        let mut closure: Managed<Closure> = self.stack[fp].as_closure();
        let fc = closure.get_core();
        let code: &[u8] = fc.get_code();


        let _sp = 0usize;

        let mut debugger = Debugger::new();
        while pc < fc.len() && result == Ok(Value::Nil) {
        read_byte!(pc, code, byte);

        let opcode = OpCode::from(byte);

        let mut command = debugger.get_command() {

        result = match opcode {
        OpCode::PushN => {
        read_byte!(pc, code, arg);
        for _i in 0..arg {
        self.stack.push(Value::Nil);
    }

        Ok(Value::Nil)
    },
        OpCode::Pop => {
        self.stack_pop();
        Ok(Value::Nil)
    },
        OpCode::PopN => {
        read_byte!(pc, code, arg);
        for _i in 0..arg {
        self.stack_pop();
    }

        Ok(Value::Nil)
    },
        OpCode::Dup => {
        let last = self.get_stack_top();
        if let Some(i) = last {
        self.stack.push(i.clone());
    } else {
        panic!("accessed stack from out of bounds");
    }
        Ok(Value::Nil)
    },
        OpCode::LoadTrue => {
        self.stack.push(Value::Bool(true));
        Ok(Value::Nil)
    },
        OpCode::LoadFalse => {
        self.stack.push(Value::Bool(false));
        Ok(Value::Nil)
    },
        OpCode::LoadNil => {
        self.stack.push(Value::Nil);
        Ok(Value::Nil)
    },
        OpCode::Load0 |
        OpCode::Load1 |
        OpCode::Load2 |
        OpCode::Load3 |
        OpCode::Load4 |
        OpCode::Load5 |
        OpCode::Load6 |
        OpCode::Load7 |
        OpCode::Load8 |
        OpCode::Load9 |
        OpCode::Load10 => {
        self.stack.push(Value::Int((u8::from(opcode) - (u8::from(OpCode::Load0))) as i64));
        Ok(Value::Nil)
    },
        OpCode::LoadValue => {
        read_word!(pc, code, arg);
        self.stack.push(closure.get_core().get_constants()[arg as usize].clone());
        Ok(Value::Nil)
    },
        OpCode::Array => {
        Ok(Value::Nil)
    },
        OpCode::Dict => {
        Ok(Value::Nil)
    },
        OpCode::DefVar => {
        read_word!(pc, code, arg);
        let top = self.get_stack_top();

        if let Some(val) = top {
        self.add_var(val);
    } else {
        panic!("stack is empty");
    }
        Ok(Value::Nil)
    },
        OpCode::LoadVar => {
        read_word!(pc, code, arg);
        let elem = self.get_var(arg as usize).clone();
        if let Some(val) = elem {
        self.stack.push(val);
    } else {
        unreachable!();
    }

        Ok(Value::Nil)
    },
        OpCode::StoreVar => {
        read_word!(pc, code, arg);
        if let Some(j) = self.get_stack_top() {
        self.set_var(arg as usize, j.clone());
    }
        Ok(Value::Nil)
    },
        OpCode::LoadLocal => {
        Ok(Value::Nil)
    },
        OpCode::LoadLocal0 => {
        Ok(Value::Nil)
    },
        OpCode::LoadLocal1 => {
        Ok(Value::Nil)
    },
        OpCode::LoadLocal2 => {
        Ok(Value::Nil)
    },
        OpCode::LoadLocal3 => {
        Ok(Value::Nil)
    },
        OpCode::LoadLocal4 => {
        Ok(Value::Nil)
    },
        OpCode::LoadLocal5 => {
        Ok(Value::Nil)
    },
        OpCode::LoadLocal6 => {
        Ok(Value::Nil)
    },
        OpCode::LoadLocal7 => {
        Ok(Value::Nil)
    },
        OpCode::LoadLocal8 => {
        Ok(Value::Nil)
    },
        OpCode::StoreLocal => {
        Ok(Value::Nil)
    },
        OpCode::StoreLocal0 => {
        Ok(Value::Nil)
    },
        OpCode::StoreLocal1 => {
        Ok(Value::Nil)
    },
        OpCode::StoreLocal2 => {
        Ok(Value::Nil)
    },
        OpCode::StoreLocal3 => {
        Ok(Value::Nil)
    },
        OpCode::StoreLocal4 => {
        Ok(Value::Nil)
    },
        OpCode::StoreLocal5 => {
        Ok(Value::Nil)
    },
        OpCode::StoreLocal6 => {
        Ok(Value::Nil)
    },
        OpCode::StoreLocal7 => {
        Ok(Value::Nil)
    },
        OpCode::StoreLocal8 => {
        Ok(Value::Nil)
    },
        OpCode::LoadUpvalue => {
        Ok(Value::Nil)
    },
        OpCode::StoreUpvalue => {
        Ok(Value::Nil)
    },
        OpCode::LoadField => {
        Ok(Value::Nil)
    },
        OpCode::StoreField => {
        Ok(Value::Nil)
    },
        OpCode::LoadStatic => {
        Ok(Value::Nil)
    },
        OpCode::StoreStatic => {
        Ok(Value::Nil)
    },
        OpCode::LoadMethod => {
        Ok(Value::Nil)
    },
        OpCode::LoadStaticMethod => {
        Ok(Value::Nil)
    },
        OpCode::LoadSuperMethod => {
        Ok(Value::Nil)
    },
        OpCode::JumpFwd => {
        read_word!(pc, code, arg);
        pc += arg as usize;
        Ok(Value::Nil)
    },
        OpCode::JumpBack => {
        read_word!(pc, code, arg);
        pc -= arg as usize;
        Ok(Value::Nil)
    },
        OpCode::JumpTrue => {
        read_word!(pc, code, arg);
        let val = self.get_stack_top();
        if let Some(i) = val {
        if !is_none(i.clone()) {
        pc += arg as usize;
    }
    }
        Ok(Value::Nil)
    },
        OpCode::JumpFalse => {
        read_word!(pc, code, arg);
        let val = self.get_stack_top();
        if let Some(i) = val {
        if is_none(i.clone()) {
        pc += arg as usize;
    }
    }
        Ok(Value::Nil)
    },
        OpCode::DefStatic => {
        Ok(Value::Nil)
    },
        OpCode::Method => {
        Ok(Value::Nil)
    },
        OpCode::StaticMethod => {
        Ok(Value::Nil)
    },
        OpCode::Instance => {
        Ok(Value::Nil)
    },
        OpCode::Closure => {
        Ok(Value::Nil)
    },
        OpCode::CloseUpvalue => {
        Ok(Value::Nil)
    },
        OpCode::Call => {
        read_byte!(pc, code, num_args);
        println!("num_args = {}", num_args);


        match self.call_value(num_args, debug, gc) {
        Ok(ref res) => {
        let top = self.stack.len();
        self.stack[top - (num_args as usize) - 1] = res.clone();
        Ok(res.clone())
    },
        Err(e) => {
        Err(e)
    },
    }
    },
        OpCode::Return => {
        if let Some(val) = self.calls.top() {
        fp = val.fp;
    } else {
        panic!("no calls left on the call stack");
    }
        let
        let top = self.stack.len();
        let last = self.stack[top - 1].clone();
        self.stack[fp] = last.clone();
        fp = self.pop_call_frame(fp);

        Ok(last.clone())
    },
        OpCode::Ternary => {
        Ok(Value::Nil)
    },
        OpCode::Neg => {
        if let Some(i) = self.get_stack_top() {
        match i {
        Value::Int(int) => {
        let val: Value = Value::Int(-int);
        let mut elem = self.get_stack_top();
        if let Some(ref mut e) = elem {
         *e = val;
    } else {
        panic!("stack is empty");
    }
        Ok(Value::Nil)
    },
        Value::Float(float) => {
        let val: Value = Value::Float(-float);
        let mut elem = self.get_stack_top();
        if let Some(ref mut e) = elem {
         *e = val;
    } else {
        panic!("stack is empty");
    }
        Ok(Value::Nil)
    },
        _ => Err("cannot negate value".to_string())
    }
    } else {
        panic!("stack is empty");
    }
    },
        OpCode::Print => {
        if let Some(val) = self.get_stack_top() {
        print!("{}", val);
    } else {
        panic!("stack is empty");
    }
        Ok(Value::Nil)
    },
        OpCode::Println => {
        if let Some(val) = self.get_stack_top() {
        println!("{}", val);
    } else {
        panic!("stack is empty");
    }
        Ok(Value::Nil)
    },
        OpCode::Input => {
        Ok(Value::Nil)
    },
        OpCode::MathOp => {
        read_byte!(pc, code, arg);
        self.execute_math_op(MathOp::from(arg), gc)
    },
        OpCode::MathAssignOp => {
        Ok(Value::Nil)
    },
        OpCode::BitwiseOp => {
        Ok(Value::Nil)
    },
        OpCode::BitwiseAssignOp => {
        Ok(Value::Nil)
    },
        OpCode::Invalid => {
        panic!("{}", "invalid instruction".to_string());
    }
    };
    }

        match result {
        Ok(ref res) => {
        let top = self.stack.len();
        self.stack[top - (num_args as usize) - 1] = res.clone();
        Ok(res.clone())
    },
        Err(e) => {
        Err(e)
    },
    }
    }*/

    fn debug_vars(&self) {
        print!("NS Vars: [");
        for i in 0..self.vars.len() {
            print!("[{}]", self.vars[i]);
        }
        println!("]");
    }
    fn debug_stack(&self, fp: usize) {
        print!("[");
        for i in 0..self.stack.len() {
            print!("[");
            if i == fp {
                print!("> ");
            }
            print!("{}", self.stack[i]);
            if i == fp {
                print!(" <");
            }
            print!("]");
        }
        println!("]");
    }

    fn execute_math_op(&mut self, op: MathOp, gc: &Gc) -> Result<Value, String> {
        match op {
            MathOp::Add => self.execute_add(gc),
            MathOp::Subtract => self.execute_subtract(gc),
            MathOp::Multiply => self.execute_multiply(gc),
            MathOp::Divide => self.execute_divide(gc),
            MathOp::Modulo => self.execute_modulo(gc),
            MathOp::Power => self.execute_power(gc),
            _ => {
                panic!("invalid math operator");
            }
        }
    }

    fn execute_math_assign_op(&mut self, op: MathOp, gc: &Gc) -> Result<Value, String> {
        match op {
            MathOp::Add => self.execute_add_assign(gc),
            MathOp::Subtract => self.execute_subtract_assign(gc),
            MathOp::Multiply => self.execute_multiply_assign(gc),
            MathOp::Divide => self.execute_divide_assign(gc),
            MathOp::Modulo => self.execute_modulo_assign(gc),
            MathOp::Power => self.execute_power_assign(gc),
            _ => {
                panic!("invalid compound assignment operator");
            }
        }
    }

    fn execute_add(&mut self, gc: &Gc) -> Result<Value, String> {
        let arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(val1) => match arg2 {
                Value::Int(val2) => {
                    arg1 = Value::Int(val1 + val2);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::Float(val1) => match arg2 {
                Value::Float(val2) => {
                    arg1 = Value::Float(val1 + val2);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::String(val1) => match arg2 {
                Value::String(val2) => {
                    arg1 = Value::String(gc.manage(format!("{}{}", *val1, *val2), &NO_GC));
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "+: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute_subtract(&mut self, _gc: &Gc) -> Result<Value, String> {
        let arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(val1) => match arg2 {
                Value::Int(val2) => {
                    arg1 = Value::Int(val1 - val2);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::Float(val1) => match arg2 {
                Value::Float(val2) => {
                    arg1 = Value::Float(val1 - val2);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "-: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute_multiply(&mut self, gc: &Gc) -> Result<Value, String> {
        let arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(val1) => match arg2 {
                Value::Int(val2) => {
                    arg1 = Value::Int(val1 * val2);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::Float(val1) => match arg2 {
                Value::Float(val2) => {
                    arg1 = Value::Float(val1 * val2);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::String(val1) => match arg2 {
                Value::Int(val2) => {
                    arg1 = Value::String(gc.manage(val1.repeat(val2 as usize), &NO_GC));
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "*: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute_divide(&mut self, _gc: &Gc) -> Result<Value, String> {
        let arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(val1) => match arg2 {
                Value::Int(val2) => {
                    if val2 <= 0 {
                        return Err(String::from("division by zero"));
                    }
                    arg1 = Value::Int(val1 / val2);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::Float(val1) => match arg2 {
                Value::Float(val2) => {
                    if val2 <= OrderedFloat(0.0) {
                        return Err(String::from("division by zero"));
                    }
                    arg1 = Value::Float(val1 / val2);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "/: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute_modulo(&mut self, _gc: &Gc) -> Result<Value, String> {
        let arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(val1) => match arg2 {
                Value::Int(val2) => {
                    if val2 <= 0 {
                        return Err(String::from("modulo by zero"));
                    }
                    arg1 = Value::Int(val1 / val2);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "/: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute_power(&mut self, _gc: &Gc) -> Result<Value, String> {
        let arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(val1) => match arg2 {
                Value::Int(val2) => {
                    arg1 = Value::Int(val1.pow(val2 as u32));
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::Float(val1) => match arg2 {
                Value::Float(val2) => {
                    arg1 = Value::Float(val1.pow(val2));
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "**: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    // compound assignment math operators
    fn execute_add_assign(&mut self, gc: &Gc) -> Result<Value, String> {
        let mut arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(ref mut val1) => match arg2 {
                Value::Int(ref mut val2) => {
                    *val1 += *val2;
                    arg1 = Value::Int(*val1);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::Float(ref mut val1) => match arg2 {
                Value::Float(ref mut val2) => {
                    *val1 += *val2;
                    arg1 = Value::Float(*val1);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::String(ref mut val1) => match arg2 {
                Value::String(ref mut val2) => {
                    **val1 = format!("{}{}", **val1, **val2);
                    arg1 = Value::String(gc.manage(val1.to_string(), &NO_GC));
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "+=: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute_subtract_assign(&mut self, _gc: &Gc) -> Result<Value, String> {
        let mut arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(ref mut val1) => match arg2 {
                Value::Int(ref mut val2) => {
                    *val1 -= *val2;
                    arg1 = Value::Int(*val1);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::Float(ref mut val1) => match arg2 {
                Value::Float(ref mut val2) => {
                    *val1 -= *val2;
                    arg1 = Value::Float(*val1);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "-=: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute_multiply_assign(&mut self, gc: &Gc) -> Result<Value, String> {
        let mut arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(ref mut val1) => match arg2 {
                Value::Int(ref mut val2) => {
                    *val1 *= *val2;
                    arg1 = Value::Int(*val1);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::Float(ref mut val1) => match arg2 {
                Value::Float(val2) => {
                    *val1 *= *val2;
                    arg1 = Value::Float(*val1);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::String(ref mut val1) => match arg2 {
                Value::Int(ref mut val2) => {
                    **val1 = val1.repeat(*val2 as usize);
                    arg1 = Value::String(gc.manage(val1.clone().to_string(), &NO_GC));
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "*=: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute_divide_assign(&mut self, _gc: &Gc) -> Result<Value, String> {
        let mut arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(ref mut val1) => match arg2 {
                Value::Int(ref mut val2) => {
                    if *val2 <= 0 {
                        return Err(String::from("division by zero"));
                    }
                    *val1 /= *val2;
                    arg1 = Value::Int(*val1);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::Float(ref mut val1) => match arg2 {
                Value::Float(ref mut val2) => {
                    if *val2 <= OrderedFloat(0.0) {
                        return Err(String::from("division by zero"));
                    }
                    *val1 /= *val2;
                    arg1 = Value::Float(*val1);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "/=: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute_modulo_assign(&mut self, _gc: &Gc) -> Result<Value, String> {
        let mut arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(ref mut val1) => match arg2 {
                Value::Int(ref mut val2) => {
                    if *val2 <= 0 {
                        return Err(String::from("modulo by zero"));
                    }
                    *val1 %= *val2;
                    arg1 = Value::Int(*val1);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "%=: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute_power_assign(&mut self, _gc: &Gc) -> Result<Value, String> {
        let mut arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(ref mut val1) => match arg2 {
                Value::Int(ref mut val2) => {
                    arg1 = Value::Int(val1.pow(val1.pow(*val2 as u32) as u32));
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::Float(ref mut val1) => match arg2 {
                Value::Float(ref mut val2) => {
                    *val1 = val1.pow(*val2);
                    arg1 = Value::Float(*val1);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "**=: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute(
        &mut self,
        compiler: &Compiler,
        gc: &Gc,
        debug: bool,
        compile: bool,
    ) -> Result<Value, InterpretErrorType> {
        self.calls.clear();
        self.stack.clear();
        let closure = compiler.get_main();

        if let Some(f) = closure {
            self.main_fun = closure;
        } else if compile {
            let mut result: bool = false;
            result = compiler.compile_to_c();
            if !result {
                eprintln("C compile to file {} failed", compiler.c_file());
                return Err(InterpretErrorType::CompileTimeError);
            } else {
                eprintln("C compile complete!");
                return Ok(Value::Nil)
            }
        } else {
            return Err(InterpretErrorType::CompileTimeError);
        }

        // push main function onto the stack
        self.stack.push(Value::Closure(closure));

        self.push_call_frame(0usize);
        let result = self.call_closure(0usize, gc, debug);
        match result {
            Ok(val) => {
                return Ok(val);
            },
            Err(e) => {
                println!(
                    "\n\nRuntime error."
                );
                self.print_backtrace();
                println!("Error: {}", e);
                return Err(InterpretErrorType::RuntimeError);
            }
        } else {
            return Err(InterpretErrorType::CompileTimeError);
        }
    }

    pub(crate) fn run(
        &mut self,
        compiler: &Compiler,
        gc: &Gc,
        debug: bool,
        compile: bool,
    ) -> Result<Value, InterpretErrorType> {
        self.execute(compiler, gc, debug, compile)
    }

    pub fn print_backtrace(&self) {
        println!("{}", self.calls.get_backtrace());
    }
}







#[derive(Clone)]
struct Local {
    name: String,
    scope: usize,
    upvalue: bool,
    defined: bool,
}

impl Local {
    fn new(name: String, scope: usize, upvalue: bool) -> Self {
        Self {
            name,
            scope,
            upvalue,
            defined: false,
        }
    }
}

#[derive(Clone)]
struct Upvalue {
    local: bool,
    index: usize,
}

impl Upvalue {
    fn new(local: bool, index: usize) -> Self {
        Self { local, index }
    }
}

#[derive(Clone)]
struct FunCompiler {
    name: String,
    ty: FunctionType,
    scope: usize,
    arity: usize,

    locals: Vec<Local>,
    upvalues: Vec<Upvalue>,
    code: Vec<u8>,
    constants: HashMap<Value, usize>,
    constant_arr: Vec<Value>,
    local_fun_symtab: HashMap<String, VarInfo>,
    closure: Option<Managed<Closure>>,
}





#[derive(Clone, PartialEq)]
enum VarType {
    Var,
    Function,
    NamedClosure,
    AnonClosure,
    Constructor,
    Method,
    Class,
    Const,
}

#[derive(Clone, PartialEq)]
enum ScopeType {
    Ns,
    Local,
    Upvalue,
}

#[derive(Clone, PartialEq)]
enum VarErrorType {
    Undefined,
    AlreadyExists,
    TooMany,
    SelfInit,
    IsFunction,
    Other,
}



#[derive(Clone)]
struct VarError {
    name: String,
    error_type: VarErrorType,
}

impl VarError {
    fn new(name: String, error_type: VarErrorType) -> Self {
        Self { name, error_type }
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_error_type(&self) -> VarErrorType {
        self.error_type.clone()
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use crate::gc::{Gc, NO_GC};
    use crate::parser::interpret_string;

    #[test]
    fn test_add_ns_var() {
        let mut deps = prep_for_test();

        let mut code = "let var = 10;";
        let mut result = interpret_string(
            &mut code.to_string(),
            &mut deps.0,
            &mut deps.1,
            false,
            false,
            false,
        );
        assert_eq!(result, Ok(Value::Nil));

        code = "var;";
        result = interpret_string(
            &mut code.to_string(),
            &mut deps.0,
            &mut deps.1,
            false,
            false,
            false,
        );
        assert_eq!(result, Ok(Value::Int(10)));
    }

    fn prep_for_test() -> (VM, Gc) {
        (VM::new(), Gc::new())
    }
}
