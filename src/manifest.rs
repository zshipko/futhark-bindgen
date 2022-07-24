use crate::*;

#[derive(Clone, Debug, serde::Deserialize)]
pub enum ElemType {
    #[serde(rename = "i8")]
    I8,

    #[serde(rename = "i16")]
    I16,

    #[serde(rename = "i32")]
    I32,

    #[serde(rename = "i64")]
    I64,

    #[serde(rename = "u8")]
    U8,

    #[serde(rename = "u16")]
    U16,

    #[serde(rename = "u32")]
    U32,

    #[serde(rename = "u64")]
    U64,

    #[serde(rename = "f16")]
    F16,

    #[serde(rename = "f32")]
    F32,

    #[serde(rename = "f64")]
    F64,

    #[serde(rename = "bool")]
    Bool,
}

impl ElemType {
    pub fn to_str(&self) -> &'static str {
        match self {
            ElemType::I8 => "i8",
            ElemType::I16 => "i16",
            ElemType::I32 => "i32",
            ElemType::I64 => "i64",
            ElemType::U8 => "u8",
            ElemType::U16 => "u16",
            ElemType::U32 => "u32",
            ElemType::U64 => "u64",
            ElemType::F16 => "f16",
            ElemType::F32 => "f32",
            ElemType::F64 => "f64",
            ElemType::Bool => "bool",
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Output {
    pub r#type: String,
    pub unique: bool,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Input {
    pub name: String,
    pub r#type: String,
    pub unique: bool,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Entry {
    pub cfun: String,
    pub outputs: Vec<Output>,
    pub inputs: Vec<Input>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct ArrayOps {
    pub free: String,
    pub shape: String,
    pub values: String,
    pub new: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct ArrayType {
    pub ctype: String,
    pub rank: i32,
    pub elemtype: ElemType,
    pub ops: ArrayOps,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct OpaqueOps {
    pub free: String,
    pub store: String,
    pub restore: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct OpaqueType {
    pub ctypes: String,
    pub ops: OpaqueOps,
    // TODO: record field
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum Type {
    Array(ArrayType),
    Opaque(OpaqueType),
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Manifest {
    pub backend: Backend,
    pub version: String,
    pub entry_points: BTreeMap<String, Entry>,
    pub types: BTreeMap<String, Type>,
}

impl Manifest {
    pub fn from_source_file(filename: impl AsRef<std::path::Path>) -> Result<Manifest, Error> {
        let f = filename.as_ref().with_extension("json");
        Self::parse_file(f)
    }

    pub fn parse_file(filename: impl AsRef<std::path::Path>) -> Result<Manifest, Error> {
        let r = std::fs::File::open(filename)?;
        let manifest = serde_json::from_reader(r)?;
        Ok(manifest)
    }
}
