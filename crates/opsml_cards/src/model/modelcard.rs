use crate::model::error::interface_error;
use crate::utils::BaseArgs;
use chrono::{DateTime, Utc};
use opsml_crypt::decrypt_directory;
use opsml_error::{
    error::{CardError, OpsmlError},
    map_err_with_logging,
};
use opsml_interfaces::ModelInterface;
use opsml_interfaces::OnnxSession;
use opsml_interfaces::{
    CatBoostModel, HuggingFaceModel, LightGBMModel, LightningModel, SklearnModel, TorchModel,
    XGBoostModel,
};
use opsml_interfaces::{ModelInterfaceMetadata, ModelLoadKwargs, ModelSaveKwargs};
use opsml_storage::storage_client;
use opsml_types::contracts::{ArtifactKey, CardRecord, ModelCardClientRecord};
use opsml_types::{
    BaseArgsType, DataType, ModelInterfaceType, ModelType, RegistryType, SaveName, Suffix, TaskType,
};
use opsml_utils::{create_tmp_path, extract_py_attr, get_utc_datetime, PyHelperFuncs};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::{IntoPyObjectExt, PyObject};
use pyo3::{PyTraverseError, PyVisit};
use serde::{
    de::{self, MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt;
use std::path::{Path, PathBuf};
use tracing::error;

fn interface_from_metadata<'py>(
    py: Python<'py>,
    metadata: &ModelInterfaceMetadata,
) -> PyResult<Bound<'py, PyAny>> {
    match metadata.interface_type {
        ModelInterfaceType::Sklearn => SklearnModel::from_metadata(py, metadata),
        ModelInterfaceType::CatBoost => CatBoostModel::from_metadata(py, metadata),
        ModelInterfaceType::LightGBM => LightGBMModel::from_metadata(py, metadata),
        ModelInterfaceType::XGBoost => XGBoostModel::from_metadata(py, metadata),
        ModelInterfaceType::Torch => TorchModel::from_metadata(py, metadata),
        ModelInterfaceType::Lightning => LightningModel::from_metadata(py, metadata),
        ModelInterfaceType::HuggingFace => HuggingFaceModel::from_metadata(py, metadata),

        _ => {
            error!("Interface type not found");
            Err(OpsmlError::new_err("Interface type not found"))
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ModelCardMetadata {
    #[pyo3(get, set)]
    pub datacard_uid: Option<String>,

    #[pyo3(get, set)]
    pub experimentcard_uid: Option<String>,

    #[pyo3(get, set)]
    pub auditcard_uid: Option<String>,

    pub interface_metadata: ModelInterfaceMetadata,
}

#[pymethods]
impl ModelCardMetadata {
    #[new]
    #[pyo3(signature = (datacard_uid=None, experimentcard_uid=None, auditcard_uid=None))]
    pub fn new(
        datacard_uid: Option<&str>,
        experimentcard_uid: Option<&str>,
        auditcard_uid: Option<&str>,
    ) -> Self {
        Self {
            datacard_uid: datacard_uid.map(|s| s.to_string()),
            experimentcard_uid: experimentcard_uid.map(|s| s.to_string()),
            auditcard_uid: auditcard_uid.map(|s| s.to_string()),
            interface_metadata: ModelInterfaceMetadata::default(),
        }
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }
}

#[pyclass]
pub struct ModelCard {
    #[pyo3(get, set)]
    pub interface: Option<PyObject>,

    #[pyo3(get, set)]
    pub space: String,

    #[pyo3(get, set)]
    pub name: String,

    #[pyo3(get, set)]
    pub version: String,

    #[pyo3(get, set)]
    pub uid: String,

    #[pyo3(get, set)]
    pub tags: Vec<String>,

    #[pyo3(get, set)]
    pub metadata: ModelCardMetadata,

    #[pyo3(get)]
    pub registry_type: RegistryType,

    #[pyo3(get)]
    pub to_onnx: bool,

    #[pyo3(get, set)]
    pub app_env: String,

    #[pyo3(get, set)]
    pub created_at: DateTime<Utc>,

    #[pyo3(get)]
    pub is_card: bool,

    #[pyo3(get)]
    pub opsml_version: String,

    artifact_key: Option<ArtifactKey>,
}

#[pymethods]
impl ModelCard {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (interface, space=None, name=None, version=None, uid=None, tags=None, datacard_uid=None, metadata=None, to_onnx=None))]
    pub fn new(
        py: Python,
        interface: &Bound<'_, PyAny>,
        space: Option<&str>,
        name: Option<&str>,
        version: Option<&str>,
        uid: Option<&str>,
        tags: Option<&Bound<'_, PyList>>,
        datacard_uid: Option<&str>,
        metadata: Option<ModelCardMetadata>,
        to_onnx: Option<bool>,
    ) -> PyResult<Self> {
        let registry_type = RegistryType::Model;
        let tags = match tags {
            None => Vec::new(),
            Some(t) => t
                .extract::<Vec<String>>()
                .map_err(|e| OpsmlError::new_err(e.to_string()))?,
        };

        let base_args = map_err_with_logging::<BaseArgsType, _>(
            BaseArgs::create_args(name, space, version, uid, &registry_type),
            "Failed to create base args for ModelCard",
        )?;

        if interface.is_instance_of::<ModelInterface>() {
            //
        } else {
            return Err(OpsmlError::new_err(interface_error()));
        }

        let interface_type = extract_py_attr::<ModelInterfaceType>(interface, "interface_type")?;
        let data_type = extract_py_attr::<DataType>(interface, "data_type")?;
        let model_type = extract_py_attr::<ModelType>(interface, "model_type")?;
        let task_type = extract_py_attr::<TaskType>(interface, "task_type")?;

        let mut metadata = metadata.unwrap_or_default();

        // if metadata.datacard_uid is None, set it to datacard_uid
        if metadata.datacard_uid.is_none() {
            metadata.datacard_uid = datacard_uid.map(|s| s.to_string());
        }

        metadata.interface_metadata.interface_type = interface_type;
        metadata.interface_metadata.data_type = data_type;
        metadata.interface_metadata.model_type = model_type;
        metadata.interface_metadata.task_type = task_type;

        Ok(Self {
            interface: Some(
                interface
                    .into_py_any(py)
                    .map_err(|e| OpsmlError::new_err(e.to_string()))?,
            ),
            space: base_args.0,
            name: base_args.1,
            version: base_args.2,
            uid: base_args.3,
            tags,
            metadata,
            registry_type,
            to_onnx: to_onnx.unwrap_or(false),
            artifact_key: None,
            app_env: std::env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string()),
            created_at: get_utc_datetime(),
            is_card: true,
            opsml_version: opsml_version::version(),
        })
    }

    #[getter]
    pub fn get_onnx_session<'py>(
        &self,
        py: Python<'py>,
    ) -> PyResult<Option<Bound<'py, OnnxSession>>> {
        if let Some(interface) = self.interface.as_ref() {
            let session = interface
                .bind(py)
                .getattr("onnx_session")
                .map_err(|e| OpsmlError::new_err(e.to_string()))?;

            if session.is_none() {
                Ok(None)
            } else {
                let session = session
                    .downcast::<OnnxSession>()
                    .map_err(|e| OpsmlError::new_err(e.to_string()))?;
                Ok(Some(session.clone()))
            }
        } else {
            Ok(None)
        }
    }

    #[getter]
    pub fn datacard_uid(&self) -> Option<&str> {
        self.metadata.datacard_uid.as_deref()
    }

    #[setter]
    pub fn set_datacard_uid(&mut self, datacard_uid: Option<String>) {
        self.metadata.datacard_uid = datacard_uid.map(|s| s.to_string());
    }

    #[getter]
    pub fn experimentcard_uid(&self) -> Option<&str> {
        self.metadata.experimentcard_uid.as_deref()
    }

    #[setter]
    pub fn set_experimentcard_uid(&mut self, experimentcard_uid: Option<String>) {
        self.metadata.experimentcard_uid = experimentcard_uid.map(|s| s.to_string());
    }

    #[getter]
    pub fn auditcard_uid(&self) -> Option<&str> {
        self.metadata.auditcard_uid.as_deref()
    }

    #[setter]
    pub fn set_auditcard_uid(&mut self, auditcard_uid: Option<String>) {
        self.metadata.auditcard_uid = auditcard_uid.map(|s| s.to_string());
    }

    #[setter]
    pub fn set_interface(&mut self, interface: &Bound<'_, PyAny>) -> PyResult<()> {
        if interface.is_instance_of::<ModelInterface>() {
            self.interface = Some(
                interface
                    .into_py_any(interface.py())
                    .map_err(|e| OpsmlError::new_err(e.to_string()))
                    .unwrap(),
            );
            Ok(())
        } else {
            Err(OpsmlError::new_err(
                "interface must be an instance of ModelInterface",
            ))
        }
    }

    pub fn add_tags(&mut self, tags: Vec<String>) {
        self.tags.extend(tags);
    }

    #[pyo3(signature = (path, save_kwargs=None))]
    pub fn save(
        &mut self,
        py: Python,
        path: PathBuf,
        save_kwargs: Option<ModelSaveKwargs>,
    ) -> Result<(), CardError> {
        // save model interface
        // if option raise error
        let model = self
            .interface
            .as_ref()
            .ok_or_else(|| CardError::Error("Model interface not found".to_string()))?;

        // scouter integration: update drift config args
        self.update_drift_config_args(py)?;

        let metadata = model
            .bind(py)
            .call_method("save", (path.clone(), self.to_onnx, save_kwargs), None)
            .map_err(|e| {
                error!("Failed to save model interface: {}", e);
                e
            })?;

        // extract into ModelInterfaceMetadata
        let interface_metadata = metadata.extract::<ModelInterfaceMetadata>().map_err(|e| {
            error!("Failed to extract metadata: {}", e);
            e
        })?;

        // update metadata
        self.metadata.interface_metadata = interface_metadata;

        // save modelcard
        let card_save_path = path.join(SaveName::Card).with_extension(Suffix::Json);
        PyHelperFuncs::save_to_json(&self, &card_save_path)?;

        Ok(())
    }

    #[pyo3(signature = (path=None, onnx=false, load_kwargs=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn load(
        &mut self,
        py: Python,
        path: Option<PathBuf>,
        onnx: bool,
        load_kwargs: Option<ModelLoadKwargs>,
    ) -> PyResult<()> {
        let path = if let Some(p) = path {
            p
        } else {
            let tmp_path = create_tmp_path()?;
            // download assets
            self.download_all_artifacts(&tmp_path)?;
            tmp_path
        };

        let save_metadata = self
            .metadata
            .interface_metadata
            .save_metadata
            .clone()
            .into_bound_py_any(py)?;

        // load model interface
        self.interface.as_ref().unwrap().bind(py).call_method(
            "load",
            (path, save_metadata, onnx, load_kwargs),
            None,
        )?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (path=None))]
    pub fn download_artifacts(&mut self, path: Option<PathBuf>) -> PyResult<()> {
        let path = path.unwrap_or_else(|| PathBuf::from("card_artifacts"));
        self.download_all_artifacts(&path)?;
        Ok(())
    }

    pub fn model_dump_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    #[staticmethod]
    #[pyo3(signature = (json_string, interface=None))]
    pub fn model_validate_json(
        py: Python,
        json_string: String,
        interface: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<ModelCard> {
        let mut card: ModelCard = serde_json::from_str(&json_string).map_err(|e| {
            error!("Failed to validate json: {}", e);
            OpsmlError::new_err(e.to_string())
        })?;

        card.load_interface(py, interface)?;

        Ok(card)
    }

    pub fn __str__(&self) -> String {
        PyHelperFuncs::__str__(self)
    }

    fn __traverse__(&self, visit: PyVisit) -> Result<(), PyTraverseError> {
        if let Some(ref interface) = self.interface {
            visit.call(interface)?;
        }
        Ok(())
    }

    fn __clear__(&mut self) {
        self.interface = None;
    }

    pub fn get_registry_card(&self) -> Result<CardRecord, CardError> {
        let record = ModelCardClientRecord {
            app_env: self.app_env.clone(),
            created_at: self.created_at,
            space: self.space.clone(),
            name: self.name.clone(),
            version: self.version.clone(),
            uid: self.uid.clone(),
            tags: self.tags.clone(),
            datacard_uid: self.metadata.datacard_uid.clone(),
            data_type: self.metadata.interface_metadata.data_type.to_string(),
            model_type: self.metadata.interface_metadata.model_type.to_string(),
            experimentcard_uid: self.metadata.experimentcard_uid.clone(),
            auditcard_uid: self.metadata.auditcard_uid.clone(),
            interface_type: self.metadata.interface_metadata.interface_type.to_string(),
            task_type: self.metadata.interface_metadata.task_type.to_string(),
            opsml_version: self.opsml_version.clone(),
            username: std::env::var("OPSML_USERNAME").unwrap_or_else(|_| "guest".to_string()),
        };

        Ok(CardRecord::Model(record))
    }

    pub fn save_card(&self, path: PathBuf) -> Result<(), CardError> {
        let card_save_path = path.join(SaveName::Card).with_extension(Suffix::Json);
        PyHelperFuncs::save_to_json(self, &card_save_path)?;

        Ok(())
    }

    /// Get the model from the interface if available.
    /// This will result in an error if the interface is not set and
    /// the model is not available.
    #[getter]
    pub fn model<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        if let Some(interface) = self.interface.as_ref() {
            // get property "model" from interface
            let model = interface
                .bind(py)
                .getattr("model")
                .map_err(|e| OpsmlError::new_err(e.to_string()))?;

            // check if model is None
            if model.is_none() {
                Err(OpsmlError::new_err(
                    "Model has not been set. Load the model and retry.",
                ))
            // return model
            } else {
                Ok(model)
            }
        } else {
            Err(OpsmlError::new_err(
                "Model interface not found. Load the interface and retry.",
            ))
        }
    }
}

impl ModelCard {
    pub fn set_artifact_key(&mut self, key: ArtifactKey) {
        self.artifact_key = Some(key);
    }
    fn update_drift_config_args(&self, py: Python) -> PyResult<()> {
        let interface = self.interface.as_ref().unwrap().bind(py);
        let drift_profiles = interface.getattr("drift_profile")?;
        // downcast to list
        let drift_profiles = drift_profiles.downcast::<PyList>()?;

        // if drift_profiles is empty, return
        if drift_profiles.len() == 0 {
            Ok(())
        } else {
            // iterate over drift_profiles and call "update_config_args"
            let kwargs = PyDict::new(py);
            kwargs.set_item("name", self.name.clone())?;
            kwargs.set_item("space", self.space.clone())?;
            kwargs.set_item("version", self.version.clone())?;

            for profile in drift_profiles.iter() {
                // get actual profile from enum
                profile.call_method("update_config_args", (), Some(&kwargs))?;
            }

            Ok(())
        }
    }

    fn load_interface(&mut self, py: Python, interface: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
        if let Some(interface) = interface {
            self.set_interface(interface)
        } else {
            // match interface type
            let interface = interface_from_metadata(py, &self.metadata.interface_metadata)?;
            self.set_interface(&interface)
        }
    }
}

impl Serialize for ModelCard {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ModelCard", 11)?;

        // set session to none
        state.serialize_field("name", &self.name)?;
        state.serialize_field("space", &self.space)?;
        state.serialize_field("version", &self.version)?;
        state.serialize_field("uid", &self.uid)?;
        state.serialize_field("tags", &self.tags)?;
        state.serialize_field("metadata", &self.metadata)?;
        state.serialize_field("registry_type", &self.registry_type)?;
        state.serialize_field("to_onnx", &self.to_onnx)?;
        state.serialize_field("created_at", &self.created_at)?;
        state.serialize_field("app_env", &self.app_env)?;
        state.serialize_field("is_card", &self.is_card)?;
        state.serialize_field("opsml_version", &self.opsml_version)?;
        state.end()
    }
}

impl FromPyObject<'_> for ModelCard {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        let interface = ob.getattr("interface")?;
        let name = ob.getattr("name")?.extract()?;
        let space = ob.getattr("space")?.extract()?;

        let version = ob.getattr("version")?.extract()?;
        let uid = ob.getattr("uid")?.extract()?;
        let tags = ob.getattr("tags")?.extract()?;
        let metadata = ob.getattr("metadata")?.extract()?;
        let registry_type = ob.getattr("registry_type")?.extract()?;
        let to_onnx = ob.getattr("to_onnx")?.extract()?;
        let created_at = ob.getattr("created_at")?.extract()?;
        let app_env = ob.getattr("app_env")?.extract()?;
        let opsml_version = ob.getattr("opsml_version")?.extract()?;

        Ok(ModelCard {
            interface: Some(interface.into()),
            name,
            space,
            version,
            uid,
            tags,
            metadata,
            registry_type,
            to_onnx,
            artifact_key: None,
            app_env,
            created_at,
            is_card: true,
            opsml_version,
        })
    }
}

impl<'de> Deserialize<'de> for ModelCard {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Interface,
            Name,
            Space,
            Version,
            Uid,
            Tags,
            Metadata,
            RegistryType,
            ToOnnx,
            AppEnv,
            CreatedAt,
            IsCard,
            OpsmlVersion,
        }

        struct ModelCardVisitor;

        impl<'de> Visitor<'de> for ModelCardVisitor {
            type Value = ModelCard;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct ModelCard")
            }

            fn visit_map<V>(self, mut map: V) -> Result<ModelCard, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut interface = None;
                let mut name = None;
                let mut space = None;
                let mut version = None;
                let mut uid = None;
                let mut tags = None;
                let mut metadata = None;
                let mut registry_type = None;
                let mut to_onnx = None;
                let mut app_env = None;
                let mut created_at = None;
                let mut is_card = None;
                let mut opsml_version = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Interface => {
                            let _interface: Option<serde_json::Value> = map.next_value()?;
                            interface = None; // Default to None always (pyobject)
                        }
                        Field::Name => {
                            name = Some(map.next_value()?);
                        }
                        Field::Space => {
                            space = Some(map.next_value()?);
                        }

                        Field::Version => {
                            version = Some(map.next_value()?);
                        }
                        Field::Uid => {
                            uid = Some(map.next_value()?);
                        }
                        Field::Tags => {
                            tags = Some(map.next_value()?);
                        }
                        Field::Metadata => {
                            metadata = Some(map.next_value()?);
                        }
                        Field::RegistryType => {
                            registry_type = Some(map.next_value()?);
                        }
                        Field::ToOnnx => {
                            to_onnx = Some(map.next_value()?);
                        }
                        Field::AppEnv => {
                            app_env = Some(map.next_value()?);
                        }
                        Field::CreatedAt => {
                            created_at = Some(map.next_value()?);
                        }
                        Field::IsCard => {
                            is_card = Some(map.next_value()?);
                        }
                        Field::OpsmlVersion => {
                            opsml_version = Some(map.next_value()?);
                        }
                    }
                }

                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let space = space.ok_or_else(|| de::Error::missing_field("space"))?;
                let version = version.ok_or_else(|| de::Error::missing_field("version"))?;
                let uid = uid.ok_or_else(|| de::Error::missing_field("uid"))?;
                let tags = tags.ok_or_else(|| de::Error::missing_field("tags"))?;
                let metadata = metadata.ok_or_else(|| de::Error::missing_field("metadata"))?;
                let registry_type =
                    registry_type.ok_or_else(|| de::Error::missing_field("registry_type"))?;
                let to_onnx = to_onnx.ok_or_else(|| de::Error::missing_field("to_onnx"))?;
                let app_env = app_env.ok_or_else(|| de::Error::missing_field("app_env"))?;
                let created_at =
                    created_at.ok_or_else(|| de::Error::missing_field("created_at"))?;
                let is_card = is_card.ok_or_else(|| de::Error::missing_field("is_card"))?;
                let opsml_version =
                    opsml_version.ok_or_else(|| de::Error::missing_field("opsml_version"))?;

                Ok(ModelCard {
                    interface,
                    name,
                    space,
                    version,
                    uid,
                    tags,
                    metadata,
                    registry_type,
                    to_onnx,
                    artifact_key: None,
                    app_env,
                    created_at,
                    is_card,
                    opsml_version,
                })
            }
        }

        const FIELDS: &[&str] = &[
            "interface",
            "name",
            "space",
            "version",
            "uid",
            "tags",
            "metadata",
            "registry_type",
            "to_onnx",
            "app_env",
            "created_at",
            "is_card",
            "opsml_version",
        ];
        deserializer.deserialize_struct("ModelCard", FIELDS, ModelCardVisitor)
    }
}

impl ModelCard {
    fn get_decryption_key(&self) -> Result<Vec<u8>, CardError> {
        if self.artifact_key.is_none() {
            Err(CardError::Error("Decryption key not found".to_string()))
        } else {
            Ok(self.artifact_key.as_ref().unwrap().get_decrypt_key()?)
        }
    }
    fn download_all_artifacts(&mut self, lpath: &Path) -> Result<(), CardError> {
        let decrypt_key = self.get_decryption_key()?;
        let uri = self.artifact_key.as_ref().unwrap().storage_path();

        storage_client()?
            .get(lpath, &uri, true)
            .map_err(|e| CardError::Error(format!("Failed to download artifacts: {}", e)))?;

        decrypt_directory(lpath, &decrypt_key)?;

        Ok(())
    }
}
