use crate::*;

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
    pub elemtype: String,
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
    pub fn parse_file(filename: impl AsRef<std::path::Path>) -> Result<Manifest, Error> {
        let r = std::fs::File::open(filename)?;
        let manifest = serde_json::from_reader(r)?;
        Ok(manifest)
    }

    pub fn print_c_functions(&self) {
        for (k, v) in &self.entry_points {
            println!("int futhark_entry_{}: {:?};", k, v);
        }
    }
}
