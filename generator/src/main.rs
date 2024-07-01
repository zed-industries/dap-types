use std::{collections::HashSet, path::PathBuf};

use indexmap::IndexMap;
use serde_json::{Map, Value};

fn main() {
    let schema = load_schema();
    let protocol_types = generate_protocol_types(&schema);
    let types = write_types(&protocol_types);
    let requests = write_requests(&protocol_types);
    let events = write_events(&protocol_types);
    write_file("types.rs", &types);
    write_file("requests.rs", &requests);
    write_file("events.rs", &events);
    println!("{types}");
}

#[test]
fn generated_files_are_up_to_date() {
    fn check_file(file: &str, contents: &str) {
        let want = with_disclaimer(contents);
        let got = std::fs::read_to_string(dst_path(file)).unwrap();
        assert!(want == got, "file {} is not up to date", file);
    }
    let schema = load_schema();
    let protocol_types = generate_protocol_types(&schema);
    let types = write_types(&protocol_types);
    let requests = write_requests(&protocol_types);
    let events = write_events(&protocol_types);
    check_file("types.rs", &types);
    check_file("requests.rs", &requests);
    check_file("events.rs", &events);
}

fn write_file(file: &str, contents: &str) {
    let contents = with_disclaimer(contents);
    std::fs::write(dst_path(file), contents).unwrap();
}

fn with_disclaimer(contents: &str) -> String {
    let disclaimer = "// This file is autogenerated. Do not edit by hand.\n// To regenerate from schema, run `cargo run -p generator`.\n\n";
    disclaimer.to_owned() + contents
}

fn dst_path(file: &str) -> PathBuf {
    let workspace_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let mut path = workspace_dir.to_owned();
    path.push("dap-types");
    path.push("src");
    path.push(file);
    path
}

fn load_schema() -> Value {
    let workspace_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let mut schema_path = workspace_dir.to_owned();
    schema_path.push("schema.json");
    let contents = std::fs::read_to_string(&schema_path).unwrap();
    serde_json::from_str(&contents).unwrap()
}

const BLACKLISTED_TYPES: &[&str] = &[
    "ProtocolMessage",
    "Request",
    "Event",
    "Response",
    "ErrorResponse",
    "RestartArguments",
    "LaunchRequestArguments",
    "AttachRequestArguments",
];

fn write_requests(types: &[ProtocolType]) -> String {
    let mut writer = Writer::default();
    writer.line("use serde::{de::DeserializeOwned, Serialize};");
    writer.line("use std::fmt::Debug;");
    writer.finished_object();
    writer.code(REQUEST_TRAIT);
    for ty in types {
        let Type::Object(o) = &ty.ty else {
            continue;
        };
        let Some(type_field) = o.find_field("type") else {
            continue;
        };
        let Type::Enum(e) = &type_field.ty else {
            continue;
        };
        if e.variants.len() != 1 || e.variants[0] != "request" || !e.exhaustive {
            continue;
        }
        let command = o.find_field("command").unwrap().ty.as_enum().single_value();
        let arguments = match &o.find_field("arguments").unwrap().ty {
            Type::Any => "()".to_owned(),
            Type::Basic(args) => format!("crate::{args}"),
            _ => panic!("bad arguments type for {}", ty.name),
        };
        let request = ty.name.strip_suffix("Request").unwrap();
        let response = format!("{request}Response");
        let response_ty = types.iter().find(|t| t.name == response).unwrap();
        let ro = response_ty.ty.as_object();
        let response_body = match &ro.find_field("body").unwrap().ty {
            Type::Any => "()".to_owned(),
            Type::Basic(body) => format!("crate::{body}"),
            Type::Object(_) => format!("crate::{response}"),
            _ => panic!("bad response body for {}", ty.name),
        };
        writer.doc(o.doc.as_ref().unwrap());
        writer.line(format!("pub enum {request} {{}}"));
        writer.finished_object();
        writer.line(format!("impl Request for {request} {{"));
        writer.indented(format!("const COMMAND: &'static str = {command:?};"));
        writer.indented(format!("type Arguments = {arguments};"));
        writer.indented(format!("type Response = {response_body};"));
        writer.line("}");
        writer.finished_object();
    }
    writer.output
}

fn write_events(types: &[ProtocolType]) -> String {
    let mut writer = Writer::default();
    writer.line("use serde::{de::DeserializeOwned, Serialize};");
    writer.line("use std::fmt::Debug;");
    writer.finished_object();
    writer.code(EVENT_TRAIT);
    for ty in types {
        let Type::Object(o) = &ty.ty else {
            continue;
        };
        let Some(type_field) = o.find_field("type") else {
            continue;
        };
        let Type::Enum(e) = &type_field.ty else {
            continue;
        };
        if e.variants.len() != 1 || e.variants[0] != "event" || !e.exhaustive {
            continue;
        }
        let name = o.find_field("event").unwrap().ty.as_enum().single_value();

        let body = match &o.find_field("body").unwrap().ty {
            Type::Any => {
                if name == "initialized" {
                    format!("Option<crate::Capabilities>")
                } else {
                    "()".to_owned()
                }
            }
            Type::Basic(args) => format!("crate::{args}"),
            Type::Object(_) => format!("crate::{}", ty.name),
            _ => panic!("bad body type for {}", ty.name),
        };
        let event = ty.name.strip_suffix("Event").unwrap();
        writer.doc(o.doc.as_ref().unwrap());
        writer.line(format!("pub enum {event} {{}}"));
        writer.finished_object();
        writer.line(format!("impl Event for {event} {{"));
        writer.indented(format!("const EVENT: &'static str = {name:?};"));
        writer.indented(format!("type Body = {body};"));
        writer.line("}");
        writer.finished_object();
    }
    writer.output
}

fn write_types(types: &[ProtocolType]) -> String {
    let mut writer = Writer::default();
    writer.line("use serde::{Deserialize, Serialize};");
    writer.finished_object();
    for ty in types {
        if ty.name.ends_with("Request") {
            continue;
        }
        println!("writing type {}", ty.name);
        if ty.name.ends_with("Response") {
            let body = &ty.ty.as_object().find_field("body").unwrap().ty;
            match body {
                Type::Any => continue,
                Type::Object(o) => {
                    let mut o = o.clone();
                    o.doc = o.doc.or(ty.ty.doc());
                    o.write(&ty.name, &mut writer);
                }
                Type::Basic(_) => continue,
                _ => panic!(),
            }
        } else if ty.name.ends_with("Event") {
            let body = &ty.ty.as_object().find_field("body").unwrap().ty;
            match body {
                Type::Any => continue,
                Type::Object(o) => {
                    let mut o = o.clone();
                    o.doc = o.doc.or(ty.ty.doc());
                    o.write(&ty.name, &mut writer);
                }
                Type::Basic(_) => continue,
                _ => panic!(),
            }
        } else {
            ty.write(&mut writer);
        }
    }
    writer.code(CUSTOM_TYPES);
    writer.output
}

fn generate_protocol_types(schema: &Value) -> Vec<ProtocolType> {
    let defs = schema.get("definitions").unwrap().as_object().unwrap();
    let mut types = Vec::new();
    for (name, def) in defs {
        if BLACKLISTED_TYPES.contains(&name.as_str()) {
            continue;
        }
        println!("generating {name}");
        types.push(ProtocolType {
            name: name.to_owned(),
            ty: translate_type(defs, def),
        });
    }
    types
}

fn translate_all_of(defs: &Map<String, Value>, def: &Value) -> Object {
    assert_eq!(def.as_object().unwrap().len(), 1);
    let members = def.get("allOf").unwrap().as_array().unwrap();
    let mut fields = IndexMap::new();
    let mut doc = None;
    for subobject in members {
        let subobject = if let Some(r) = subobject.get("$ref") {
            let r = r.as_str().unwrap().strip_prefix("#/definitions/").unwrap();
            match translate_type(defs, &defs[r]) {
                Type::Object(o) => o,
                _ => todo!(),
            }
        } else {
            translate_object(defs, subobject)
        };
        for f in subobject.fields {
            fields.insert(f.name.clone(), f);
        }
        doc = subobject.doc.or(doc);
    }
    Object {
        doc,
        fields: fields.into_iter().map(|x| x.1).collect(),
    }
}

fn translate_object(defs: &Map<String, Value>, def: &Value) -> Object {
    assert_eq!(
        def.get("type")
            .expect("has type")
            .as_str()
            .expect("type is string"),
        "object"
    );
    let required = def
        .get("required")
        .map(|r| {
            r.as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_str().unwrap())
                .collect::<HashSet<&str>>()
        })
        .unwrap_or_default();
    let mut fields = IndexMap::new();
    if let Some(properties) = def.get("properties") {
        for (name, field) in properties.as_object().unwrap().iter() {
            let field = generate_field(defs, name, field, required.contains(name.as_str()));
            fields.insert(name.to_owned(), field);
        }
    }
    Object {
        doc: def
            .get("description")
            .map(|x| x.as_str().unwrap().to_owned()),
        fields: fields.into_iter().map(|x| x.1).collect(),
    }
}

fn generate_field(defs: &Map<String, Value>, name: &str, def: &Value, required: bool) -> Field {
    Field {
        doc: def
            .get("description")
            .map(|x| x.as_str().unwrap().to_owned()),
        name: name.to_owned(),
        ty: translate_type(defs, def),
        required,
    }
}

fn translate_type(defs: &Map<String, Value>, t: &Value) -> Type {
    if is_any(t) {
        return Type::Any;
    }
    if is_enum_of(t, ["string", "null"]) {
        return Type::Option(Box::new("String".into()));
    }
    if is_enum_of(t, ["integer", "string"]) {
        return "ModuleId".into();
    }
    if let Some(r) = t.get("$ref") {
        let r = r.as_str().unwrap().strip_prefix("#/definitions/").unwrap();
        return r.into();
    }
    if t.get("allOf").is_some() {
        return Type::Object(translate_all_of(defs, t));
    }
    let ty = t.get("type").and_then(|x| x.as_str()).unwrap_or_else(|| {
        panic!("failed to find type on {}", t);
    });
    match ty {
        "integer" => "u64".into(),
        "number" => "u64".into(),
        "boolean" => "bool".into(),
        "string" => {
            let doc = t.get("description").map(|x| x.as_str().unwrap().to_owned());
            let variant_descriptions = t.get("enumDescriptions").map(|x| {
                x.as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_str().unwrap().to_owned())
                    .collect::<Vec<_>>()
            });
            if let Some(values) = t.get("enum") {
                Type::Enum(Enum {
                    doc,
                    variants: values
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap().to_owned())
                        .collect(),
                    exhaustive: true,
                    variant_descriptions,
                })
            } else if let Some(values) = t.get("_enum") {
                Type::Enum(Enum {
                    doc,
                    variants: values
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap().to_owned())
                        .collect(),
                    exhaustive: false,
                    variant_descriptions,
                })
            } else {
                "String".into()
            }
        }
        "object" => {
            if t.get("properties").is_none() && t.get("additionalProperties").is_some() {
                Type::Any
            } else {
                Type::Object(translate_object(defs, t))
            }
        }
        "array" => {
            let item = translate_type(defs, t.get("items").unwrap());
            Type::Vec(Box::new(item))
        }
        other => other.into(),
    }
}

fn is_any(t: &Value) -> bool {
    is_enum_of(
        t,
        [
            "array", "boolean", "integer", "null", "number", "object", "string",
        ],
    )
}

fn is_enum_of<const N: usize>(t: &Value, values: [&str; N]) -> bool {
    let Some(arr) = t.get("type").and_then(|x| x.as_array()) else {
        return false;
    };
    if arr.len() != values.len() {
        return false;
    }
    arr.iter().zip(values).all(|(a, b)| a == b)
}

fn to_snake_case(raw: &str) -> String {
    if raw == "type" {
        return "type_".to_owned();
    }
    words(raw)
        .into_iter()
        .map(|w| format!("_{w}"))
        .collect::<String>()[1..]
        .to_owned()
}

fn to_pascal_case(raw: &str) -> String {
    words(raw)
        .into_iter()
        .map(|w| {
            w.chars()
                .take(1)
                .flat_map(|c| c.to_uppercase())
                .chain(w.chars().skip(1))
                .collect::<String>()
        })
        .collect()
}

fn words(raw: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut last = String::new();
    let mut prev_upper = false;
    for c in raw.chars() {
        if c == '_' || c == ' ' {
            result.push(std::mem::take(&mut last));
        } else if c.is_uppercase() && !prev_upper {
            result.push(std::mem::take(&mut last));
            last.extend(c.to_lowercase());
        } else {
            last.extend(c.to_lowercase());
        }
        prev_upper = c.is_uppercase();
    }
    result.push(last);
    result.retain(|x| !x.is_empty());
    result
}

#[derive(Default)]
struct Writer {
    output: String,
    finished_object: bool,
}

impl Writer {
    fn check_finish(&mut self) {
        if self.finished_object {
            self.output.push_str("\n");
            self.finished_object = false;
        }
    }

    fn line(&mut self, line: impl AsRef<str>) {
        self.check_finish();
        self.output.push_str(line.as_ref().trim_end());
        self.output.push_str("\n");
    }

    fn indented(&mut self, line: impl AsRef<str>) {
        self.check_finish();
        self.output.push_str("    ");
        self.output.push_str(line.as_ref().trim_end());
        self.output.push_str("\n");
    }

    fn finished_object(&mut self) {
        self.finished_object = true;
    }

    fn doc(&mut self, doc: impl AsRef<str>) {
        for line in doc.as_ref().lines() {
            self.line(format!("/// {line}"));
        }
    }

    fn indented_doc(&mut self, doc: impl AsRef<str>) {
        for line in doc.as_ref().lines() {
            self.indented(format!("/// {line}"));
        }
    }

    fn code(&mut self, code: &str) {
        for line in code.lines() {
            if line == "" {
                self.finished_object();
                continue;
            }
            if let Some(indented) = line.strip_prefix("    ") {
                self.indented(indented);
            } else {
                self.line(line);
            }
        }
        self.finished_object();
    }
}

struct ProtocolType {
    name: String,
    ty: Type,
}

#[derive(Clone)]
enum Type {
    Any,
    Basic(String),
    Enum(Enum),
    Object(Object),
    Vec(Box<Type>),
    Option(Box<Type>),
}

impl Type {
    #[track_caller]
    fn as_enum(&self) -> &Enum {
        match self {
            Type::Enum(e) => e,
            _ => panic!("not an enum"),
        }
    }

    #[track_caller]
    fn as_object(&self) -> &Object {
        match self {
            Type::Object(o) => o,
            _ => panic!("not an object"),
        }
    }

    fn doc(&self) -> Option<String> {
        match self {
            Type::Any | Type::Basic(_) | Type::Vec(_) | Type::Option(_) => None,
            Type::Enum(x) => x.doc.clone(),
            Type::Object(x) => x.doc.clone(),
        }
    }
}

impl Enum {
    fn single_value(&self) -> &str {
        assert!(self.variants.len() == 1);
        assert!(self.exhaustive);
        &self.variants[0]
    }
}

impl From<&str> for Type {
    fn from(value: &str) -> Self {
        Type::Basic(value.to_owned())
    }
}

#[derive(Clone)]
struct Enum {
    doc: Option<String>,
    variants: Vec<String>,
    exhaustive: bool,
    variant_descriptions: Option<Vec<String>>,
}

#[derive(Clone)]
struct Object {
    doc: Option<String>,
    fields: Vec<Field>,
}

#[derive(Clone)]
struct Field {
    doc: Option<String>,
    name: String,
    ty: Type,
    required: bool,
}

impl ProtocolType {
    fn write(&self, dst: &mut Writer) {
        match &self.ty {
            Type::Any => todo!(),
            Type::Basic(_) => todo!(),
            Type::Enum(e) => e.write(&self.name, dst),
            Type::Object(o) => o.write(&self.name, dst),
            Type::Vec(_) => todo!(),
            Type::Option(_) => todo!(),
        }
    }
}

impl Object {
    fn write(&self, name: &str, dst: &mut Writer) {
        if let Some(doc) = &self.doc {
            dst.doc(&doc);
        }
        dst.line("#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]");
        let mut pending = Vec::new();
        if self.fields.is_empty() {
            dst.line(format!("pub struct {};", name));
        } else {
            dst.line(format!("pub struct {} {{", name));
            for field in &self.fields {
                let inline_name = format!("{}{}", name, to_pascal_case(&field.name));
                let ty = field.ty.stringify(inline_name, &mut pending);
                if let Some(doc) = &field.doc {
                    dst.indented_doc(doc);
                }
                dst.indented(format!("#[serde(rename = \"{}\")]", field.name));
                let clean_name = to_snake_case(&field.name);
                if field.required {
                    dst.indented(format!("pub {}: {},", clean_name, ty));
                } else {
                    dst.indented("#[serde(skip_serializing_if = \"Option::is_none\")]");
                    dst.indented("#[serde(default)]");
                    dst.indented(format!("pub {}: Option<{}>,", clean_name, ty));
                }
            }
            dst.line("}");
        }
        dst.finished_object();
        for p in pending {
            p.write(dst);
        }
    }

    fn find_field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name == name)
    }
}

impl Enum {
    fn write(&self, name: &str, dst: &mut Writer) {
        if let Some(doc) = &self.doc {
            dst.doc(doc);
        }
        if self.exhaustive {
            dst.line("#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy, Deserialize, Serialize)]");
        } else {
            dst.line("#[derive(PartialEq, Eq, Debug, Hash, Clone, Deserialize, Serialize)]");
        }
        if !self.exhaustive {
            dst.line("#[non_exhaustive]");
        }
        dst.line(format!("pub enum {} {{", name));
        for (i, value) in self.variants.iter().enumerate() {
            if let Some(desc) = &self.variant_descriptions {
                assert!(desc.len() == self.variants.len());
                dst.indented_doc(&desc[i]);
            }
            dst.indented(format!("#[serde(rename = \"{value}\")]"));
            dst.indented(format!("{},", to_pascal_case(value)));
        }
        if !self.exhaustive {
            dst.indented("#[serde(other)]");
            dst.indented("Unknown,");
        }
        dst.line("}");
        dst.finished_object();
    }
}

impl Type {
    fn stringify(&self, inline_name: String, pending: &mut Vec<PendingInline>) -> String {
        match self {
            Type::Any => "serde_json::Value".to_owned(),
            Type::Basic(x) => x.clone(),
            Type::Enum(e) => {
                pending.push(PendingInline::Enum {
                    name: inline_name.clone(),
                    e: e.clone(),
                });
                inline_name
            }
            Type::Object(o) => {
                pending.push(PendingInline::Object {
                    name: inline_name.clone(),
                    o: o.clone(),
                });
                inline_name
            }
            Type::Vec(x) => format!("Vec<{}>", x.stringify(inline_name, pending)),
            Type::Option(x) => format!("Option<{}>", x.stringify(inline_name, pending)),
        }
    }
}

enum PendingInline {
    Enum { name: String, e: Enum },
    Object { name: String, o: Object },
}

impl PendingInline {
    fn write(&self, dst: &mut Writer) {
        match self {
            PendingInline::Enum { name, e } => {
                e.write(name, dst);
            }
            PendingInline::Object { name, o } => {
                o.write(name, dst);
            }
        }
    }
}

const CUSTOM_TYPES: &str = "
#[derive(PartialEq, Eq, Debug, Hash, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ModuleId {
    Number(u32),
    String(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct AttachRequestArguments {
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct LaunchRequestArguments {
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct RestartArguments {
    pub raw: serde_json::Value,
}
";

const REQUEST_TRAIT: &str = "
/// Request is a request, with associated command, and argument and response types.
pub trait Request {
    const COMMAND: &'static str;
    type Arguments: Debug + Clone + Serialize + DeserializeOwned + Send + Sync;
    type Response: Debug + Clone + Serialize + DeserializeOwned + Send + Sync;
}
";

const EVENT_TRAIT: &str = "
/// Event is an event, with associated name and body type.
pub trait Event {
    const EVENT: &'static str;
    type Body: Debug + Clone + Serialize + DeserializeOwned + Send + Sync;
}
";
