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
enum Value {
    Struct { fields: Vec<FieldValue> },
    Enum { variant: Box<VariantValue> },
    String(String),
    Bool(bool),
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct FieldValue {
    name: String,
    value: Value,
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct V {
    name: String,
    value: String,
    children: Vec<Self>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct VariantValue {
    name: String,
    value: Value,
}

fn parse_v(input: &str) -> IResult<&str, V> {
    map(
        tuple((
            terminated(alphanumeric1, tag("=")),
            terminated(alphanumeric1, multispace0),
            opt(delimited(
                terminated(tag("{"), multispace0),
                separated_list(multispace1, parse_v),
                terminated(tag("}"), multispace0),
            )),
        )),
        |(name, value, children): (&str, &str, Option<Vec<V>>)| V {
            name: name.to_string(),
            value: value.to_string(),
            children: children.unwrap_or(vec![]),
        },
    )(input)
}

fn parse_field_value(field: &Field) -> impl Fn(&str) -> IResult<&str, FieldValue> {
    let field = field.clone();
    move |input: &str| {
        nom::combinator::map(
            nom::sequence::separated_pair(
                nom::character::complete::alphanumeric1,
                nom::bytes::complete::tag("="),
                parse_value(&field.schema),
            ),
            |(name, value): (&str, Value)| FieldValue {
                name: name.to_string(),
                value,
            },
        )(input)
    }
}

fn parse_value(schema: &Schema) -> Box<dyn Fn(&str) -> IResult<&str, Value>> {
    match schema {
        // Schema::Struct { fields } => Box::new(|input: &str| {
        //     nom::combinator::map(
        //         nom::sequence::delimited(
        //             nom::bytes::complete::tag("{"),
        //             // nom::multi::separated_list(tag(" "), parse_field_value),
        //             parse_field_value,
        //             nom::bytes::complete::tag("}"),
        //         ),
        //         |fields: FieldValue| Value::Struct { fields: vec![] },
        //     )(input)
        // }),
        Schema::String => Box::new(|input: &str| {
            nom::combinator::map(nom::character::complete::alphanumeric1, |v: &str| {
                Value::String(v.to_string())
            })(input)
        }),
        Schema::Bool => Box::new(|input: &str| {
            nom::combinator::map(nom::character::complete::alphanumeric1, |v: &str| {
                let v = v == "true";
                Value::Bool(v)
            })(input)
        }),
        _ => Box::new(|input: &str| {
            nom::combinator::map(nom::character::complete::alphanumeric1, |v: &str| {
                let v = v == "true";
                Value::Bool(v)
            })(input)
        }),
    }
}

fn print(value: &Value) -> String {
    match value {
        Value::Bool(v) => {
            if *v {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Value::String(v) => v.to_string(),
        Value::Struct { fields } => {
            let body = fields
                .iter()
                .map(|field| format!("{}={}", field.name, print(&field.value)))
                .collect::<Vec<_>>()
                .join(" ");
            format!("{{ {} }}", body)
        }
        Value::Enum { variant } => format!("{} {}", variant.name, print(&variant.value)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_v() {
        struct Test {
            string: String,
            value: V,
        };

        let tests = vec![
            Test {
                string: "foo=bar".to_string(),
                value: V {
                    name: "foo".to_string(),
                    value: "bar".to_string(),
                    children: vec![],
                },
            },
            Test {
                string: "foo=true".to_string(),
                value: V {
                    name: "foo".to_string(),
                    value: "true".to_string(),
                    children: vec![],
                },
            },
            Test {
                string: "foo=bar{bar=zoo}".to_string(),
                value: V {
                    name: "foo".to_string(),
                    value: "bar".to_string(),
                    children: vec![V {
                        name: "bar".to_string(),
                        value: "zoo".to_string(),
                        children: vec![],
                    }],
                },
            },
            Test {
                string: "foo=bar{ bar=zoo}".to_string(),
                value: V {
                    name: "foo".to_string(),
                    value: "bar".to_string(),
                    children: vec![V {
                        name: "bar".to_string(),
                        value: "zoo".to_string(),
                        children: vec![],
                    }],
                },
            },
            Test {
                string: "foo=bar { bar=zoo }".to_string(),
                value: V {
                    name: "foo".to_string(),
                    value: "bar".to_string(),
                    children: vec![V {
                        name: "bar".to_string(),
                        value: "zoo".to_string(),
                        children: vec![],
                    }],
                },
            },
            Test {
                string: "foo=bar111 { bar=zoo }".to_string(),
                value: V {
                    name: "foo".to_string(),
                    value: "bar111".to_string(),
                    children: vec![V {
                        name: "bar".to_string(),
                        value: "zoo".to_string(),
                        children: vec![],
                    }],
                },
            },
        ];

        for t in tests.iter() {
            // assert_eq!(t.string, print(&value));
            assert_eq!(Ok(("", t.value.clone())), parse_v(&t.string));
        }
    }

    #[test]
    fn test_parse_field_value() {
        struct Test {
            field: Field,
            string: String,
            value: FieldValue,
        };

        let tests = vec![
            Test {
                field: Field {
                    name: "foo".to_string(),
                    repeated: false,
                    schema: Schema::String,
                },
                string: "foo=bar".to_string(),
                value: FieldValue {
                    name: "foo".to_string(),
                    value: Value::String("bar".to_string()),
                },
            },
            Test {
                field: Field {
                    name: "foo".to_string(),
                    repeated: false,
                    schema: Schema::Bool,
                },
                string: "foo=true".to_string(),
                value: FieldValue {
                    name: "foo".to_string(),
                    value: Value::Bool(true),
                },
            },
        ];

        for t in tests.iter() {
            // assert_eq!(t.string, print(&value));
            assert_eq!(
                Ok(("", t.value.clone())),
                parse_field_value(&t.field)(&t.string)
            );
        }
    }

    // "foo=bar { aa=bb cc=dd }"
    // "foo=bar { aa=bb cc=dd }, boo { aa=bb uu=oo }"
    // "foo={ aa=bb cc=dd }"
    // "foo=bar aa=bb cc=dd"

    #[test]
    fn test_parse_value() {
        let schema = Schema::Enum {
            variants: vec![
                Variant {
                    name: "add".to_string(),
                    schema: Schema::Struct {
                        fields: vec![
                            Field {
                                name: "verbose".to_string(),
                                repeated: false,
                                schema: Schema::Bool,
                            },
                            Field {
                                name: "force".to_string(),
                                repeated: false,
                                schema: Schema::Bool,
                            },
                            Field {
                                name: "chmod".to_string(),
                                repeated: false,
                                schema: Schema::Bool,
                            },
                            Field {
                                name: "pathspec".to_string(),
                                repeated: true,
                                schema: Schema::Bool,
                            },
                        ],
                    },
                },
                Variant {
                    name: "diff".to_string(),
                    schema: Schema::Struct {
                        fields: vec![
                            Field {
                                name: "minimal".to_string(),
                                repeated: false,
                                schema: Schema::Bool,
                            },
                            Field {
                                name: "color".to_string(),
                                repeated: false,
                                schema: Schema::Bool,
                            },
                            Field {
                                name: "path".to_string(),
                                repeated: true,
                                schema: Schema::Bool,
                            },
                        ],
                    },
                },
            ],
        };
        let value = Value::Enum {
            variant: Box::new(VariantValue {
                name: "add".to_string(),
                value: Value::Struct {
                    fields: vec![
                        FieldValue {
                            name: "verbose".to_string(),
                            value: Value::Bool(true),
                        },
                        FieldValue {
                            name: "force".to_string(),
                            value: Value::Bool(false),
                        },
                    ],
                },
            }),
        };
        let value_string = "add { verbose=true force=false }";
        // assert_eq!(value_string, print(&value));
        // assert_eq!(Ok("", value), parse_value(&schema, value_string));
    }
}
