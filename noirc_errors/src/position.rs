use codespan::{Span as ByteSpan,ByteIndex};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position{
    line : usize,
    column : usize,
}

impl Default for Position {
    fn default() -> Self{
        Position{
            line: 0,
            column: 0
        }
    }
}

impl Position {
    pub fn new_line(&mut self) {
        self.line +=1;
        self.column = 0;
    }
    pub fn right_shift(&mut self) {
        self.column +=1;
    }

    pub fn mark(&self) -> Position {
        self.clone()
    }
    pub fn forward(self) -> Position{
        self.forward_by(1)
    }
    pub fn backward(self) -> Position{
        self.backward_by(1)
    }
    
    pub fn into_span(self) -> Span{
        Span{
            start : self,
            end : self,
        }
    }

    pub fn backward_by(self, amount : usize) -> Position {
        Position {
            line : self.line,
            column : self.column - amount
        }
    }
    pub fn forward_by(self, amount : usize) -> Position {
        Position {
            line : self.line,
            column : self.column + amount
        }
    }
    pub fn to_byte_index(self) -> ByteIndex {
        ByteIndex((self.column * self.line) as u32)
    }

}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Spanned<T> {
    pub contents : T,
    span : Span,
}

impl<T> Spanned<T> {
    
    pub fn from(start : Position, end : Position, contents : T) -> Spanned<T> {
        Spanned {
            span : Span{
                start, end
            },contents
        }
    }
    
    pub fn span(&self) -> Span {
        self.span()
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Span {
    // file_id : usize,
    pub start : Position,
    pub end : Position,
}

impl Span {
    pub fn merge(self, other : Span) -> Span {
        
        // assert_eq!(self.file_id, other.file_id);
        // let file_id = self.file_id;

        use std::cmp::{max, min};

        let start = min(self.start, other.start);
        let end = max(self.end, other.end);
        Span{start,end}
    }
    pub fn to_byte_span(self) -> ByteSpan {
        ByteSpan::from(self.start.to_byte_index() .. self.end.to_byte_index())
    }
}