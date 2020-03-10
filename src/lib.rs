use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric1, multispace0, multispace1},
    combinator::{map, opt},
    multi::separated_list,
    sequence::{delimited, terminated, tuple},
    IResult,
};

#[derive(Debug, Eq, PartialEq, Clone)]
enum Schema {
    Struct { fields: Vec<Field> },
    Enum { variants: Vec<Variant> },
    String,
    Bool,
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Variant {
    name: String,
    schema: Schema,
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Field {
    name: String,
    repeated: bool,
    schema: Schema,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Entry {
    name: String,
    values: Vec<Value>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Value {
    value: String,
    children: Vec<Entry>,
}

pub fn parse_entry(input: &str) -> IResult<&str, Entry> {
    map(
        tuple((
            terminated(alphanumeric1, tag("=")),
            separated_list(terminated(tag(","), multispace0), parse_value),
        )),
        |(name, values): (&str, Vec<Value>)| Entry {
            name: name.to_string(),
            values,
        },
    )(input)
}

pub fn parse_value(input: &str) -> IResult<&str, Value> {
    map(
        tuple((
            terminated(alphanumeric1, multispace0),
            opt(delimited(
                terminated(tag("{"), multispace0),
                separated_list(multispace1, parse_entry),
                terminated(tag("}"), multispace0),
            )),
        )),
        |(value, children): (&str, Option<Vec<Entry>>)| Value {
            value: value.to_string(),
            children: children.unwrap_or(vec![]),
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_entry() {
        struct Test {
            string: String,
            value: Entry,
        };

        let tests = vec![
            Test {
                string: "foo=bar".to_string(),
                value: Entry {
                    name: "foo".to_string(),
                    values: vec![Value {
                        value: "bar".to_string(),
                        children: vec![],
                    }],
                },
            },
            Test {
                string: "foo=true".to_string(),
                value: Entry {
                    name: "foo".to_string(),
                    values: vec![Value {
                        value: "true".to_string(),
                        children: vec![],
                    }],
                },
            },
            Test {
                string: "foo=a,b".to_string(),
                value: Entry {
                    name: "foo".to_string(),
                    values: vec![
                        Value {
                            value: "a".to_string(),
                            children: vec![],
                        },
                        Value {
                            value: "b".to_string(),
                            children: vec![],
                        },
                    ],
                },
            },
            Test {
                string: "foo=bar{zoo=qat}".to_string(),
                value: Entry {
                    name: "foo".to_string(),
                    values: vec![Value {
                        value: "bar".to_string(),
                        children: vec![Entry {
                            name: "zoo".to_string(),
                            values: vec![Value {
                                value: "qat".to_string(),
                                children: vec![],
                            }],
                        }],
                    }],
                },
            },
            Test {
                string: "foo=bar{zoo=qat},xxx{aaa=bbb}".to_string(),
                value: Entry {
                    name: "foo".to_string(),
                    values: vec![
                        Value {
                            value: "bar".to_string(),
                            children: vec![Entry {
                                name: "zoo".to_string(),
                                values: vec![Value {
                                    value: "qat".to_string(),
                                    children: vec![],
                                }],
                            }],
                        },
                        Value {
                            value: "xxx".to_string(),
                            children: vec![Entry {
                                name: "aaa".to_string(),
                                values: vec![Value {
                                    value: "bbb".to_string(),
                                    children: vec![],
                                }],
                            }],
                        },
                    ],
                },
            },
            Test {
                string: "a=b{c=d{e=f g=h}i=j},k{l=m{n=o}}".to_string(),
                value: Entry {
                    name: "a".to_string(),
                    values: vec![
                        Value {
                            value: "b".to_string(),
                            children: vec![Entry {
                                name: "c".to_string(),
                                values: vec![Value {
                                    value: "d".to_string(),
                                    children: vec![Entry {
                                        name: "e".to_string(),
                                        values: vec![],
                                    }],
                                }],
                            }],
                        },
                        Value {
                            value: "k".to_string(),
                            children: vec![Entry {
                                name: "l".to_string(),
                                values: vec![Value {
                                    value: "m".to_string(),
                                    children: vec![Entry {
                                        name: "n".to_string(),
                                        values: vec![Value {
                                            value: "o".to_string(),
                                            children: vec![],
                                        }],
                                    }],
                                }],
                            }],
                        },
                    ],
                },
            },
            Test {
                string: "foo=bar{zoo=qat} , xxx{aaa=bbb}".to_string(),
                value: Entry {
                    name: "foo".to_string(),
                    values: vec![
                        Value {
                            value: "bar".to_string(),
                            children: vec![Entry {
                                name: "zoo".to_string(),
                                values: vec![Value {
                                    value: "qat".to_string(),
                                    children: vec![],
                                }],
                            }],
                        },
                        Value {
                            value: "xxx".to_string(),
                            children: vec![Entry {
                                name: "aaa".to_string(),
                                values: vec![Value {
                                    value: "bbb".to_string(),
                                    children: vec![],
                                }],
                            }],
                        },
                    ],
                },
            },
            Test {
                string: "foo=bar{ zoo=qat}".to_string(),
                value: Entry {
                    name: "foo".to_string(),
                    values: vec![Value {
                        value: "bar".to_string(),
                        children: vec![Entry {
                            name: "zoo".to_string(),
                            values: vec![Value {
                                value: "qat".to_string(),
                                children: vec![],
                            }],
                        }],
                    }],
                },
            },
            Test {
                string: "foo=bar { zoo=qat }".to_string(),
                value: Entry {
                    name: "foo".to_string(),
                    values: vec![Value {
                        value: "bar".to_string(),
                        children: vec![Entry {
                            name: "zoo".to_string(),
                            values: vec![Value {
                                value: "qat".to_string(),
                                children: vec![],
                            }],
                        }],
                    }],
                },
            },
            Test {
                string: "foo=bar111 { zoo=qat }".to_string(),
                value: Entry {
                    name: "foo".to_string(),
                    values: vec![Value {
                        value: "bar111".to_string(),
                        children: vec![Entry {
                            name: "zoo".to_string(),
                            values: vec![Value {
                                value: "qat".to_string(),
                                children: vec![],
                            }],
                        }],
                    }],
                },
            },
        ];

        for t in tests.iter() {
            // assert_eq!(t.string, print(&value));
            assert_eq!(Ok(("", t.value.clone())), parse_entry(&t.string));
        }
    }
}
