use crate::closure::*;
use crate::function_core::*;
use crate::gc::{Gc, NO_GC};
use crate::lexer::*;
use crate::token::*;
use crate::value::*;
use crate::vm::*;

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

struct Parser {
    gc: &Gc,
    lexer_stack: Vec<LexerData>,
    error_count: usize,
    //compiler: Compiler,
    in_panic_mode: bool,
    in_panic_lock_mode: bool,

}

impl<'a> Parser {
    fn new(gc: &Gc, debug: bool) -> Self {
        let mut s = Self {
            gc,
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

    pub fn parse_text(&self, text: String) -> Result<Value, InterpretErrorType> {
        self.lexer_stack = vec![];
        self.error_count = 0;
        self.in_panic_mode = false;
        self.in_panic_lock_mode = false;

        let lexer = Lexer::new_from_text(text);

        if lexer.is_err() {
            return Err(InterpretErrorType::ParseError)
        }

        self.lexer_stack.clear();
        self.lexer_stack.push(LexerData {
            lexer: lexer.unwrap(),
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

        let lexer = Some(Lexer::new_from_file(fname));

        if lexer.is_err() {
            return Err(InterpretErrorType::ParseError)
        }

        self.lexer_stack.push(LexerData {
            lexer: lexer.unwrap(),
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
                self.lexer_stack.last_mut().unwrap().lexer.scan(gc);
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
            self.lexer_stack.last_mut().unwrap().lexer.file_name(),
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
    fn parse(gc: &Gc) -> bool {
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
    fn synchronize_error(&mut self, gc: &Gc) -> bool {
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
            _ => false,
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
            _ => false,
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
            _ => Nonw,
        }
    }

    fn parse_stmt(&mut self, gc: &Gc) -> bool {
        let mut node = false;
        let tk = self.current().kind;
        let stmt = self.stmt_bp(tk.clone());
        if stmt.is_some() {
            node = self.call_stmt_fn(tk.clone(), false, gc);
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
        if prefix.is_false() {
            self.error_at_curr("expression expected".to_string());
            return false;
        }

        let mut l = self.call_prefix_fn(gc);

        tk = self.current().kind;
        let mut infix = self.infix_bp(tk);
        if infix.is_false() {
            return l;
        }
        while <BindingPower as Into<u8>>::into(bp.clone())
            <= <BindingPower as Into<u8>>::into(infix.unwrap())
        {
            tk = self.current().kind;
            infix = self.infix_bp(tk);
            if infix.is_false() {
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
        if e.is_false() {
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
        if if_expr.is_some() {
            self.advance(gc);
            let else_expr = self.parse_expr(if_expr, gc, BindingPower::Assignment);
            return else_expr;
        }
        false
    }

    // prefix operators

    fn parse_anon_closure(&mut self, gc: &Gc) -> bool {
        self.advance(gc);
        let mut params: Vec<Box<Ast>> = vec![];
        let mut code: Vec<Box<Ast>> = vec![];

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
            l = self.parse_stmt(l, gc);
            if let Some(v) = l {
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

    fn parse_named_closure(&mut self, gc: &Gc) -> Option<Box<Ast>> -> bool {
        self.advance(gc);
        let mut params: Vec<Box<Ast>> = vec![];
        let mut code: Vec<Box<Ast>> = vec![];

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

                let param = Box::new(Ast::Variable {
                    name: self.previous().lexeme.clone(),
                    arity: false,
                });
                params.push(param);
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
            l = self.parse_stmt(l, gc);
            if let Some(v) = l {
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

        if self.error_count > 0 {
            false;
        } else {
            true
        }

        
    }

    fn parse_return_stmt(&mut self, gc: &Gc) -> bool{
        self.advance(gc);
        let mut ret_expr: Option<Box<Ast>> = false;
        if !self.match_token(TokenKind::Semicolon, gc) {
            ret_expr = self.parse_expr(gc, BindingPower::Assignment);
            if ret_expr.is_false() {
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

        let mut list: Vec<Box<Ast>> = vec![];

        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Done) {
            let mut l: Option<Box<Ast>> = false;
            l = self.parse_stmt(l, gc);
            if let Some(i) = l {
                list.push(i.clone());
            } else {
                return false;
            }
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
        }

        true
    }

    fn parse_expr_stmt(&mut self, gc: &Gc) -> bool {
        // generate expression nodes
        let node = self.parse_expr(gc, BindingPower::Assignment);

        if node.is_false() {
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
        let mut arg_list: Vec<Box<String>> = vec![];
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

        if arg_list.is_false() {
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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

        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        if bp.is_some() {
            self.advance(gc);

            let right = self.parse_expr(
                left.clone(),
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
        if bp.is_false() {
            return false;
        }

        if bp.is_some() {
            self.advance(gc);

            right = self.parse_expr(
                left.clone(),
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
                left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
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
        if bp.is_false() {
            return false;
        }

        self.advance(gc);


        let right = self.parse_expr(
            left.clone(),
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

    fn prep_for_test() -> (VM, Gc) -> bool {
        (VM::new(), Gc::new())
    }
}


#[derive(Clone)]
pub struct SymbolTables {
    mods: Option<Rc<RefCell<Modules>>>,
    old_mods: Rc<RefCell<Modules>>,
}

impl SymbolTables {
    fn new() -> Self {
        Self {
            mods: Modules::new(),
        }
    }

    fn get_mod(&self, name: String) -> Result<Module, String> {
        let res = self.mods.get(name);
        if res.is_some() {
            Ok(res.unwrap())
        } else {
            Err(format!("Module '{}' doesn't exist", name))
        }
    }
}

pub struct Modules {
    mod_table: HashMap<String, Module>,
}

impl Modules {
    fn new(&self) -> Self {
        Self {
            mod_table: HashMap::<String, Module>::new(),
            mods: Vec::new(),
        }
    }

    fn define_mod(&self, name: String) -> Result<(), String> {
        if self.mod_table.len() == MAX_MODULES {
            Err(format!("Module limit exceeded for module '{}'", name))
        } else {
            let id = self.mod_table.len();
            self.mod_table.insert(name, id);
            Ok(())
        }
    }
}

type FuncEnv = Environment;

pub struct Module {
    mod_name: String,
    mod_index: usize,
    value: Option<Value>,
    mod_parent: Option<Module>,
    mod_vars: HashMap<String, VarEntry>,
    mod_env: Environment,
}

pub struct Environment {
    vars: Rc<RefCell<HashMap<String, VarEntry>>>,
    env_parent: Option<Rc<RefCell<Environment>>>,
}

pub struct VarEntry {
    name: String,
    var_type: VarType,
    var_index: usize,
    value: Option<Value>,
    var_mod: Option<Module>,
}

impl VarEntry {
    pub fn new(name: String, var_type: VarType, value: Option<Value>) -> Self {
        Self {
            name,
            var_type,
            value,
        }
    }

    pub fn get_name() -> String {
        name.clone()
    }

    pub fn get_type() -> VarType {
        var_type.clone()
    }

    pub fn get_value() -> Option<Value> {
        value.clone()
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
    ast: &mut Option<Box<Ast>>,
    vm: *mut VM,
    gc: &Gc,
) -> Option<Managed<Closure>> {
    if let Some(node) = ast {
        if let Some(last) = unsafe { (*vm).compiler.fc_vec.last_mut() } {
            match **node {
                Ast::Add(ref left, ref right) => {
                    self.code_gen(&mut left.clone(), vm, gc);
                    self.code_gen(&mut right.clone(), vm, gc);
                    last.emit_math_op(MathOp::Add);
                    last.closure
                }
                Ast::Subtract(ref left, ref right) => {
                    self.code_gen(&mut left.clone(), vm, gc);
                    self.code_gen(&mut right.clone(), vm, gc);
                    last.emit_math_op(MathOp::Subtract);
                    last.closure
                }
                Ast::Multiply(ref left, ref right) => {
                    self.code_gen(&mut left.clone(), vm, gc);
                    self.code_gen(&mut right.clone(), vm, gc);
                    last.emit_math_op(MathOp::Multiply);
                    last.closure
                }
                Ast::Divide(ref left, ref right) => {
                    self.code_gen(&mut left.clone(), vm, gc);
                    self.code_gen(&mut right.clone(), vm, gc);
                    last.emit_math_op(MathOp::Divide);
                    last.closure
                }
                Ast::Modulo(ref left, ref right) => {
                    self.code_gen(&mut left.clone(), vm, gc);
                    self.code_gen(&mut right.clone(), vm, gc);
                    last.emit_math_op(MathOp::Modulo);
                    last.closure
                }
                Ast::Power(ref left, ref right, ..) => {
                    self.code_gen(&mut left.clone(), vm, gc);
                    self.code_gen(&mut right.clone(), vm, gc);
                    last.emit_math_op(MathOp::Power);
                    last.closure
                }
                _ => {
                    unreachable!();
                }
            }
        } else {
            panic!("function compiler stack is empty");
        }
    } else {
        panic!("ast node doesn't exist");
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
