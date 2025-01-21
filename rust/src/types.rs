use std::collections::HashMap;
use std::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(HashMap<String, Json>),
}

#[derive(Debug)]
pub enum JC { // tokens for streaming JSON
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    BeginArray,
    EndArray,
    BeginObject,
    EndObject,
    Element(mpsc::Receiver<JC>), // of an array or object
}

#[derive(Debug, Clone, PartialEq)]
pub enum Accessor {
    Field(String, Box<Accessor>), // .s a
    Index(usize, Box<Accessor>), // [n] a
    Map(Box<Accessor>), // map a
    End,
}