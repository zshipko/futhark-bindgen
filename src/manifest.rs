use crate::*;

/// Scalar types
#[derive(Clone, Debug, serde::Deserialize)]
pub enum ElemType {
    /// Signed 8 bit integer
    #[serde(rename = "i8")]
    I8,

    /// Signed 16 bit integer
    #[serde(rename = "i16")]
    I16,

    /// Signed 32 bit integer
    #[serde(rename = "i32")]
    I32,

    /// Signed 64 bit integer
    #[serde(rename = "i64")]
    I64,

    /// Unsigned 8 bit integer
    #[serde(rename = "u8")]
    U8,

    /// Unsigned 16 bit integer
    #[serde(rename = "u16")]
    U16,

    /// Unsigned 32 bit integer
    #[serde(rename = "u32")]
    U32,

    /// Unsigned 64 bit integer
    #[serde(rename = "u64")]
    U64,

    /// 16 bit float
    #[serde(rename = "f16")]
    F16,

    /// 32 bit float
    #[serde(rename = "f32")]
    F32,

    /// 64 bit float
    #[serde(rename = "f64")]
    F64,

    /// Boolean
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
pub struct Field {
    pub name: String,
    pub project: String,
    pub r#type: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Record {
    pub new: String,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct OpaqueType {
    pub ctype: String,
    pub ops: OpaqueOps,
    pub record: Option<Record>,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(tag = "kind")]
pub enum Type {
    #[serde(rename = "array")]
    Array(ArrayType),
    #[serde(rename = "opaque")]
    Opaque(OpaqueType),
}

/// A Rust encoding of the Futhark manifest file
#[derive(Clone, Debug, serde::Deserialize)]
pub struct Manifest {
    pub backend: Backend,
    pub version: String,
    pub entry_points: BTreeMap<String, Entry>,
    pub types: BTreeMap<String, Type>,
}

impl Manifest {
    /// Parse the manifest file
    pub fn parse_file(filename: impl AsRef<std::path::Path>) -> Result<Manifest, Error> {
        let r = std::fs::File::open(filename)?;
        let manifest = serde_json::from_reader(r)?;
        Ok(manifest)
    }
}
