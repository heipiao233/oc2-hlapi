use serde::{Deserialize, Serialize};

/// Information associated with an imported file, containing the file's name and size in bytes.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct ImportFileInfo {
    /// The file's name.
    pub name: Box<str>,
    /// The file's size in bytes.
    pub size: u64,
}

/// A description of a device or interface block.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDescriptor {
    /// The device's UUID.
    pub device_id: uuid::Uuid,
    /// A list of strings that determine which kind of device the UUID refers to.
    pub type_names: Box<[Box<str>]>,
}

/// An RPC method signature and description.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MethodDescriptor {
    /// The method's name.
    pub name: Box<str>,
    /// The method's return type.
    pub return_type: Box<str>,
    /// Documentation about what the method does.
    pub description: Box<str>,
    /// A list of method parameters.
    pub parameters: Box<[ParameterDescriptor]>,
}

/// A description of one of a method's parameters, including the method name, description, and type.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct ParameterDescriptor {
    /// The parameter's name.
    pub name: Box<str>,
    /// Documentation about what the parameter is used for.
    pub description: Box<str>,
    #[serde(rename = "type")]
    /// The parameter's type.
    pub ty: Box<str>,
}

/// A block's relative direction.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Up,
    Down,
    Front,
    Back,
    Left,
    Right,
}

/// A rotation direction.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RotationDirection {
    Left,
    Right,
}

/// The state of a robot's current action.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RobotActionResult {
    Incomplete,
    Success,
    Failure,
}
